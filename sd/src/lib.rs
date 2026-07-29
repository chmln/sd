mod error;
mod input;
pub mod replacer;
mod unescape;

use std::{
    borrow::Cow,
    fs,
    io::{BufRead, BufWriter, Read, Write},
    path::PathBuf,
};

pub use self::error::{Error, FailedJobs, Result};
pub use self::input::{Source, open_source, read_source};
pub use self::replacer::Replacer;

/// Core processing function that handles file replacement
///
/// Returns whether the pattern matched any of the inputs, so that callers can
/// implement `--must-match`. Reporting it rather than acting on it keeps the
/// exit-status policy in the CLI.
pub fn process_sources(
    replacer: &Replacer,
    sources: &[Source],
    preview: bool,
    line_by_line: bool,
    output_writer: &mut dyn Write,
) -> Result<bool> {
    if line_by_line {
        return process_sources_line_by_line(
            replacer,
            sources,
            preview,
            output_writer,
        );
    }

    let mut inputs = Vec::new();
    for source in sources.iter() {
        let input = match source {
            Source::File(path) => {
                if path.exists() {
                    read_source(source)?
                } else {
                    return Err(Error::InvalidPath(path.to_owned()));
                }
            }
            Source::Stdin => read_source(source)?,
        };

        inputs.push(input);
    }

    let needs_separator = sources.len() > 1;

    let replaced: Vec<_> = {
        use rayon::prelude::*;
        inputs
            .par_iter()
            .map(|input| replacer.replace(input))
            .collect()
    };

    // `replace` only allocates when the pattern matched, so the `Cow` variant
    // already tells us whether anything changed.
    let matched = replaced.iter().any(|r| matches!(r, Cow::Owned(_)));

    if preview || sources.first() == Some(&Source::Stdin) {
        for (source, replaced) in sources.iter().zip(replaced) {
            if needs_separator {
                writeln!(output_writer, "----- {} -----", source.display())?;
            }
            output_writer.write_all(&replaced)?;
        }
    } else {
        let mut failed_jobs = Vec::new();
        for (source, replaced) in sources.iter().zip(replaced) {
            match source {
                Source::File(path) => {
                    if let Err(e) = write_with_temp(path, &replaced) {
                        failed_jobs.push((path.to_owned(), e));
                    }
                }
                _ => unreachable!("stdin should go previous branch"),
            }
        }
        if !failed_jobs.is_empty() {
            return Err(Error::FailedJobs(FailedJobs(failed_jobs)));
        }
    }

    Ok(matched)
}

fn process_sources_line_by_line(
    replacer: &Replacer,
    sources: &[Source],
    preview: bool,
    output_writer: &mut dyn Write,
) -> Result<bool> {
    let needs_separator = sources.len() > 1;
    let mut matched = false;

    if preview || sources.first() == Some(&Source::Stdin) {
        for source in sources {
            if needs_separator {
                writeln!(output_writer, "----- {} -----", source.display())?;
            }
            let reader = open_source(source)?;
            matched |=
                process_reader_line_by_line(replacer, reader, output_writer)?;
        }
    } else {
        // Pre-validate all files before modifying any, matching the
        // whole-file processing path which opens all inputs upfront.
        for source in sources {
            match source {
                Source::File(path) => {
                    if !path.exists() {
                        return Err(Error::InvalidPath(path.to_owned()));
                    }
                    std::fs::File::open(path)?;
                }
                _ => unreachable!("stdin should go previous branch"),
            }
        }

        let mut failed_jobs = Vec::new();
        for source in sources {
            match source {
                Source::File(path) => {
                    match write_file_line_by_line(replacer, path) {
                        Ok(file_matched) => matched |= file_matched,
                        Err(e) => failed_jobs.push((path.to_owned(), e)),
                    }
                }
                _ => unreachable!("stdin should go previous branch"),
            }
        }
        if !failed_jobs.is_empty() {
            return Err(Error::FailedJobs(FailedJobs(failed_jobs)));
        }
    }

    Ok(matched)
}

fn process_reader_line_by_line(
    replacer: &Replacer,
    mut reader: Box<dyn BufRead + '_>,
    writer: &mut dyn Write,
) -> Result<bool> {
    const CHUNK_SIZE: usize = 8192;

    let mut chunk = vec![0u8; CHUNK_SIZE];
    let mut line = Vec::with_capacity(256);
    let mut matched = false;

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            // Finish any remaining line
            if !line.is_empty() {
                let replaced = replacer.replace(&line);
                matched |= matches!(replaced, Cow::Owned(_));
                writer.write_all(&replaced)?;
            }
            break;
        }

        let mut start = 0;
        for (i, &byte) in chunk[..n].iter().enumerate() {
            if byte == b'\n' {
                // Found a complete line
                line.extend_from_slice(&chunk[start..i]);
                let replaced = replacer.replace(&line);
                matched |= matches!(replaced, Cow::Owned(_));
                writer.write_all(&replaced)?;
                writer.write_all(b"\n")?;
                line.clear();
                start = i + 1;
            }
        }

        // Keep partial line for next chunk
        if start < n {
            line.extend_from_slice(&chunk[start..n]);
        }
    }

    Ok(matched)
}

fn write_file_line_by_line(
    replacer: &Replacer,
    path: &PathBuf,
) -> Result<bool> {
    let canonical = fs::canonicalize(path)?;

    let temp = tempfile::NamedTempFile::new_in(
        canonical
            .parent()
            .ok_or_else(|| Error::InvalidPath(canonical.to_path_buf()))?,
    )?;

    if let Ok(metadata) = fs::metadata(&canonical) {
        temp.as_file().set_permissions(metadata.permissions()).ok();
    }

    let matched = {
        let source = Source::File(path.clone());
        let reader = open_source(&source)?;
        let mut writer = BufWriter::new(temp.as_file());
        let matched =
            process_reader_line_by_line(replacer, reader, &mut writer)?;
        writer.flush()?;
        matched
    };

    temp.persist(&canonical)?;

    Ok(matched)
}

fn write_with_temp(path: &PathBuf, data: &[u8]) -> Result<()> {
    let path = fs::canonicalize(path)?;

    let mut temp = tempfile::NamedTempFile::new_in(
        path.parent()
            .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?,
    )?;

    let file = temp.as_file();
    file.set_len(data.len() as u64)?;
    if let Ok(metadata) = fs::metadata(&path) {
        file.set_permissions(metadata.permissions()).ok();
    }

    if !data.is_empty() {
        temp.as_file_mut().write_all(data)?;
        temp.as_file_mut().flush()?;
    }

    temp.persist(&path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_process_sources_with_preview() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc123def").unwrap();

        let replacer =
            Replacer::new("abc".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, true, false, &mut output)?;

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "xyz123def");

        Ok(())
    }

    #[test]
    fn test_process_sources_in_place() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc123def").unwrap();

        let replacer =
            Replacer::new("abc".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path.clone())];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, false, false, &mut output)?;

        let result = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(result, "xyz123def");

        Ok(())
    }

    #[test]
    fn test_process_sources_nonexistent_file() {
        let replacer =
            Replacer::new("abc".into(), "def".into(), false, None, 0).unwrap();
        let nonexistent = PathBuf::from("/nonexistent/file.txt");
        let sources = vec![Source::File(nonexistent.clone())];
        let mut output = Vec::new();

        let result =
            process_sources(&replacer, &sources, false, false, &mut output);
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::InvalidPath(path) => assert_eq!(path, nonexistent),
            _ => panic!("Expected InvalidPath error"),
        }
    }

    #[test]
    fn test_write_with_temp() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "original").unwrap();

        let new_data = b"new content";
        write_with_temp(&file_path, new_data)?;

        let result = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(result, "new content");

        Ok(())
    }

    #[test]
    fn test_process_sources_line_by_line_preview() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc123\ndef456\n").unwrap();

        let replacer =
            Replacer::new("abc".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, true, true, &mut output)?;

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "xyz123\ndef456\n");

        Ok(())
    }

    #[test]
    fn test_process_sources_line_by_line_in_place() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc123\ndef456\n").unwrap();

        let replacer =
            Replacer::new("abc".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path.clone())];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, false, true, &mut output)?;

        let result = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(result, "xyz123\ndef456\n");

        Ok(())
    }

    #[test]
    fn test_line_by_line_no_trailing_newline() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc").unwrap();

        let replacer =
            Replacer::new("abc".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, true, true, &mut output)?;

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "xyz");

        Ok(())
    }

    #[test]
    fn test_line_by_line_caret_no_phantom() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "1\n2\n3\n").unwrap();

        let replacer = Replacer::new("^".into(), "p-".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, true, true, &mut output)?;

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "p-1\np-2\np-3\n");

        Ok(())
    }

    #[test]
    fn test_process_sources_reports_match() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc123def").unwrap();

        let replacer =
            Replacer::new("abc".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];

        for line_by_line in [false, true] {
            let mut output = Vec::new();
            assert!(process_sources(
                &replacer,
                &sources,
                true,
                line_by_line,
                &mut output
            )?);
        }

        Ok(())
    }

    #[test]
    fn test_process_sources_reports_no_match() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "abc123def").unwrap();

        let replacer =
            Replacer::new("nowhere".into(), "xyz".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];

        for line_by_line in [false, true] {
            let mut output = Vec::new();
            assert!(!process_sources(
                &replacer,
                &sources,
                true,
                line_by_line,
                &mut output
            )?);
        }

        Ok(())
    }

    /// A match in any input counts, whichever input it was.
    #[test]
    fn test_process_sources_reports_match_in_later_source() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let miss = temp_dir.path().join("miss.txt");
        let hit = temp_dir.path().join("hit.txt");
        std::fs::write(&miss, "xyz\n").unwrap();
        std::fs::write(&hit, "abc\n").unwrap();

        let replacer =
            Replacer::new("abc".into(), "def".into(), false, None, 0)?;
        let sources =
            vec![Source::File(miss.clone()), Source::File(hit.clone())];

        for line_by_line in [false, true] {
            let mut output = Vec::new();
            assert!(process_sources(
                &replacer,
                &sources,
                true,
                line_by_line,
                &mut output
            )?);
        }

        Ok(())
    }

    #[test]
    fn test_line_by_line_whitespace_trim() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "a \nb \n").unwrap();

        let replacer =
            Replacer::new(r"\s+$".into(), "".into(), false, None, 0)?;
        let sources = vec![Source::File(file_path)];
        let mut output = Vec::new();

        process_sources(&replacer, &sources, true, true, &mut output)?;

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "a\nb\n");

        Ok(())
    }
}

mod error;
mod input;
pub mod replacer;
mod unescape;

use std::{
    fs,
    io::{BufRead, BufWriter, Read, Write},
    path::PathBuf,
};

pub use self::error::{Error, FailedJobs, Result};
pub use self::input::{Source, open_source, read_source};
pub use self::replacer::Replacer;

/// Core processing function that handles file replacement
pub fn process_sources(
    replacer: &Replacer,
    sources: &[Source],
    preview: bool,
    line_by_line: bool,
    output_writer: &mut dyn Write,
) -> Result<()> {
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

    Ok(())
}

fn process_sources_line_by_line(
    replacer: &Replacer,
    sources: &[Source],
    preview: bool,
    output_writer: &mut dyn Write,
) -> Result<()> {
    let needs_separator = sources.len() > 1;

    if preview || sources.first() == Some(&Source::Stdin) {
        for source in sources {
            if needs_separator {
                writeln!(output_writer, "----- {} -----", source.display())?;
            }
            let reader = open_source(source)?;
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
                    if let Err(e) = write_file_line_by_line(replacer, path) {
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

    Ok(())
}

fn process_reader_line_by_line(
    replacer: &Replacer,
    mut reader: Box<dyn BufRead + '_>,
    writer: &mut dyn Write,
) -> Result<()> {
    const CHUNK_SIZE: usize = 8192;

    let mut chunk = vec![0u8; CHUNK_SIZE];
    let mut line = Vec::with_capacity(256);

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            // Finish any remaining line
            if !line.is_empty() {
                let replaced = replacer.replace(&line);
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

    Ok(())
}

fn write_file_line_by_line(replacer: &Replacer, path: &PathBuf) -> Result<()> {
    let canonical = fs::canonicalize(path)?;

    let temp = tempfile::NamedTempFile::new_in(
        canonical
            .parent()
            .ok_or_else(|| Error::InvalidPath(canonical.to_path_buf()))?,
    )?;

    if let Ok(metadata) = fs::metadata(&canonical) {
        temp.as_file().set_permissions(metadata.permissions()).ok();
    }

    {
        let source = Source::File(path.clone());
        let reader = open_source(&source)?;
        let mut writer = BufWriter::new(temp.as_file());
        process_reader_line_by_line(replacer, reader, &mut writer)?;
        writer.flush()?;
    }

    temp.persist(&canonical)?;

    Ok(())
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

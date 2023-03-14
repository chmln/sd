mod cli;
mod error;
mod input;

pub(crate) mod replacer;

use clap::Parser;
use memmap2::MmapMut;
use std::{
    fs,
    io::{stdout, BufRead, Write},
    ops::DerefMut,
    path::PathBuf,
    process,
};

pub(crate) use self::error::{Error, FailedJobs, Result};
pub(crate) use self::input::Source;
use self::input::{make_mmap, make_mmap_stdin};
use self::replacer::Replacer;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let options = cli::Options::parse();

    let replacer = Replacer::new(
        options.find,
        options.replace_with,
        options.literal_mode,
        options.flags,
        options.replacements,
    )?;

    let sources = if !options.files.is_empty() {
        Source::from_paths(options.files)
    } else {
        Source::from_stdin()
    };

    if options.line_buffered && sources == Source::from_stdin() {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        let mut buffer = String::new();

        loop {
            let mut read_handle = stdin.lock();
            let bytes_read = read_handle.read_line(&mut buffer)?;

            // .lock()
            // .read_line(&mut buffer)
            // .expect("Error reading from standard input");
            if bytes_read == 0 {
                break;
            }
            let replaced = replacer.replace(buffer.as_bytes());
            handle
                .write_all(replaced.as_ref())
                .expect("Error writing to standard output");
            handle.flush().expect("Error flushing output");
            buffer.clear();
        }

        return Ok(());
    }

    let mut mmaps = Vec::new();
    for source in sources.iter() {
        let mmap = match source {
            Source::File(path) => {
                if path.exists() {
                    unsafe { make_mmap(&path)? }
                } else {
                    return Err(Error::InvalidPath(path.to_owned()));
                }
            }
            Source::Stdin => make_mmap_stdin()?,
        };

        mmaps.push(mmap);
    }

    let needs_separator = sources.len() > 1;

    let replaced: Vec<_> = {
        use rayon::prelude::*;
        mmaps
            .par_iter()
            .map(|mmap| replacer.replace(&mmap))
            .collect()
    };

    if options.preview || sources.first() == Some(&Source::Stdin) {
        let mut handle = stdout().lock();

        for (source, replaced) in sources.iter().zip(replaced) {
            if needs_separator {
                writeln!(handle, "----- {} -----", source.display())?;
            }
            handle.write_all(&replaced)?;
        }
    } else {
        // Windows requires closing mmap before writing:
        // > The requested operation cannot be performed on a file with a user-mapped section open
        #[cfg(target_family = "windows")]
        let replaced: Vec<Vec<u8>> =
            replaced.into_iter().map(|r| r.to_vec()).collect();
        #[cfg(target_family = "windows")]
        drop(mmaps);

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

fn write_with_temp(path: &PathBuf, data: &[u8]) -> Result<()> {
    let path = fs::canonicalize(path)?;

    let temp = tempfile::NamedTempFile::new_in(
        path.parent()
            .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?,
    )?;

    let file = temp.as_file();
    file.set_len(data.len() as u64)?;
    if let Ok(metadata) = fs::metadata(&path) {
        file.set_permissions(metadata.permissions()).ok();
    }

    if !data.is_empty() {
        let mut mmap_temp = unsafe { MmapMut::map_mut(file)? };
        mmap_temp.deref_mut().write_all(data)?;
        mmap_temp.flush_async()?;
    }

    temp.persist(&path)?;

    Ok(())
}

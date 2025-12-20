mod cli;
mod error;
mod input;
mod output;

pub(crate) mod replacer;
mod unescape;

use clap::Parser;
use std::{
    io::{Write, stdout},
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
        Source::from_paths(options.files)?
    } else {
        Source::from_stdin()
    };

    let mmaps = sources
        .iter()
        .map(|source| {
            Ok(match source {
                Source::File(path) => make_mmap(path)?,
                Source::Stdin => make_mmap_stdin()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let replaced: Vec<_> = {
        use rayon::prelude::*;
        mmaps
            .par_iter()
            .map(|mmap| replacer.replace(mmap))
            .collect()
    };

    if options.preview || sources.first() == Some(&Source::Stdin) {
        let needs_separator = sources.len() > 1;
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

        let failed_jobs = sources
            .iter()
            .zip(replaced)
            .filter_map(|(source, replaced)| match source {
                Source::File(path) => output::write_atomic(path, &replaced)
                    .err()
                    .map(|e| (path.to_owned(), e)),
                _ => None,
            })
            .collect::<Vec<_>>();

        if !failed_jobs.is_empty() {
            return Err(Error::FailedJobs(FailedJobs(failed_jobs)));
        }
    }

    Ok(())
}

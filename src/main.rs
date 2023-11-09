mod cli;
mod error;
mod input;

pub(crate) mod replacer;

use std::{
    fs,
    process,
    path::PathBuf,
    io::{stdout, Write},
    ops::DerefMut,
};
use memmap2::{Mmap, MmapMut};
use clap::Parser;

pub(crate) use self::input::Source;
pub(crate) use self::error::{Error, Result};
use self::replacer::Replacer;
use self::input::{make_mmap, make_mmap_stdin};

fn main() -> Result<()> {
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

    let sources: Vec<(&Source, Result<Mmap>)> = sources
        .iter()
        .map(|source| (source, match source {
            Source::File(path) => if path.exists() {
                make_mmap(path)
            } else {
                Err(Error::InvalidPath(path.clone()))
            },
            Source::Stdin => make_mmap_stdin()
        }))
        .collect();
    let needs_separator = sources.len() > 1;

    let replaced: Vec<_> = {
        use rayon::prelude::*;
        sources
            .par_iter()
            .map(|(source, maybe_mmap)| (source, maybe_mmap.as_ref().map(|mmap| replacer.replace(mmap))))
            .collect()
    };

    let mut errors = Vec::new();
    let mut has_error = !errors.is_empty();

    if options.preview || sources.get(0).map(|s| s.0) == Some(&Source::Stdin) {
        let mut handle = stdout().lock();

        for (source, replaced) in replaced.iter().filter(|(_, r)| r.is_ok()) {
            if needs_separator {
                writeln!(handle, "----- {} -----", source.display())?;
            }
            handle.write_all(replaced.as_ref().unwrap())?;
        }
    } else {
        for (source, replaced) in replaced.iter().filter(|(_, r)| r.is_ok()) {
            let path = match source {
                Source::File(path) => path,
                Source::Stdin => {
                    unreachable!("stdin should only go previous branch")
                }
            };

            if let Err(e) = write_with_temp(path, replaced.as_ref().unwrap()) {
                errors.push((source, e));
            }
        }
    }

    for (source, error) in replaced.iter().filter(|(_, r)| r.is_err()) {
        eprintln!("error: {}: {:?}", source.display(), error);
        has_error = true;
    }
    for (source, error) in errors {
        eprintln!("error: {}: {:?}", source.display(), error);
    }

    if has_error {
        process::exit(1);
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

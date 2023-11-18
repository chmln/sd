mod cli;
mod error;
mod input;

pub(crate) mod replacer;

use clap::Parser;
use memmap2::MmapMut;
use std::{
    fs,
    io::{stdout, Write},
    ops::DerefMut,
    path::PathBuf,
    process,
};

pub(crate) use self::error::{Error, Result};
pub(crate) use self::input::Source;
use self::input::{make_mmap, make_mmap_stdin};
use self::replacer::Replacer;

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

    let mut errors = Vec::new();

    let mut mmaps = Vec::new();
    for source in sources.iter() {
        let maybe_mmap = match source {
            Source::File(path) => {
                if path.exists() {
                    unsafe { make_mmap(&path) }
                } else {
                    Err(Error::InvalidPath(path.clone()))
                }
            }
            Source::Stdin => make_mmap_stdin(),
        };

        match maybe_mmap {
            Ok(mmap) => mmaps.push(Some(mmap)),
            Err(e) => {
                mmaps.push(None);
                errors.push((source, e));
            }
        };
    }

    let needs_separator = sources.len() > 1;

    let replaced: Vec<_> = {
        use rayon::prelude::*;
        mmaps
            .par_iter()
            .map(|maybe_mmap| {
                maybe_mmap.as_ref().map(|mmap| replacer.replace(&mmap))
            })
            .collect()
    };

    if options.preview || sources.first() == Some(&Source::Stdin) {
        let mut handle = stdout().lock();

        for (source, replaced) in
            sources.iter().zip(replaced).filter(|(_, r)| r.is_some())
        {
            if needs_separator {
                writeln!(handle, "----- {} -----", source.display())?;
            }
            handle.write_all(replaced.as_ref().unwrap())?;
        }
    } else {
        // Windows requires closing mmap before writing:
        // > The requested operation cannot be performed on a file with a user-mapped section open
        #[cfg(target_family = "windows")]
        let replaced: Vec<Option<Vec<u8>>> = replaced
            .into_iter()
            .map(|r| r.map(|r| r.to_vec()))
            .collect();
        #[cfg(target_family = "windows")]
        drop(mmaps);

        for (source, replaced) in
            sources.iter().zip(replaced).filter(|(_, r)| r.is_some())
        {
            let path = match source {
                Source::File(path) => path,
                _ => unreachable!("stdin should go previous branch"),
            };
            if let Err(e) = write_with_temp(path, &replaced.unwrap()) {
                errors.push((source, e));
            }
        }
    }

    if !errors.is_empty() {
        for (source, error) in errors {
            eprintln!("error: {}: {:?}", source.display(), error);
        }
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

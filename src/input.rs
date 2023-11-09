use std::{
    fs::{self, File},
    io::{stdin, stdout, Read, Write},
    ops::DerefMut,
    path::PathBuf,
};

use crate::{Error, Replacer, Result};

use memmap2::{Mmap, MmapMut, MmapOptions};

#[derive(Debug, PartialEq)]
pub(crate) enum Source {
    Stdin,
    File(PathBuf),
}

impl Source {
    pub(crate) fn from_paths(paths: Vec<PathBuf>) -> Vec<Self> {
        paths.into_iter().map(Self::File).collect()
    }

    pub(crate) fn from_stdin() -> Vec<Self> {
        vec![Self::Stdin]
    }

	fn display(&self) -> String {
		match self {
			Self::Stdin => "STDIN".to_string(),
			Self::File(path) => format!("FILE {}", path.display()),
		}
	}
}

pub(crate) struct App {
    replacer: Replacer,
    sources: Vec<Source>,
}

impl App {
    pub(crate) fn new(sources: Vec<Source>, replacer: Replacer) -> Self {
        Self { sources, replacer }
    }

    pub(crate) fn run(&self, preview: bool) -> Result<()> {
        let sources: Vec<(&Source, Result<Mmap>)> = self.sources
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
                .map(|(source, maybe_mmap)| (source, maybe_mmap.as_ref().map(|mmap| self.replacer.replace(mmap))))
                .collect()
        };

        let mut errors = Vec::new();

        if preview || self.sources.get(0) == Some(&Source::Stdin) {
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
        }
        for (source, error) in errors {
            eprintln!("error: {}: {:?}", source.display(), error);
        }

        Ok(())
    }
}

fn make_mmap(path: &PathBuf) -> Result<Mmap> {
    Ok(unsafe { Mmap::map(&File::open(path)?)? })
}

fn make_mmap_stdin() -> Result<Mmap> {
    let mut handle = stdin().lock();
    let mut buf = Vec::new();
    handle.read_to_end(&mut buf)?;
    let mut mmap = MmapOptions::new().len(buf.len()).map_anon()?;
    mmap.copy_from_slice(&buf);
    let mmap = mmap.make_read_only()?;
    Ok(mmap)
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

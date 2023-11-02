use std::{
	fs::{File, self},
	io::{Write, stdin, stdout, Read},
	path::PathBuf,
	ops::DerefMut,
};

use crate::{Error, Replacer, Result};

use memmap2::{Mmap, MmapMut, MmapOptions};

#[derive(Debug, PartialEq)]
pub(crate) enum Source {
    Stdin,
    Files(Vec<PathBuf>),
}

pub(crate) struct App {
    replacer: Replacer,
    source: Source,
}

impl App {
    pub(crate) fn new(source: Source, replacer: Replacer) -> Self {
        Self { source, replacer }
    }

    pub(crate) fn run(&self, preview: bool) -> Result<()> {
        let sources: Vec<(PathBuf, Mmap)> = match &self.source {
            Source::Stdin => {
				let mut handle = stdin().lock();
				let mut buf = Vec::new();
				handle.read_to_end(&mut buf)?;
				let mut mmap = MmapOptions::new()
					.len(buf.len())
					.map_anon()?;
				mmap.copy_from_slice(&buf);
				let mmap = mmap.make_read_only()?;
				vec![(PathBuf::from("STDIN"), mmap)]
			},
            Source::Files(paths) => {
				let mut refs = Vec::new();
                for path in paths {
                    if !path.exists() {
                        return Err(Error::InvalidPath(path.clone()));
                    }
					let mmap = unsafe { Mmap::map(&File::open(path)?)? };
                    refs.push((path.clone(), mmap));
                }
                refs
            },
        };
        let needs_separator = sources.len() > 1;

        let replaced: Vec<_> = {
            use rayon::prelude::*;
            sources.par_iter()
                .map(|(path, mmap)| {
					let replaced = self.replacer.replace(mmap); 
					(path, mmap, replaced)
				})
                .collect()
        };

        if preview || self.source == Source::Stdin {
            let mut handle = stdout().lock();

            for (path, _, replaced) in replaced {
                if needs_separator {
                    writeln!(handle, "----- FILE {} -----", path.display())?;
                }
                handle.write_all(replaced.as_ref())?;
            }
        } else {
            for (path, _, replaced) in replaced {
                let source = File::open(path)?;
                let meta = fs::metadata(path)?;
                drop(source);

                let target = tempfile::NamedTempFile::new_in(
                    path.parent()
                        .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?,
                )?;
                let file = target.as_file();
                file.set_len(replaced.len() as u64)?;
                file.set_permissions(meta.permissions())?;

                if !replaced.is_empty() {
                    let mut mmap_target = unsafe { MmapMut::map_mut(file)? };
                    mmap_target.deref_mut().write_all(&replaced)?;
                    mmap_target.flush_async()?;
                }
                
                target.persist(fs::canonicalize(path)?)?;
            }
        }

        Ok(())
    }
}

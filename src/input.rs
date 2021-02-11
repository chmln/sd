use crate::{Replacer, Result};
use std::{fs::File, io::prelude::*, path::PathBuf};

pub(crate) enum Source {
    Stdin,
    Files(Vec<PathBuf>),
}

impl Source {
    pub(crate) fn glob(glob: String) -> Result<Self> {
        Ok(Self::Files(
            globwalk::glob(glob)?
                .into_iter()
                .filter_map(|entry| {
                    if let Ok(e) = entry {
                        Some(e.into_path())
                    } else {
                        None
                    }
                })
                .collect(),
        ))
    }
}

pub(crate) struct App {
    replacer: Replacer,
    source: Source,
}

impl App {
    pub(crate) fn new(source: Source, replacer: Replacer) -> Self {
        Self { source, replacer }
    }
    pub(crate) fn run(&self, in_place: bool) -> Result<()> {
        match (&self.source, in_place) {
            (Source::Stdin, _) => {
                let mut buffer = Vec::with_capacity(256);
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_end(&mut buffer)?;

                let stdout = std::io::stdout();
                let mut handle = stdout.lock();

                if self.replacer.has_matches(&buffer) {
                    handle.write_all(&self.replacer.replace(&buffer))?;
                } else {
                    handle.write_all(&buffer)?;
                }

                Ok(())
            }
            (Source::Files(paths), true) => {
                use rayon::prelude::*;

                #[allow(unused_must_use)]
                paths.par_iter().for_each(|p| {
                    self.replacer.replace_file(p).map_err(|e| {
                        eprintln!("Error processing {}: {}", p.display(), e)
                    });
                });

                Ok(())
            }
            (Source::Files(paths), false) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();

                paths.iter().try_for_each(|path| {
                    if let Err(_) = Replacer::check_not_empty(File::open(path)?)
                    {
                        return Ok(());
                    }
                    let file =
                        unsafe { memmap::Mmap::map(&File::open(path)?)? };
                    handle.write_all(&self.replacer.replace(&file))?;

                    Ok(())
                })
            }
        }
    }
}

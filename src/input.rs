use crate::{Replacer, Result};
use std::{fs::File, io::prelude::*, path::PathBuf};

#[derive(Debug)]
pub(crate) enum Source {
    Stdin,
    Files(Vec<PathBuf>),
}

impl Source {
    pub(crate) fn recursive() -> Result<Self> {
        Ok(Self::Files(
            ignore::WalkBuilder::new(".")
                .hidden(false)
                .filter_entry(|e| e.file_name() != ".git")
                .build()
                .filter_map(|d| d.ok())
                .filter_map(|d| match d.file_type() {
                    Some(t) if t.is_file() => Some(d.into_path()),
                    _ => None,
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
    pub(crate) fn run(&self, preview: bool) -> Result<()> {
        let is_tty = atty::is(atty::Stream::Stdout);

        match (&self.source, preview) {
            (Source::Stdin, _) => {
                let mut buffer = Vec::with_capacity(256);
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_end(&mut buffer)?;

                let stdout = std::io::stdout();
                let mut handle = stdout.lock();

                handle.write_all(&if is_tty {
                    self.replacer.replace_preview(&buffer)
                } else {
                    self.replacer.replace(&buffer)
                })?;

                Ok(())
            }
            (Source::Files(paths), false) => {
                use rayon::prelude::*;

                #[allow(unused_must_use)]
                paths.par_iter().for_each(|p| {
                    self.replacer.replace_file(p).map_err(|e| {
                        eprintln!("Error processing {}: {}", p.display(), e)
                    });
                });

                Ok(())
            }
            (Source::Files(paths), true) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                let print_path = paths.len() > 1;

                paths.iter().try_for_each(|path| {
                    if let Err(_) = Replacer::check_not_empty(File::open(path)?)
                    {
                        return Ok(());
                    }
                    let file =
                        unsafe { memmap::Mmap::map(&File::open(path)?)? };
                    if self.replacer.has_matches(&file) {
                        if print_path {
                            writeln!(
                                handle,
                                "----- FILE {} -----",
                                path.display()
                            )?;
                        }

                        handle
                            .write_all(&self.replacer.replace_preview(&file))?;
                        writeln!(handle)?;
                    }

                    Ok(())
                })
            }
        }
    }
}

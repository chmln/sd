use std::{fs::File, io::prelude::*, path::PathBuf};

use crate::{Error, Replacer, Result};

use is_terminal::IsTerminal;

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
    fn stdin_replace(&self, is_tty: bool) -> Result<()> {
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

    pub(crate) fn new(source: Source, replacer: Replacer) -> Self {
        Self { source, replacer }
    }
    pub(crate) fn run(&self, preview: bool) -> Result<()> {
        let is_tty = std::io::stdout().is_terminal();

        match (&self.source, preview) {
            (Source::Stdin, true) => {
                eprintln!("WARN: `--preview` flag is redundant");
                self.stdin_replace(is_tty)
            }
            (Source::Stdin, false) => self.stdin_replace(is_tty),
            (Source::Files(paths), false) => {
                use rayon::prelude::*;

                let failed_jobs: Vec<_> = paths
                    .par_iter()
                    .filter_map(|p| {
                        if let Err(e) = self.replacer.replace_file(p) {
                            Some((p.to_owned(), e))
                        } else {
                            None
                        }
                    })
                    .collect();

                if failed_jobs.is_empty() {
                    Ok(())
                } else {
                    let failed_jobs =
                        crate::error::FailedJobs::from(failed_jobs);
                    Err(Error::FailedProcessing(failed_jobs))
                }
            }
            (Source::Files(paths), true) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                let print_path = paths.len() > 1;

                paths.iter().try_for_each(|path| {
                    if Replacer::check_not_empty(File::open(path)?).is_err() {
                        return Ok(());
                    }
                    let file =
                        unsafe { memmap2::Mmap::map(&File::open(path)?)? };
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

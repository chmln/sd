use crate::{utils, Result};
use regex::Regex;
use std::{borrow::Cow, fs::File, io::prelude::*};

pub(crate) enum Source {
    Stdin,
    Files(Vec<String>),
}

impl Source {
    pub(crate) fn from(file_paths: Vec<String>) -> Self {
        if file_paths.len() == 0 {
            return Source::Stdin;
        }
        return Source::Files(file_paths);
    }

    fn file_to_string(path: impl AsRef<str>) -> Result<String> {
        let mut file = File::open(path.as_ref())?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}

pub(crate) struct Replacer<'a> {
    regex: Regex,
    replace_with: &'a str,
    is_literal: bool,
}

impl<'a> Replacer<'a> {
    pub(crate) fn new(
        look_for: String,
        replace_with: &'a str,
        is_literal: bool,
        flags: Option<String>,
    ) -> Result<Self> {
        let look_for = if is_literal {
            regex::escape(&look_for)
        }
        else {
            look_for
        };

        let mut regex = regex::RegexBuilder::new(&look_for);
        regex.case_insensitive(!utils::regex_case_sensitive(&look_for));

        if let (Some(flags), false) = (flags, is_literal) {
            for c in flags.chars() {
                match c {
                    'c' => { regex.case_insensitive(false); },
                    'i' => { regex.case_insensitive(true); },
                    'm' => { regex.multi_line(true); },
                    _ => {},
                };
            }
        }

        return Ok(Replacer {
            regex: regex.build()?,
            replace_with,
            is_literal,
        });
    }

    fn replace<'r>(&self, content: impl Into<Cow<'r, str>>) -> Cow<'r, str> {
        let content = content.into();
        if self.regex.find(&content).is_none() {
            return content;
        }

        if self.is_literal {
            self.regex
                .replace_all(&content, regex::NoExpand(self.replace_with))
                .to_string()
                .into()
        }
        else {
            self.regex
                .replace_all(&content, self.replace_with)
                .to_string()
                .into()
        }
    }

    pub(crate) fn run(&self, source: &Source, in_place: bool) -> Result<()> {
        match source {
            Source::Stdin => {
                let mut buffer = String::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_string(&mut buffer)?;

                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                handle.write(&self.replace(&buffer).as_bytes())?;
                Ok(())
            },
            Source::Files(paths) => {
                use atomic_write::atomic_write;
                use rayon::prelude::*;

                if in_place {
                    paths
                        .par_iter()
                        .map(|p| {
                            Ok(atomic_write(
                                p,
                                &*self.replace(&Source::file_to_string(p)?),
                            )?)
                        })
                        .collect::<Result<Vec<()>>>()?;
                    Ok(())
                }
                else {
                    let stdout = std::io::stdout();
                    let mut handle = stdout.lock();

                    paths
                        .iter()
                        .map(|p| {
                            handle.write(
                                &self
                                    .replace(&Source::file_to_string(p)?)
                                    .as_bytes(),
                            )?;
                            Ok(())
                        })
                        .collect::<Result<Vec<()>>>()?;
                    Ok(())
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {}

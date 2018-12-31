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

pub(crate) enum Replacer<'a> {
    Regex(Regex, &'a str),
    Literal(&'a str, &'a str),
}

impl<'a> Replacer<'a> {
    pub(crate) fn new(
        look_for: &'a str,
        replace_with: &'a str,
        is_literal: bool,
        flags: Option<String>
    ) -> Result<Self> {
        if is_literal {
            return Ok(Replacer::Literal(look_for, replace_with));
        }

        let mut regex = regex::RegexBuilder::new(look_for);
        regex.case_insensitive(!utils::regex_case_sensitive(look_for));

        if let Some(flags) = flags {
            for c in flags.chars() {
                match c {
                    'c' => { regex.case_insensitive(false); },
                    'i' => { regex.case_insensitive(true); },
                    'm' => { regex.multi_line(true); }
                    _ => {}
                };
            }
        }
        
        return Ok(Replacer::Regex(regex.build()?, replace_with));
    }

    pub(crate) fn replace<'r>(&self, content: impl Into<Cow<'r, str>>) -> Cow<'r, str> {
        let content = content.into();
        match self {
            Replacer::Regex(regex, replace_with) => {
                if regex.find(&content).is_none() {
                    return content;
                }

                let replaced =
                    regex.replace_all(&content, *replace_with).to_string();
                utils::unescape(&replaced).unwrap_or_else(|| replaced).into()
            },
            Replacer::Literal(search, replace_with) => {
                content.replace(search, replace_with).into()
            },
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

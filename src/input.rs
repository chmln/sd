use crate::{utils, Error, Result};
use regex::bytes::Regex;
use std::{fs::File, io::prelude::*};

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
}

pub(crate) struct Replacer {
    regex: Regex,
    replace_with: Vec<u8>,
    is_literal: bool,
}

impl Replacer {
    pub(crate) fn new(
        look_for: String,
        replace_with: String,
        is_literal: bool,
        flags: Option<String>,
    ) -> Result<Self> {
        let (look_for, replace_with) = if is_literal {
            (regex::escape(&look_for), replace_with.into_bytes())
        } else {
            (
                look_for,
                utils::unescape(&replace_with)
                    .unwrap_or_else(|| replace_with)
                    .into_bytes(),
            )
        };

        let mut regex = regex::bytes::RegexBuilder::new(&look_for);

        if let Some(flags) = flags {
            for c in flags.chars() {
                match c {
                    'c' => {
                        regex.case_insensitive(false);
                    },
                    'i' => {
                        regex.case_insensitive(true);
                    },
                    'm' => {
                        regex.multi_line(true);
                    },
                    'w' => {
                        regex = regex::bytes::RegexBuilder::new(&format!(
                            "\\b{}\\b",
                            look_for
                        ));
                    },
                    _ => {},
                };
            }
        };

        Ok(Replacer {
            regex: regex.build()?,
            replace_with,
            is_literal,
        })
    }

    fn has_matches(&self, content: &[u8]) -> bool {
        self.regex.is_match(content)
    }

    fn replace<'a>(&'a self, content: &'a [u8]) -> std::borrow::Cow<'a, [u8]> {
        if self.is_literal {
            self.regex.replace_all(
                &content,
                regex::bytes::NoExpand(&self.replace_with),
            )
        } else {
            self.regex.replace_all(&content, &*self.replace_with)
        }
    }

    fn replace_file(&self, path: impl AsRef<str>) -> Result<()> {
        use memmap::{Mmap, MmapMut};
        use std::ops::DerefMut;

        let path = std::path::Path::new(path.as_ref());
        let source = File::open(path)?;
        let meta = source.metadata()?;
        let mmap_source = unsafe { Mmap::map(&source)? };
        let replaced = self.replace(mmap_source.as_ref());

        let target = tempfile::NamedTempFile::new_in(
            path.parent().ok_or_else(|| Error {
                message: "Invalid path given".to_owned(),
            })?,
        )?;
        let file = target.as_file();
        file.set_len(replaced.len() as u64)?;
        file.set_permissions(meta.permissions())?;

        let mut mmap_target = unsafe { MmapMut::map_mut(&file)? };
        mmap_target.deref_mut().write_all(&replaced)?;
        mmap_target.flush()?;

        drop(source);
        drop(mmap_source);

        target.persist(path)?;
        Ok(())
    }

    pub(crate) fn run(&self, source: &Source, in_place: bool) -> Result<()> {
        match (source, in_place) {
            (Source::Stdin, _) => {
                let mut buffer = Vec::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_end(&mut buffer)?;

                if self.has_matches(&buffer) {
                    let stdout = std::io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(&self.replace(&buffer))?;
                }
                Ok(())
            },
            (Source::Files(paths), true) => {
                use rayon::prelude::*;

                paths
                    .par_iter()
                    .map(|p| self.replace_file(p).map_err(Error::log))
                    .collect::<Vec<Result<()>>>();
                Ok(())
            },
            (Source::Files(paths), false) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();

                paths
                    .iter()
                    .map(|path| {
                        let file =
                            unsafe { memmap::Mmap::map(&File::open(&path)?)? };
                        handle.write_all(&self.replace(&file[..]))?;
                        Ok(())
                    })
                    .collect::<Result<Vec<()>>>()?;
                Ok(())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace<'a>(
        look_for: impl Into<String>,
        replace_with: impl Into<String>,
        literal: bool,
        flags: Option<&'static str>,
        src: &'static str,
        target: &'static str,
    ) {
        let replacer = Replacer::new(
            look_for.into(),
            replace_with.into(),
            literal,
            flags.map(ToOwned::to_owned),
        )
        .unwrap();
        assert_eq!(
            std::str::from_utf8(&replacer.replace(src.as_bytes())),
            Ok(target)
        );
    }

    #[test]
    fn default_global() {
        replace("a", "b", false, None, "aaa", "bbb");
    }

    #[test]
    fn escaped_char_preservation() {
        replace("a", "b", false, None, "a\\n", "b\\n");
    }

    #[test]
    fn case_sensitive_default() {
        replace("abc", "x", false, None, "abcABC", "xABC");
        replace("abc", "x", true, None, "abcABC", "xABC");
    }

    #[test]
    fn sanity_check_literal_replacements() {
        replace("((special[]))", "x", true, None, "((special[]))y", "xy");
    }

    #[test]
    fn unescape_regex_replacements() {
        replace("test", r"\n", false, None, "testtest", "\n\n");
    }

    #[test]
    fn no_unescape_literal_replacements() {
        replace("test", r"\n", true, None, "testtest", r"\n\n");
    }

    #[test]
    fn full_word_replace() {
        replace("abc", "def", false, Some("w"), "abcd abc", "abcd def");
    }

}

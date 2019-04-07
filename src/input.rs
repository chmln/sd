use crate::{utils, Result};
use regex::bytes::Regex;
use std::{fs::File, fs::set_permissions, io::prelude::*};

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
        }
        else {
            (
                look_for,
                utils::unescape(&replace_with).unwrap_or_else(|| replace_with).into_bytes(),
            )
        };

        let mut regex = regex::bytes::RegexBuilder::new(&look_for);
        regex.case_insensitive(
            !is_literal && !utils::regex_case_sensitive(&look_for),
        );

        if let (Some(flags), false) = (flags, is_literal) {
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
        self.regex.find(&content).is_some()
    }

    fn replace(&self, content: impl AsRef<[u8]>) -> Vec<u8> {
        let content = content.as_ref();
        if self.is_literal {
            self.regex
                .replace_all(&content, regex::bytes::NoExpand(&self.replace_with))
                .into_owned()
        }
        else {
            self.regex
                .replace_all(&content, &*self.replace_with)
                .into_owned()
        }
    }

    fn replace_file(&self, path: impl AsRef<str>) -> Result<()> {
        use memmap::{Mmap, MmapMut};

        let path = std::path::Path::new(path.as_ref());
        let source = File::open(path)?;
        let mmap_source = unsafe { Mmap::map(&source)? };
        let meta = source.metadata()?;
        let permissions = meta.permissions();

        let target = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        let replaced = self.replace(&mmap_source[..]);
        let file = target.as_file();

        file.set_len(meta.len())?;
        let mut mmap_target = unsafe { MmapMut::map_mut(&file).unwrap() };
        (&mut mmap_target[..]).write(&replaced)?;
        mmap_target.flush()?;

        target.persist(path)?;
        set_permissions(path, permissions)?;
        Ok(())
    }

    pub(crate) fn run(&self, source: &Source, in_place: bool) -> Result<()> {
        match (source, in_place) {
            (Source::Stdin, _) => {
                let mut buffer = String::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_string(&mut buffer)?;
                let buffer = buffer.as_bytes();

                if self.has_matches(&buffer) {
                    let stdout = std::io::stdout();
                    let mut handle = stdout.lock();
                    handle.write(&self.replace(buffer))?;
                }
                Ok(())
            },
            (Source::Files(paths), true) => {
                use rayon::prelude::*;

                paths
                    .par_iter()
                    .map(|p| self.replace_file(p))
                    .collect::<Result<Vec<()>>>()
                    .map(|_| ())
            },
            (Source::Files(paths), false) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();

                paths
                    .iter()
                    .map(|path| {
                        unsafe {
                            let file = memmap::Mmap::map(&File::open(&path)?)?;
                            handle.write(
                                &self
                                    .replace(&file[..])
                            )?;
                            Ok(())
                        }
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

    #[test]
    fn default_global() -> Result<()> {
        let r = Replacer::new("a".into(), "b".into(), false, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("aaa")), Ok("bbb"));
        Ok(())
    }

    #[test]
    fn escaped_char_preservation() -> Result<()> {
        let r = Replacer::new("a".into(), "b".into(), false, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("a\\n")), Ok("b\\n"));
        Ok(())
    }

    #[test]
    fn smart_case_insensitive() -> Result<()> {
        let r = Replacer::new("abc".into(), "x".into(), false, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("abcABCAbcabC")), Ok("xxxx"));
        Ok(())
    }

    #[test]
    fn smart_case_sensitive() -> Result<()> {
        let r = Replacer::new("Abc".into(), "x".into(), false, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("abcABCAbcabC")), Ok("abcABCxabC"));
        Ok(())
    }

    #[test]
    fn no_smart_case_literals() -> Result<()> {
        let r = Replacer::new("abc".into(), "x".into(), true, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("abcABCAbcabC")), Ok("xABCAbcabC"));
        Ok(())
    }

    #[test]
    fn sanity_check_literal_replacements() -> Result<()> {
        let r = Replacer::new("((special[]))".into(), "x".into(), true, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("((special[]))y")), Ok("xy"));
        Ok(())
    }

    #[test]
    fn unescape_regex_replacements() -> Result<()> {
        let r = Replacer::new("test".into(), r"\n".into(), false, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("testtest")), Ok("\n\n"));

        // escaping the newline char
        let r = Replacer::new("test".into(), r"\\n".into(), false, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("testtest")), Ok(r"\n\n"));
        Ok(())
    }

    #[test]
    fn no_unescape_literal_replacements() -> Result<()> {
        let r = Replacer::new("test".into(), r"\n".into(), true, None)?;
        assert_eq!(std::str::from_utf8(&r.replace("testtest")), Ok(r"\n\n"));
        Ok(())
    }

}

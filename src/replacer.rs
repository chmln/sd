use crate::{utils, Error, Result};
use regex::bytes::Regex;
use std::{borrow::Cow, fs, fs::File, io::prelude::*, path::Path};

pub(crate) struct Replacer {
    regex: Regex,
    replace_with: Vec<u8>,
    is_literal: bool,
    replacements: usize,
}

impl Replacer {
    pub(crate) fn new(
        look_for: String,
        replace_with: String,
        is_literal: bool,
        flags: Option<String>,
        replacements: Option<usize>,
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
        regex.multi_line(true);

        if let Some(flags) = flags {
            flags.chars().for_each(|c| {
                #[rustfmt::skip]
                match c {
                    'c' => { regex.case_insensitive(false); },
                    'i' => { regex.case_insensitive(true); },
                    'm' => {},
                    'e' => { regex.multi_line(false); },
                    's' => {
                        if !flags.contains("m") {
                            regex.multi_line(false);
                        }
                        regex.dot_matches_new_line(true);
                    },
                    'w' => {
                        regex = regex::bytes::RegexBuilder::new(&format!(
                            "\\b{}\\b",
                            look_for
                        ));
                    },
                    _ => {},
                };
            });
        };

        Ok(Self {
            regex: regex.build()?,
            replace_with,
            is_literal,
            replacements: replacements.unwrap_or(0),
        })
    }

    pub(crate) fn has_matches(&self, content: &[u8]) -> bool {
        self.regex.is_match(content)
    }

    pub(crate) fn check_not_empty(mut file: File) -> Result<()> {
        let mut buf: [u8; 1] = Default::default();
        file.read_exact(&mut buf)?;
        Ok(())
    }

    pub(crate) fn replace<'a>(&'a self, content: &'a [u8]) -> Cow<'a, [u8]> {
        if self.is_literal {
            self.regex.replacen(
                &content,
                self.replacements,
                regex::bytes::NoExpand(&self.replace_with),
            )
        } else {
            self.regex.replacen(
                &content,
                self.replacements,
                &*self.replace_with,
            )
        }
    }

    pub(crate) fn replace_preview<'a>(
        &'a self,
        content: &'a [u8],
    ) -> Cow<'a, [u8]> {
        use ansi_term::Color;
        use itertools::Itertools;
        use regex::bytes::Replacer;

        let captures = self
            .regex
            .captures_iter(content)
            .enumerate()
            .collect::<Vec<_>>();
        let num_captures = captures.len();

        if num_captures == 0 {
            return Cow::Borrowed(content);
        }

        let surrounding_text = self.regex.split(content).collect::<Vec<_>>();
        let mut output = Vec::<u8>::with_capacity(5000);

        captures.into_iter().for_each(|(capture_index, capture)| {
            let text_before = surrounding_text.get(capture_index).unwrap();
            let text_after = surrounding_text.get(capture_index + 1);

            let l_pos = text_before
                .iter()
                .positions(|c| c == &b'\n')
                .collect::<Vec<_>>();

            if l_pos.len() > 0 {
                if let Some(i) = l_pos
                .get(l_pos.len() - 3)
                .or_else(|| l_pos.get(l_pos.len() - 2))
                .or_else(|| l_pos.get(l_pos.len() - 1))
            {
                output.extend_from_slice(&text_before[*i..]);
            }
            }

            output.extend_from_slice(
                Color::Green.prefix().to_string().as_bytes(),
            );

            if self.is_literal {
                regex::bytes::NoExpand(&self.replace_with)
                    .replace_append(&capture, &mut output);
            } else {
                (&*self.replace_with).replace_append(&capture, &mut output);
            }

            output.extend_from_slice(
                Color::Green.suffix().to_string().as_bytes(),
            );

            if let Some(text_after) = text_after {
                let pos = text_after
                    .iter()
                    .positions(|c| c == &b'\n')
                    .collect::<Vec<_>>();

                if let Some(i) =
                    pos.get(2).or_else(|| pos.get(1)).or_else(|| pos.get(0))
                {
                    output.extend_from_slice(&text_after[..*i]);
                    output.push(b'\n');
                    if num_captures > 1 && capture_index != num_captures - 1 {
                        output.extend_from_slice(&[b'.', b'.', b'.', b'\n'])
                    }
                } else {
                    output.extend_from_slice(&text_after);
                }
            }
        });

        return Cow::Owned(output);
    }

    pub(crate) fn replace_file(&self, path: &Path) -> Result<()> {
        use memmap::{Mmap, MmapMut};
        use std::ops::DerefMut;

        if let Err(_) = Self::check_not_empty(File::open(path)?) {
            return Ok(());
        }

        let source = File::open(path)?;
        let meta = fs::metadata(path)?;
        let mmap_source = unsafe { Mmap::map(&source)? };
        let replaced = self.replace(&mmap_source);

        let target = tempfile::NamedTempFile::new_in(
            path.parent()
                .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?,
        )?;
        let file = target.as_file();
        file.set_len(replaced.len() as u64)?;
        file.set_permissions(meta.permissions())?;

        if !replaced.is_empty() {
            let mut mmap_target = unsafe { MmapMut::map_mut(&file)? };
            mmap_target.deref_mut().write_all(&replaced)?;
            mmap_target.flush_async()?;
        }

        drop(mmap_source);
        drop(source);

        target.persist(fs::canonicalize(path)?)?;
        Ok(())
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
            None,
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

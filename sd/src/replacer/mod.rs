use std::borrow::Cow;

use crate::{Result, unescape};

use regex::bytes::Regex;

#[cfg(test)]
mod tests;
mod validate;

pub use validate::{InvalidReplaceCapture, validate_replace};

pub struct Replacer {
    regex: Regex,
    replace_with: Vec<u8>,
    is_literal: bool,
    replacements: usize,
}

impl Replacer {
    pub fn new(
        look_for: String,
        replace_with: String,
        is_literal: bool,
        flags: Option<String>,
        replacements: usize,
    ) -> Result<Self> {
        let (look_for, replace_with) = if is_literal {
            (regex::escape(&look_for), replace_with.into_bytes())
        } else {
            validate_replace(&replace_with)?;

            (look_for, unescape::unescape(&replace_with).into_bytes())
        };

        // `w` rewrites the pattern rather than setting a builder option, so it
        // has to be resolved before the builder exists. Otherwise the builder
        // would have to be replaced mid-loop, discarding every option set by a
        // flag to the left of the `w`.
        //
        // The group is non-capturing so that `\b` binds to the whole pattern
        // rather than to its first and last alternatives: `foo|bar` has to
        // become `\b(?:foo|bar)\b`, not `\bfoo|bar\b`. A capturing group would
        // shift the user's own `$1` to `$2`.
        let word_boundary = flags.as_deref().is_some_and(|f| f.contains('w'));
        let pattern = if word_boundary {
            format!("\\b(?:{look_for})\\b")
        } else {
            look_for
        };

        let mut regex = regex::bytes::RegexBuilder::new(&pattern);
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
                        if !flags.contains('m') {
                            regex.multi_line(false);
                        }
                        regex.dot_matches_new_line(true);
                    },
                    // handled above, before the builder was constructed
                    'w' => {},
                    _ => {},
                };
            });
        };

        Ok(Self {
            regex: regex.build()?,
            replace_with,
            is_literal,
            replacements,
        })
    }

    pub fn replace<'a>(&'a self, content: &'a [u8]) -> Cow<'a, [u8]> {
        let regex = &self.regex;
        let limit = self.replacements;
        let use_color = false;
        if self.is_literal {
            Self::replacen(
                regex,
                limit,
                content,
                use_color,
                regex::bytes::NoExpand(&self.replace_with),
            )
        } else {
            Self::replacen(
                regex,
                limit,
                content,
                use_color,
                &*self.replace_with,
            )
        }
    }

    /// A modified form of [`regex::bytes::Regex::replacen`] that supports
    /// coloring replacements
    pub fn replacen<'haystack, R: regex::bytes::Replacer>(
        regex: &regex::bytes::Regex,
        limit: usize,
        haystack: &'haystack [u8],
        _use_color: bool,
        mut rep: R,
    ) -> Cow<'haystack, [u8]> {
        let mut it = regex.captures_iter(haystack).enumerate().peekable();
        if it.peek().is_none() {
            return Cow::Borrowed(haystack);
        }
        let mut new = Vec::with_capacity(haystack.len());
        let mut last_match = 0;
        for (i, cap) in it {
            // unwrap on 0 is OK because captures only reports matches
            let m = cap.get(0).unwrap();
            new.extend_from_slice(&haystack[last_match..m.start()]);
            rep.replace_append(&cap, &mut new);
            last_match = m.end();
            if limit > 0 && i >= limit - 1 {
                break;
            }
        }
        new.extend_from_slice(&haystack[last_match..]);
        Cow::Owned(new)
    }
}

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
                        if !flags.contains('m') {
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
            replacements,
        })
    }

    pub fn replace<'a>(&'a self, content: &'a [u8]) -> Cow<'a, [u8]> {
        let mut budget = self.budget();
        self.replace_within_budget(content, &mut budget)
    }

    /// The number of replacements this replacer may perform per input.
    /// [`usize::MAX`] when unlimited.
    pub fn budget(&self) -> usize {
        if self.replacements == 0 {
            usize::MAX
        } else {
            self.replacements
        }
    }

    /// Like [`Replacer::replace`], but bounded by `budget`, which is
    /// decremented by the number of replacements performed. This lets a single
    /// limit span several calls, e.g. one call per line of the same file.
    pub fn replace_within_budget<'a>(
        &'a self,
        content: &'a [u8],
        budget: &mut usize,
    ) -> Cow<'a, [u8]> {
        let regex = &self.regex;
        let use_color = false;
        if self.is_literal {
            Self::replacen(
                regex,
                budget,
                content,
                use_color,
                regex::bytes::NoExpand(&self.replace_with),
            )
        } else {
            Self::replacen(
                regex,
                budget,
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
        budget: &mut usize,
        haystack: &'haystack [u8],
        _use_color: bool,
        mut rep: R,
    ) -> Cow<'haystack, [u8]> {
        if *budget == 0 {
            return Cow::Borrowed(haystack);
        }
        let mut it = regex.captures_iter(haystack).peekable();
        if it.peek().is_none() {
            return Cow::Borrowed(haystack);
        }
        let mut new = Vec::with_capacity(haystack.len());
        let mut last_match = 0;
        for cap in it {
            // unwrap on 0 is OK because captures only reports matches
            let m = cap.get(0).unwrap();
            new.extend_from_slice(&haystack[last_match..m.start()]);
            rep.replace_append(&cap, &mut new);
            last_match = m.end();
            *budget -= 1;
            if *budget == 0 {
                break;
            }
        }
        new.extend_from_slice(&haystack[last_match..]);
        Cow::Owned(new)
    }
}

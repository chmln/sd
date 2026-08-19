use std::{borrow::Cow, process::Command};

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
        let regex = &self.regex;
        let limit = self.replacements;
        if self.is_literal {
            Self::replacen(
                regex,
                limit,
                content,
                regex::bytes::NoExpand(&self.replace_with),
            )
        } else {
            Self::replacen(regex, limit, content, &*self.replace_with)
        }
    }

    // e.g. "((($((echo $(echo ciao) $(echo ciao)))))) $(echo hola)" => ["$((echo $(echo ciao) $(echo ciao)))", "$(echo hola)"]
    fn find_dollar_parenthesis(bytes: &[u8]) -> Vec<&[u8]> {
        fn check_if_dollar_paren_begin(char1: u8, char2: u8) -> bool {
            let cond = char1 == b'$' && char2 == b'(';
            cond
        }
        let mut res = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            // iterate until u find $(
            while i + 1 < bytes.len()
                && !(check_if_dollar_paren_begin(bytes[i], bytes[i + 1]))
            {
                i += 1;
            }
            let no_dollar_parens_found = i + 1 == bytes.len();
            if no_dollar_parens_found {
                return res;
            }

            let start_open_dollar = i;
            let mut depth_parens = 0;
            while i < bytes.len() {
                fn is_non_escaped_open_paren(bytes: &[u8], i: usize) -> bool {
                    let char2 = bytes[i];
                    if i == 0 {
                        return char2 == b'(';
                    }
                    let char1 = bytes[i - 1];
                    let cond = char1 != b'\\' && char2 == b'(';
                    cond
                }
                fn is_non_escaped_close_paren(bytes: &[u8], i: usize) -> bool {
                    let char2 = bytes[i];
                    if i == 0 {
                        return char2 == b')';
                    }

                    let char1 = bytes[i - 1];
                    let cond = char1 != b'\\' && char2 == b')';
                    cond
                }

                if is_non_escaped_open_paren(bytes, i) {
                    depth_parens += 1;
                }
                if is_non_escaped_close_paren(bytes, i) {
                    depth_parens -= 1;
                    if depth_parens == 0 {
                        res.push(&bytes[start_open_dollar..i + 1]);
                        break;
                    }
                }

                i += 1;
            }
        }
        res
    }

    /// A modified form of [`regex::bytes::Regex::replacen`] that supports
    /// coloring replacements
    pub fn replacen<'haystack, R: regex::bytes::Replacer>(
        regex: &regex::bytes::Regex,
        limit: usize,
        haystack: &'haystack [u8],
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
            // can't eval function only once due to non-pure functions

            let mut content_with_group_replaced = Vec::new();
            rep.replace_append(&cap, &mut content_with_group_replaced);

            let mut content_with_group_replaced =
                String::from_utf8_lossy(content_with_group_replaced.as_slice())
                    .into_owned();

            let content_w_group_replaced_bytes =
                content_with_group_replaced.clone();
            let content_w_group_replaced_bytes =
                content_w_group_replaced_bytes.as_bytes();

            let outer_dollar_parens_vec =
                Self::find_dollar_parenthesis(content_w_group_replaced_bytes);
            let there_are_cmds_to_eval = outer_dollar_parens_vec.len() != 0;

            if there_are_cmds_to_eval {
                // can't use regex to get outer parenthesis https://stackoverflow.com/questions/3469089/writing-a-regex-to-capture-text-between-outer-parenthesis
                for outer_dollar_paren in outer_dollar_parens_vec {
                    let command =
                        &outer_dollar_paren[2..(outer_dollar_paren.len() - 1)];
                    let command = String::from_utf8(command.to_vec()).unwrap();
                    let output = Command::new("bash")
                        .arg("-c")
                        .arg(
                            String::from_utf8(command.as_bytes().to_vec())
                                .unwrap(),
                        )
                        .output()
                        .unwrap();
                    let output = String::from_utf8(output.stdout).unwrap();
                    let output = output.trim_end_matches(['\r', '\n']);
                    let substring_w_cmd_to_be_replaced =
                        &String::from_utf8(outer_dollar_paren.to_vec())
                            .unwrap();

                    content_with_group_replaced = content_with_group_replaced
                        .replace(substring_w_cmd_to_be_replaced, output);
                }
                new.extend_from_slice(content_with_group_replaced.as_bytes());
            } else {
                rep.replace_append(&cap, &mut new);
            }

            last_match = m.end();
            if limit > 0 && i >= limit - 1 {
                break;
            }
        }
        new.extend_from_slice(&haystack[last_match..]);
        Cow::Owned(new)
    }
}

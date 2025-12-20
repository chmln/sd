use std::{error::Error, fmt, str::CharIndices};

#[derive(Debug)]
pub struct InvalidReplaceCapture {
    original_replace: String,
    invalid_ident: Span,
    num_leading_digits: usize,
}

impl Error for InvalidReplaceCapture {}

// NOTE: This code is much more allocation heavy than it needs to be, but it's
//       only displayed as a hard error to the user, so it's not a big deal
impl fmt::Display for InvalidReplaceCapture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(Clone, Copy)]
        enum SpecialChar {
            Newline,
            CarriageReturn,
            Tab,
        }

        impl SpecialChar {
            fn new(c: char) -> Option<Self> {
                match c {
                    '\n' => Some(Self::Newline),
                    '\r' => Some(Self::CarriageReturn),
                    '\t' => Some(Self::Tab),
                    _ => None,
                }
            }

            /// Renders as the character from the "Control Pictures" block
            ///
            /// https://en.wikipedia.org/wiki/Control_Pictures
            fn render(self) -> char {
                match self {
                    Self::Newline => '␊',
                    Self::CarriageReturn => '␍',
                    Self::Tab => '␉',
                }
            }
        }

        let Self {
            original_replace,
            invalid_ident,
            num_leading_digits,
        } = self;

        // Build up the error to show the user
        let mut formatted = String::new();
        let mut arrows_start = Span::start_at(0);
        for (byte_index, c) in original_replace.char_indices() {
            let (prefix, suffix, text) = match SpecialChar::new(c) {
                Some(c) => {
                    (
                        Some("" /* special prefix */),
                        Some("" /* special suffix */),
                        c.render(),
                    )
                }
                None => {
                    let (prefix, suffix) = if byte_index == invalid_ident.start
                    {
                        (Some("" /* error prefix */), None)
                    } else if byte_index
                        == invalid_ident.end.checked_sub(1).unwrap()
                    {
                        (None, Some("" /* error suffix */))
                    } else {
                        (None, None)
                    };
                    (prefix, suffix, c)
                }
            };

            if let Some(prefix) = prefix {
                formatted.push_str(prefix);
            }
            formatted.push(text);
            if let Some(suffix) = suffix {
                formatted.push_str(suffix);
            }

            if byte_index < invalid_ident.start {
                // Assumes that characters have a base display width of 1. While
                // that's not technically true, it's near impossible to do right
                // since the specifics on text rendering is up to the user's
                // terminal/font. This _does_ rely on variable-width characters
                // like \n, \r, and \t getting converting to single character
                // representations above
                arrows_start.start += 1;
            }
        }

        // This relies on all non-curly-braced capture chars being 1 byte
        let arrows_span = arrows_start.end_offset(invalid_ident.len());
        let mut arrows = " ".repeat(arrows_span.start);
        arrows.push_str(&"^".repeat(arrows_span.len()));

        let ident = invalid_ident.slice(original_replace);
        let (number, the_rest) = ident.split_at(*num_leading_digits);
        let disambiguous = format!("${{{number}}}{the_rest}");
        let error_message = format!(
            "The numbered capture group `${number}` in the replacement text is ambiguous.",
        );
        let hint_message = format!(
            "{}: Use curly braces to disambiguate it `{}`.",
            "hint", disambiguous
        );

        writeln!(f, "{}", error_message)?;
        writeln!(f, "{}", hint_message)?;
        writeln!(f, "{}", formatted)?;
        write!(f, "{}", arrows)
    }
}

pub fn validate_replace(s: &str) -> Result<(), InvalidReplaceCapture> {
    for ident in ReplaceCaptureIter::new(s) {
        let mut char_it = ident.name.char_indices();
        let (_, c) = char_it.next().unwrap();
        if c.is_ascii_digit() {
            for (i, c) in char_it {
                if !c.is_ascii_digit() {
                    return Err(InvalidReplaceCapture {
                        original_replace: s.to_owned(),
                        invalid_ident: ident.span,
                        num_leading_digits: i,
                    });
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn start_at(start: usize) -> SpanOpen {
        SpanOpen { start }
    }

    fn new(start: usize, end: usize) -> Self {
        // `<` instead of `<=` because `Span` is exclusive on the upper bound
        assert!(start < end);
        Self { start, end }
    }

    fn slice(self, s: &str) -> &str {
        &s[self.start..self.end]
    }

    fn len(self) -> usize {
        self.end - self.start
    }
}

#[derive(Clone, Copy)]
struct SpanOpen {
    start: usize,
}

impl SpanOpen {
    fn end_at(self, end: usize) -> Span {
        let Self { start } = self;
        Span::new(start, end)
    }

    fn end_offset(self, offset: usize) -> Span {
        assert_ne!(offset, 0);
        let Self { start } = self;
        self.end_at(start + offset)
    }
}

#[derive(Debug)]
struct Capture<'rep> {
    name: &'rep str,
    span: Span,
}

impl<'rep> Capture<'rep> {
    fn new(name: &'rep str, span: Span) -> Self {
        Self { name, span }
    }
}

/// An iterator over the capture idents in an interpolated replacement string
///
/// This code is adapted from the `regex` crate
/// <https://docs.rs/regex-automata/latest/src/regex_automata/util/interpolate.rs.html>
/// (hence the high quality doc comments).
struct ReplaceCaptureIter<'rep>(CharIndices<'rep>);

impl<'rep> ReplaceCaptureIter<'rep> {
    fn new(s: &'rep str) -> Self {
        Self(s.char_indices())
    }
}

impl<'rep> Iterator for ReplaceCaptureIter<'rep> {
    type Item = Capture<'rep>;

    fn next(&mut self) -> Option<Self::Item> {
        // Continually seek to `$` until we find one that has a capture group
        loop {
            let (start, _) = self.0.find(|(_, c)| *c == '$')?;

            let replacement = self.0.as_str();
            let rep = replacement.as_bytes();
            let open_span = Span::start_at(start + 1);
            let maybe_cap = match rep.first()? {
                // Handle escaping of '$'.
                b'$' => {
                    self.0.next().unwrap();
                    None
                }
                b'{' => find_cap_ref_braced(rep, open_span),
                _ => find_cap_ref(rep, open_span),
            };

            if let Some(cap) = maybe_cap {
                // Advance the inner iterator to consume the capture
                let mut remaining_bytes = cap.name.len();
                while remaining_bytes > 0 {
                    let (_, c) = self.0.next().unwrap();
                    remaining_bytes =
                        remaining_bytes.checked_sub(c.len_utf8()).unwrap();
                }
                return Some(cap);
            }
        }
    }
}

/// Parses a possible reference to a capture group name in the given text,
/// starting at the beginning of `replacement`.
///
/// If no such valid reference could be found, None is returned.
fn find_cap_ref(rep: &[u8], open_span: SpanOpen) -> Option<Capture<'_>> {
    if rep.is_empty() {
        return None;
    }

    let mut cap_end = 0;
    while rep.get(cap_end).copied().is_some_and(is_valid_cap_letter) {
        cap_end += 1;
    }
    if cap_end == 0 {
        return None;
    }

    // We just verified that the range 0..cap_end is valid ASCII, so it must
    // therefore be valid UTF-8. If we really cared, we could avoid this UTF-8
    // check via an unchecked conversion or by parsing the number straight from
    // &[u8].
    let name = core::str::from_utf8(&rep[..cap_end])
        .expect("valid UTF-8 capture name");
    Some(Capture::new(name, open_span.end_offset(name.len())))
}

/// Looks for a braced reference, e.g., `${foo1}`. This then looks for a
/// closing brace and returns the capture reference within the brace.
fn find_cap_ref_braced(rep: &[u8], open_span: SpanOpen) -> Option<Capture<'_>> {
    assert_eq!(b'{', rep[0]);
    let mut cap_end = 1;

    while rep.get(cap_end).is_some_and(|&b| b != b'}') {
        cap_end += 1;
    }
    if rep.get(cap_end).is_none_or(|&b| b != b'}') {
        return None;
    }

    // When looking at braced names, we don't put any restrictions on the name,
    // so it's possible it could be invalid UTF-8. But a capture group name
    // can never be invalid UTF-8, so if we have invalid UTF-8, then we can
    // safely return None.
    let name = core::str::from_utf8(&rep[..cap_end + 1]).ok()?;
    Some(Capture::new(name, open_span.end_offset(name.len())))
}

fn is_valid_cap_letter(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    #[test]
    fn literal_dollar_sign() {
        let replace = "$$0";
        let mut cap_iter = ReplaceCaptureIter::new(replace);
        assert!(cap_iter.next().is_none());
    }

    #[test]
    fn wacky_captures() {
        let replace =
            "$foo $1 $1invalid ${1}valid ${valid} $__${__weird__}${${__}";

        let cap_iter = ReplaceCaptureIter::new(replace);
        let expecteds = &[
            "foo",
            "1",
            "1invalid",
            "{1}",
            "{valid}",
            "__",
            "{__weird__}",
            "{${__}",
        ];
        for (&expected, cap) in expecteds.iter().zip(cap_iter) {
            assert_eq!(expected, cap.name, "name didn't match");
            assert_eq!(expected, cap.span.slice(replace), "span didn't match");
        }
    }

    const INTERPOLATED_CAPTURE: &str = "<interpolated>";

    fn upstream_interpolate(s: &str) -> String {
        let mut dst = String::new();
        regex_automata::util::interpolate::string(
            s,
            |_, dst| dst.push_str(INTERPOLATED_CAPTURE),
            |_| Some(0),
            &mut dst,
        );
        dst
    }

    fn our_interpolate(s: &str) -> String {
        let mut after_last_write = 0;
        let mut dst = String::new();
        for cap in ReplaceCaptureIter::new(s) {
            // This only iterates over the capture groups, so copy any text
            // before the capture
            // -1 here to exclude the `$` that starts a capture
            dst.push_str(
                &s[after_last_write..cap.span.start.checked_sub(1).unwrap()],
            );
            // Interpolate our capture
            dst.push_str(INTERPOLATED_CAPTURE);
            after_last_write = cap.span.end;
        }
        if after_last_write < s.len() {
            // And now any text that was after the last capture
            dst.push_str(&s[after_last_write..]);
        }

        // Handle escaping literal `$`s
        dst.replace("$$", "$")
    }

    proptest! {
        // `regex-automata` doesn't expose a way to iterate over replacement
        // captures, but we can use our iterator to mimic interpolation, so that
        // we can pit the two against each other
        #[test]
        fn interpolation_matches_upstream(s in r"\PC*(\$\PC*){0,5}") {
            assert_eq!(our_interpolate(&s), upstream_interpolate(&s));
        }
    }
}

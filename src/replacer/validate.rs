use std::{error::Error, fmt, str::CharIndices};

use ansi_term::{Color, Style};

#[derive(Debug)]
pub struct InvalidReplaceCapture {
    original_replace: String,
    invalid_ident: Span,
    // TODO: switch this to just num_leading_digits. Span is overkill here
    digits_prefix: Span,
}

impl Error for InvalidReplaceCapture {}

// TODO: rip out the line handling stuff and just display one line. Less
// confusing and less code
impl fmt::Display for InvalidReplaceCapture {
    // We save byte offsets for the spans, but in the unlikely event that the
    // replacement string has newlines or weird carriage returns we walk back
    // over the string to regain context
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            original_replace,
            invalid_ident,
            digits_prefix,
        } = self;

        // TODO: dedupe style code
        // Build up the error to show the user
        let mut formatted = String::new();
        let mut arrows_start = Span::start_at(0);
        let special_char_style = Style::new().bold();
        let error_style = Style::from(Color::Red).bold();
        for (byte_index, c) in original_replace.char_indices() {
            match c {
                '\n' => formatted
                    .push_str(&special_char_style.paint(r"\n").to_string()),
                '\r' => formatted
                    .push_str(&special_char_style.paint(r"\r").to_string()),
                '\t' => formatted
                    .push_str(&special_char_style.paint(r"\t").to_string()),
                other => {
                    if invalid_ident.contains(byte_index) {
                        // TODO: don't repaint on every char here
                        formatted.push_str(&format!(
                            "{}",
                            error_style.paint(String::from(other))
                        ));
                    } else {
                        formatted.push(other);
                    }
                }
            }

            if byte_index < invalid_ident.start {
                // This assumes that characters have a base display width of 1.
                // Not technically true, but hard to do right
                let width = if ['\n', '\r', '\t'].contains(&c) {
                    2
                } else {
                    1
                };
                arrows_start.start += width;
            }
        }

        // This relies on all ident chars being 1 byte
        let arrows_span = arrows_start.end_offset(invalid_ident.len());
        let mut arrows = " ".repeat(arrows_span.start);
        arrows.push_str(&format!(
            "{}",
            Style::new().bold().paint("^".repeat(arrows_span.len()))
        ));

        let ident = invalid_ident.slice(original_replace);
        let number = digits_prefix.slice(ident);
        let the_rest = &ident[digits_prefix.end..];
        let disambiguous = format!("${{{number}}}{the_rest}");
        let error_message = format!(
            "The numbered capture group `{}` in the replacement text is ambiguous.",
            Style::new().bold().paint(format!("${}", number).to_string())
        );
        let hint_message = format!(
            "{}: Use curly braces to disambiguate it `{}`.",
            Style::from(Color::Blue).bold().paint("hint"),
            Style::new().bold().paint(disambiguous)
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
                        digits_prefix: Span::new(0, i),
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

    fn contains(self, elem: usize) -> bool {
        self.start <= elem && self.end > elem
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
                b'$' => None,
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
    while rep.get(cap_end).copied().map_or(false, is_valid_cap_letter) {
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

    while rep.get(cap_end).map_or(false, |&b| b != b'}') {
        cap_end += 1;
    }
    if !rep.get(cap_end).map_or(false, |&b| b == b'}') {
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
}

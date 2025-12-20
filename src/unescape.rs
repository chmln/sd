use std::char;
use std::str::Chars;

/// Takes in a string with backslash escapes written out with literal backslash characters and
/// converts it to a string with the proper escaped characters.
pub fn unescape(input: &str) -> String {
    let mut chars = input.chars();
    let mut s = String::new();

    while let Some(c) = chars.next() {
        if c != '\\' {
            s.push(c);
            continue;
        }
        let Some(char) = chars.next() else {
            // This means that the last char is a `\\`
            assert_eq!(c, '\\');
            s.push('\\');
            break;
        };

        let escaped: Option<char> = match char {
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            '\'' => Some('\''),
            '\"' => Some('\"'),
            '\\' => Some('\\'),
            'u' => escape_n_chars(&mut chars, 4),
            'x' => escape_n_chars(&mut chars, 2),
            _ => None,
        };
        if let Some(char) = escaped {
            // Successfully escaped a sequence
            s.push(char);
        } else {
            // User didn't meant to escape that
            s.push('\\');
            s.push(char);
        }
    }

    s
}

/// This is for sequences such as `\x08` or `\u1234`
fn escape_n_chars(chars: &mut Chars<'_>, length: usize) -> Option<char> {
    let s = chars.as_str().get(0..length)?;
    let u = u32::from_str_radix(s, 16).ok()?;
    let ch = char::from_u32(u)?;
    _ = chars.nth(length);
    Some(ch)
}

#[cfg(test)]
mod test {
    use std::fmt::Write as _;

    #[test]
    fn test_unescape() {
        let mut out = String::new();
        let mut test = |s: &str, name: &str| {
            writeln!(out, "{name}: `{s}` -> `{}`", super::unescape(s)).unwrap();
        };

        test("", "empty");
        test("\\", "single backslash");
        test("\\\\", "two backslashes");
        test("\\n", "newline");
        test("\\t", "tab");
        test("\\r", "carriage return");
        test("\\\"", "escaped double quote");
        test("\\'", "escaped single quote");
        test("\\\\", "escaped backslash");
        test("\\u0042", "unicode escape");
        test("\\x41", "hex escape");
        test("\\xG", "invalid hex escape");
        test("\\u00Z1", "invalid unicode escape");
        test("a\\t\\xG\\n", "mixed valid and invalid escapes");
        test("ab", "non-escape characters");
        test("\\u004", "incomplete escape sequence");
        test("a", "single characters");
        test("\\t{", "issue #313");
        test("\\t\\{", "issue #313");

        insta::assert_snapshot!(out);
    }
}

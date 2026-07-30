use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn validate_doesnt_panic(s in r"(\PC*\$?){0,5}") {
        let _ = validate::validate_replace(&s);
    }

    // $ followed by a digit and a non-ident char or an ident char
    #[test]
    fn validate_ok(s in r"([^\$]*(\$([0-9][^a-zA-Z_0-9\$]|a-zA-Z_))?){0,5}") {
        validate::validate_replace(&s).unwrap();
    }

    // Force at least one $ followed by a digit and an ident char
    #[test]
    fn validate_err(s in r"[^\$]*?\$[0-9][a-zA-Z_]\PC*") {
        validate::validate_replace(&s).unwrap_err();
    }

    /// Where `w` sits in the flag string must not matter. The rest of the flag
    /// string keeps its order, so the genuine last-one-wins conflicts (`c` vs
    /// `i`, `e` vs `m`) resolve the same way in every variant and only `w`'s
    /// position varies.
    #[test]
    fn word_boundary_flag_position_is_irrelevant(
        flags in FLAGS,
        pattern in PATTERN,
        src in SRC,
    ) {
        let expected = replace_with_flags(&pattern, &src, &format!("w{flags}"));
        for i in 0..=flags.len() {
            let (head, tail) = flags.split_at(i);
            let actual =
                replace_with_flags(&pattern, &src, &format!("{head}w{tail}"));
            prop_assert_eq!(&expected, &actual, "flags: {}w{}", head, tail);
        }
    }

    /// `-f <flags>w` on a pattern must be equivalent to `-f <flags>` on that
    /// pattern already wrapped, i.e. `w` only ever adds the boundaries and
    /// changes nothing else.
    #[test]
    fn word_boundary_flag_only_adds_boundaries(
        flags in FLAGS,
        pattern in PATTERN,
        src in SRC,
    ) {
        let with_flag =
            replace_with_flags(&pattern, &src, &format!("{flags}w"));
        let pre_wrapped = replace_with_flags(
            &format!("\\b(?:{pattern})\\b"),
            &src,
            &flags,
        );
        prop_assert_eq!(with_flag, pre_wrapped);
    }
}

/// Every flag `w` has to compose with.
const FLAGS: &str = "[cimes]{0,4}";

/// Alternations and anchors, since a top-level `|` is what exposes a misplaced
/// `\b` and the anchors are what expose a lost `multi_line`.
const PATTERN: &str = r"(foo|bar|\w+|\^?foo|bar\$?)(\|(foo|bar|baz)){0,2}";

/// Deliberately built from affixed and cased forms of the pattern words, and
/// with newlines. Uniform `[a-z ]` junk almost never straddles a word boundary
/// or differs in case, which makes the properties above vacuous: they pass
/// against the unfixed code.
const SRC: &str = r"(foo|bar|baz|foox|xfoo|xbarx|FOO|Bar| |\n){1,6}";

/// Run a replacement with a runtime-built flag string, for the proptests above.
/// Returns `None` if the pattern fails to compile.
fn replace_with_flags(
    look_for: &str,
    src: &str,
    flags: &str,
) -> Option<String> {
    const UNLIMITED_REPLACEMENTS: usize = 0;
    let replacer = Replacer::new(
        look_for.to_owned(),
        "X".to_owned(),
        false,
        Some(flags.to_owned()),
        UNLIMITED_REPLACEMENTS,
    )
    .ok()?;

    Some(
        String::from_utf8(replacer.replace(src.as_bytes()).into_owned())
            .unwrap(),
    )
}

#[derive(Default)]
struct Replace {
    look_for: &'static str,
    replace_with: &'static str,
    literal: bool,
    flags: Option<&'static str>,
    src: &'static str,
    expected: &'static str,
}

impl Replace {
    fn test(&self) {
        const UNLIMITED_REPLACEMENTS: usize = 0;
        let replacer = Replacer::new(
            self.look_for.into(),
            self.replace_with.into(),
            self.literal,
            self.flags.map(ToOwned::to_owned),
            UNLIMITED_REPLACEMENTS,
        )
        .unwrap();

        let binding = replacer.replace(self.src.as_bytes());
        let actual = std::str::from_utf8(&binding).unwrap();

        assert_eq!(self.expected, actual);
    }
}

#[test]
fn default_global() {
    Replace {
        look_for: "a",
        replace_with: "b",
        src: "aaa",
        expected: "bbb",
        ..Default::default()
    }
    .test();
}

#[test]
fn escaped_char_preservation() {
    Replace {
        look_for: "a",
        replace_with: "b",
        src: r#"a\n"#,
        expected: r#"b\n"#,
        ..Default::default()
    }
    .test();
}

#[test]
fn case_sensitive_default() {
    Replace {
        look_for: "abc",
        replace_with: "x",
        src: "abcABC",
        expected: "xABC",
        ..Default::default()
    }
    .test();

    Replace {
        look_for: "abc",
        replace_with: "x",
        literal: true,
        src: "abcABC",
        expected: "xABC",
        ..Default::default()
    }
    .test();
}

#[test]
fn sanity_check_literal_replacements() {
    Replace {
        look_for: "((special[]))",
        replace_with: "x",
        literal: true,
        src: "((special[]))y",
        expected: "xy",
        ..Default::default()
    }
    .test();
}

#[test]
fn unescape_regex_replacements() {
    Replace {
        look_for: "test",
        replace_with: r"\n",
        src: "testtest",
        expected: "\n\n",
        ..Default::default()
    }
    .test();
}

#[test]
fn no_unescape_literal_replacements() {
    Replace {
        look_for: "test",
        replace_with: r"\n",
        literal: true,
        src: "testtest",
        expected: r"\n\n",
        ..Default::default()
    }
    .test();
}

#[test]
fn full_word_replace() {
    Replace {
        look_for: "abc",
        replace_with: "def",
        flags: Some("w"),
        src: "abcd abc",
        expected: "abcd def",
        ..Default::default()
    }
    .test();
}

#[test]
fn full_word_replace_binds_to_whole_alternation() {
    // `\b` has to apply to every alternative, not just the first and last:
    // `foo|bar` means `\b(?:foo|bar)\b`, not `\bfoo|bar\b`.
    Replace {
        look_for: "foo|bar",
        replace_with: "X",
        flags: Some("w"),
        src: "foox bar",
        expected: "foox X",
        ..Default::default()
    }
    .test();

    Replace {
        look_for: "foo|bar",
        replace_with: "X",
        flags: Some("w"),
        src: "foo barx",
        expected: "X barx",
        ..Default::default()
    }
    .test();

    Replace {
        look_for: "foo|bar",
        replace_with: "X",
        flags: Some("w"),
        src: "foo bar",
        expected: "X X",
        ..Default::default()
    }
    .test();
}

#[test]
fn full_word_replace_does_not_shift_capture_groups() {
    // The wrapping group must be non-capturing, or the user's `$1` becomes `$2`.
    Replace {
        look_for: "(f)(o)o",
        replace_with: "$2$1",
        flags: Some("w"),
        src: "foo",
        expected: "of",
        ..Default::default()
    }
    .test();
}

#[test]
fn full_word_replace_is_case_sensitive() {
    Replace {
        look_for: "abc",
        replace_with: "def",
        flags: Some("w"),
        src: "ABC abc",
        expected: "ABC def",
        ..Default::default()
    }
    .test();
}

#[test]
fn full_word_replace_keeps_flags_regardless_of_order() {
    for flags in ["iw", "wi"] {
        Replace {
            look_for: "abc",
            replace_with: "def",
            flags: Some(flags),
            src: "ABCd ABC",
            expected: "ABCd def",
            ..Default::default()
        }
        .test();
    }

    for flags in ["sw", "ws"] {
        Replace {
            look_for: "a.b",
            replace_with: "X",
            flags: Some(flags),
            src: "a\nb",
            expected: "X",
            ..Default::default()
        }
        .test();
    }
}

#[test]
fn full_word_replace_keeps_multi_line_anchors() {
    for flags in ["w", "wm", "mw"] {
        Replace {
            look_for: r"^\w+",
            replace_with: "M",
            flags: Some(flags),
            src: "aa\nbb",
            expected: "M\nM",
            ..Default::default()
        }
        .test();

        Replace {
            look_for: r"\w+$",
            replace_with: "M",
            flags: Some(flags),
            src: "aa\nbb",
            expected: "M\nM",
            ..Default::default()
        }
        .test();
    }
}

#[test]
fn escaping_unnecessarily() {
    // https://github.com/chmln/sd/issues/313
    Replace {
        look_for: "abc",
        replace_with: r#"\n{"#,
        src: "abc",
        expected: "\n{",
        ..Default::default()
    }
    .test();

    Replace {
        look_for: "abc",
        replace_with: r#"\n\{"#,
        src: "abc",
        expected: "\n\\{",
        ..Default::default()
    }
    .test();
}

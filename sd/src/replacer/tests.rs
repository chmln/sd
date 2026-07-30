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
fn multi_line_is_the_default() {
    for flags in [None, Some("m")] {
        Replace {
            look_for: r"^\w+",
            replace_with: "M",
            flags,
            src: "aa\nbb",
            expected: "M\nM",
            ..Default::default()
        }
        .test();
    }
}

#[test]
fn disabling_multi_line() {
    Replace {
        look_for: r"^\w+",
        replace_with: "M",
        flags: Some("e"),
        src: "aa\nbb",
        expected: "M\nbb",
        ..Default::default()
    }
    .test();
}

#[test]
fn multi_line_flag_can_undo_a_preceding_e() {
    // `e` then `m` reads as "off, then on", so it has to end up on. `m` being a
    // no-op made an `e` permanent for the rest of the flag string.
    Replace {
        look_for: r"^\w+",
        replace_with: "M",
        flags: Some("em"),
        src: "aa\nbb",
        expected: "M\nM",
        ..Default::default()
    }
    .test();

    // ...and the reverse order still ends up off, since the last one wins.
    Replace {
        look_for: r"^\w+",
        replace_with: "M",
        flags: Some("me"),
        src: "aa\nbb",
        expected: "M\nbb",
        ..Default::default()
    }
    .test();
}

#[test]
fn dot_matches_newline_flag_leaves_anchors_alone() {
    // `s` is documented as only affecting `.`. It used to disable multi-line as
    // an undocumented side effect, so `^` stopped matching at line starts.
    for flags in ["s", "sm", "ms"] {
        Replace {
            look_for: r"^\w+",
            replace_with: "M",
            flags: Some(flags),
            src: "aa\nbb",
            expected: "M\nM",
            ..Default::default()
        }
        .test();
    }

    // What `s` is actually for, unaffected by the above.
    for flags in ["s", "sm", "ms"] {
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

    // `.` must still not match a newline without `s`.
    Replace {
        look_for: "a.b",
        replace_with: "X",
        src: "a\nb",
        expected: "a\nb",
        ..Default::default()
    }
    .test();
}

#[test]
fn dot_matches_newline_composes_with_disabled_multi_line() {
    // Both at once has to be expressible: `.` matching newlines while `^`/`$`
    // anchor only to the whole input. Coupling the two made this impossible.
    Replace {
        look_for: r"^a.b",
        replace_with: "X",
        flags: Some("es"),
        src: "a\nb\na\nb",
        expected: "X\na\nb",
        ..Default::default()
    }
    .test();
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

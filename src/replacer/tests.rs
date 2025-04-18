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

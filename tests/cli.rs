#[cfg(test)]
#[cfg(not(sd_cross_compile))] // Cross-compilation does not allow to spawn threads but `command.assert()` would do.
mod cli {
    use anyhow::Result;
    use assert_cmd::Command;
    use std::io::prelude::*;

    fn sd() -> Command {
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Error invoking sd")
    }

    fn assert_file(path: &std::path::Path, content: &str) {
        assert_eq!(content, std::fs::read_to_string(path).unwrap());
    }

    fn create_soft_link<P: AsRef<std::path::Path>>(
        src: &P,
        dst: &P,
    ) -> Result<()> {
        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(src, dst)?;
        #[cfg(target_family = "windows")]
        std::os::windows::fs::symlink_file(src, dst)?;

        Ok(())
    }

    #[test]
    fn in_place() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"abc123def")?;
        let path = file.into_temp_path();

        sd().args(["abc\\d+", "", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path, "def");

        Ok(())
    }

    #[test]
    fn in_place_with_empty_result_file() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"a7c")?;
        let path = file.into_temp_path();

        sd().args(["a\\dc", "", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path, "");

        Ok(())
    }

    #[test]
    fn in_place_following_symlink() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path();
        let file = path.join("file");
        let link = path.join("link");

        create_soft_link(&file, &link)?;
        std::fs::write(&file, "abc123def")?;

        sd().args(["abc\\d+", "", link.to_str().unwrap()])
            .assert()
            .success();

        assert_file(&file, "def");
        assert!(std::fs::symlink_metadata(link)?.file_type().is_symlink());

        Ok(())
    }

    #[test]
    fn replace_into_stdout() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"abc123def")?;

        sd().args(["-p", "abc\\d+", "", file.path().to_str().unwrap()])
            .assert()
            .success()
            .stdout(format!(
                "{}{}def\n",
                ansi_term::Color::Green.prefix(),
                ansi_term::Color::Green.suffix()
            ));

        assert_file(file.path(), "abc123def");

        Ok(())
    }

    #[test]
    fn stdin() -> Result<()> {
        sd().args(["abc\\d+", ""])
            .write_stdin("abc123def")
            .assert()
            .success()
            .stdout("def");

        Ok(())
    }

    fn bad_replace_helper_styled(replace: &str) -> String {
        let err = sd()
            .args(["find", replace])
            .write_stdin("stdin")
            .unwrap_err();
        String::from_utf8(err.as_output().unwrap().stderr.clone()).unwrap()
    }

    fn bad_replace_helper_plain(replace: &str) -> String {
        let stderr = bad_replace_helper_styled(replace);

        // TODO: no easy way to toggle off styling yet. Add a `--color <when>`
        // flag, and respect things like `$NO_COLOR`. `ansi_term` is
        // unmaintained, so we should migrate off of it anyways
        console::AnsiCodeIterator::new(&stderr)
            .filter_map(|(s, is_ansi)| (!is_ansi).then_some(s))
            .collect()
    }

    #[test]
    fn fixed_strings_ambiguous_replace_is_fine() {
        sd().args([
            "--fixed-strings",
            "foo",
            "inner_before $1fine inner_after",
        ])
        .write_stdin("outer_before foo outer_after")
        .assert()
        .success()
        .stdout("outer_before inner_before $1fine inner_after outer_after");
    }

    #[test]
    fn ambiguous_replace_basic() {
        let plain_stderr = bad_replace_helper_plain("before $1bad after");
        insta::assert_snapshot!(plain_stderr, @r###"
        error: The numbered capture group `$1` in the replacement text is ambiguous.
        hint: Use curly braces to disambiguate it `${1}bad`.
        before $1bad after
                ^^^^
        "###);
    }

    #[test]
    fn ambiguous_replace_variable_width() {
        let plain_stderr = bad_replace_helper_plain("\r\n\t$1bad\r");
        insta::assert_snapshot!(plain_stderr, @r###"
        error: The numbered capture group `$1` in the replacement text is ambiguous.
        hint: Use curly braces to disambiguate it `${1}bad`.
        ␍␊␉$1bad␍
            ^^^^
        "###);
    }

    #[test]
    fn ambiguous_replace_multibyte_char() {
        let plain_stderr = bad_replace_helper_plain("😈$1bad😇");
        insta::assert_snapshot!(plain_stderr, @r###"
        error: The numbered capture group `$1` in the replacement text is ambiguous.
        hint: Use curly braces to disambiguate it `${1}bad`.
        😈$1bad😇
          ^^^^
        "###);
    }

    #[test]
    fn ambiguous_replace_issue_44() {
        let plain_stderr =
            bad_replace_helper_plain("$1Call $2($5, GetFM20ReturnKey(), $6)");
        insta::assert_snapshot!(plain_stderr, @r###"
        error: The numbered capture group `$1` in the replacement text is ambiguous.
        hint: Use curly braces to disambiguate it `${1}Call`.
        $1Call $2($5, GetFM20ReturnKey(), $6)
         ^^^^^
        "###);
    }

    // NOTE: styled terminal output is platform dependent, so convert to a
    // common format, in this case HTML, to check
    #[test]
    fn ambiguous_replace_ensure_styling() {
        let styled_stderr = bad_replace_helper_styled("\t$1bad after");
        let html_stderr =
            ansi_to_html::convert(&styled_stderr, true, true).unwrap();
        insta::assert_snapshot!(html_stderr, @r###"
        <b><span style='color:#a00'>error</span></b>: The numbered capture group `<b>$1</b>` in the replacement text is ambiguous.
        <b><span style='color:#00a'>hint</span></b>: Use curly braces to disambiguate it `<b>${1}bad</b>`.
        <b>␉</b>$<b><span style='color:#a00'>1bad</span></b> after
          <b>^^^^</b>
        "###);
    }

    #[test]
    fn limit_replacements_file() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"foo\nfoo\nfoo")?;
        let path = file.into_temp_path();

        sd().args(["-n", "1", "foo", "bar", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path, "bar\nfoo\nfoo");

        Ok(())
    }

    #[test]
    fn limit_replacements_file_preview() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"foo\nfoo\nfoo")?;
        let path = file.into_temp_path();

        sd().args([
            "--preview",
            "-n",
            "1",
            "foo",
            "bar",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(format!(
            "{}\nfoo\nfoo",
            ansi_term::Color::Green.paint("bar")
        ));

        Ok(())
    }

    #[test]
    fn limit_replacements_stdin() {
        sd().args(["-n", "1", "foo", "bar"])
            .write_stdin("foo\nfoo\nfoo")
            .assert()
            .success()
            .stdout("bar\nfoo\nfoo");
    }

    #[test]
    fn limit_replacements_stdin_preview() {
        sd().args(["--preview", "-n", "1", "foo", "bar"])
            .write_stdin("foo\nfoo\nfoo")
            .assert()
            .success()
            .stdout("bar\nfoo\nfoo");
    }
}

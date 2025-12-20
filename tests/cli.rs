#[cfg(test)]
#[cfg(not(sd_cross_compile))] // Cross-compilation does not allow to spawn threads but `command.assert()` would do.
mod cli {
    use anyhow::Result;
    use assert_cmd::{Command, cargo_bin};
    use std::{fs, io::prelude::*, path::Path};

    fn sd() -> Command {
        Command::new(cargo_bin!(env!("CARGO_PKG_NAME")))
    }

    fn assert_file(path: &std::path::Path, content: &str) {
        assert_eq!(content, std::fs::read_to_string(path).unwrap());
    }

    // This should really be cfg_attr(target_family = "windows"), but wasi impl
    // is nightly for now, and other impls are not part of std
    #[cfg_attr(
        not(target_family = "unix"),
        ignore = "Windows symlinks are privileged"
    )]
    fn create_soft_link<P: AsRef<std::path::Path>>(
        src: &P,
        dst: &P,
    ) -> Result<()> {
        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(src, dst)?;

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

    #[cfg_attr(
        target_family = "windows",
        ignore = "Windows symlinks are privileged"
    )]
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
            .stdout("def");

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
        stderr
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
    #[ignore = "TODO: wait for proper colorization"]
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
        .stdout("bar\nfoo\nfoo");

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

    const UNTOUCHED_CONTENTS: &str = "untouched";

    fn assert_fails_correctly(
        command: &mut Command,
        valid: &Path,
        test_home: &Path,
        snap_name: &str,
    ) {
        let failed_command = command.assert().failure().code(1);

        assert_eq!(fs::read_to_string(&valid).unwrap(), UNTOUCHED_CONTENTS);

        let stderr_orig =
            std::str::from_utf8(&failed_command.get_output().stderr).unwrap();
        // Normalize unstable path bits
        let stderr_norm = stderr_orig
            .replace(test_home.to_str().unwrap(), "<test_home>")
            .replace('\\', "/");
        insta::assert_snapshot!(snap_name, stderr_norm);
    }

    #[test]
    fn correctly_fails_on_missing_file() -> Result<()> {
        let test_dir = tempfile::Builder::new().prefix("sd-test-").tempdir()?;
        let test_home = test_dir.path();

        let valid = test_home.join("valid");
        fs::write(&valid, UNTOUCHED_CONTENTS)?;
        let missing = test_home.join("missing");

        assert_fails_correctly(
            sd().args([".*", ""]).arg(&valid).arg(&missing),
            &valid,
            test_home,
            "correctly_fails_on_missing_file",
        );

        Ok(())
    }

    #[cfg_attr(not(target_family = "unix"), ignore = "only runs on unix")]
    #[test]
    fn correctly_fails_on_unreadable_file() -> Result<()> {
        #[cfg(not(target_family = "unix"))]
        {
            unreachable!("This test should be ignored");
        }
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::OpenOptionsExt;

            let test_dir =
                tempfile::Builder::new().prefix("sd-test-").tempdir()?;
            let test_home = test_dir.path();

            let valid = test_home.join("valid");
            fs::write(&valid, UNTOUCHED_CONTENTS)?;
            let write_only = {
                let path = test_home.join("write_only");
                let mut write_only_file = std::fs::OpenOptions::new()
                    .mode(0o333)
                    .create(true)
                    .write(true)
                    .open(&path)?;
                write!(write_only_file, "unreadable")?;
                path
            };

            assert_fails_correctly(
                sd().args([".*", ""]).arg(&valid).arg(&write_only),
                &valid,
                test_home,
                "correctly_fails_on_unreadable_file",
            );

            Ok(())
        }
    }

    // Failing to create a temporary file in the same directory as the input is
    // one of the failure cases that is past the "point of no return" (after we
    // already start making replacements). This means that any files that could
    // be modified are, and we report any failure cases
    #[cfg_attr(not(target_family = "unix"), ignore = "only runs on unix")]
    #[test]
    fn reports_errors_on_atomic_file_swap_creation_failure() -> Result<()> {
        #[cfg(not(target_family = "unix"))]
        {
            unreachable!("This test should be ignored");
        }
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;

            const FIND_REPLACE: [&str; 2] = ["able", "ed"];
            const ORIG_TEXT: &str = "modifiable";
            const MODIFIED_TEXT: &str = "modified";

            let test_dir =
                tempfile::Builder::new().prefix("sd-test-").tempdir()?;
            let test_home = test_dir.path().canonicalize()?;

            let writable_dir = test_home.join("writable");
            fs::create_dir(&writable_dir)?;
            let writable_dir_file = writable_dir.join("foo");
            fs::write(&writable_dir_file, ORIG_TEXT)?;

            let unwritable_dir = test_home.join("unwritable");
            fs::create_dir(&unwritable_dir)?;
            let unwritable_dir_file1 = unwritable_dir.join("bar");
            fs::write(&unwritable_dir_file1, ORIG_TEXT)?;
            let unwritable_dir_file2 = unwritable_dir.join("baz");
            fs::write(&unwritable_dir_file2, ORIG_TEXT)?;
            let mut perms = fs::metadata(&unwritable_dir)?.permissions();
            perms.set_mode(0o555);
            fs::set_permissions(&unwritable_dir, perms)?;

            let failed_command = sd()
                .args(FIND_REPLACE)
                .arg(&writable_dir_file)
                .arg(&unwritable_dir_file1)
                .arg(&unwritable_dir_file2)
                .assert()
                .failure()
                .code(1);

            // Confirm that we modified the one file that we were able to
            assert_eq!(fs::read_to_string(&writable_dir_file)?, MODIFIED_TEXT);
            assert_eq!(fs::read_to_string(&unwritable_dir_file1)?, ORIG_TEXT);
            assert_eq!(fs::read_to_string(&unwritable_dir_file2)?, ORIG_TEXT);

            let stderr_orig =
                std::str::from_utf8(&failed_command.get_output().stderr)
                    .unwrap();
            // Normalize unstable path bits
            let stderr_partial_norm = stderr_orig
                .replace(test_home.to_str().unwrap(), "<test_home>")
                .replace('\\', "/");
            let tmp_file_rep = regex::Regex::new(r"\.tmp\w+")?;
            let stderr_norm =
                tmp_file_rep.replace_all(&stderr_partial_norm, "<tmp_file>");
            insta::assert_snapshot!(stderr_norm);

            // Make the unwritable dir writable again, so it can be cleaned up
            // when dropping the temp dir
            let mut perms = fs::metadata(&unwritable_dir)?.permissions();
            perms.set_mode(0o777);
            fs::set_permissions(&unwritable_dir, perms)?;
            test_dir.close()?;

            Ok(())
        }
    }
}

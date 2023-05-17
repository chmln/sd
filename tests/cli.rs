#[cfg(test)]
#[cfg(not(sd_cross_compile))] // Cross-compilation does not allow to spawn threads but `command.assert()` would do.
mod cli {
    use anyhow::Result;
    use assert_cmd::Command;
    use rstest::rstest;
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

    #[rstest]
    fn in_place(#[values(false, true)] no_swap: bool) -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"abc123def")?;
        let path = file.into_temp_path();

        let mut cmd = sd();
        cmd.args(["abc\\d+", "", path.to_str().unwrap()]);
        if no_swap {
            cmd.arg("--no-swap");
        }
        cmd.assert().success();
        assert_file(&path, "def");

        Ok(())
    }

    #[rstest]
    fn in_place_with_empty_result_file(
        #[values(false, true)] no_swap: bool,
    ) -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"a7c")?;
        let path = file.into_temp_path();

        let mut cmd = sd();
        cmd.args(["a\\dc", "", path.to_str().unwrap()]);
        if no_swap {
            cmd.arg("--no-swap");
        }
        cmd.assert().success();
        assert_file(&path, "");

        Ok(())
    }

    #[rstest]
    fn in_place_following_symlink(
        #[values(false, true)] no_swap: bool,
    ) -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path();
        let file = path.join("file");
        let link = path.join("link");

        create_soft_link(&file, &link)?;
        std::fs::write(&file, "abc123def")?;

        let mut cmd = sd();
        cmd.args(["abc\\d+", "", link.to_str().unwrap()]);
        if no_swap {
            cmd.arg("--no-swap");
        }
        cmd.assert().success();

        assert_file(&file, "def");
        assert!(std::fs::symlink_metadata(link)?.file_type().is_symlink());

        Ok(())
    }

    #[rstest]
    fn replace_into_stdout(#[values(false, true)] no_swap: bool) -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"abc123def")?;

        let mut cmd = sd();
        cmd.args(["-p", "abc\\d+", "", file.path().to_str().unwrap()]);
        if no_swap {
            cmd.arg("--no-swap");
        }
        cmd.assert().success().stdout(format!(
            "{}{}def\n",
            ansi_term::Color::Green.prefix(),
            ansi_term::Color::Green.suffix()
        ));

        assert_file(file.path(), "abc123def");

        Ok(())
    }

    #[rstest]
    fn stdin(#[values(false, true)] no_swap: bool) -> Result<()> {
        let mut cmd = sd();
        cmd.args(["abc\\d+", ""]);
        if no_swap {
            cmd.arg("--no-swap");
        }
        cmd.write_stdin("abc123def")
            .assert()
            .success()
            .stdout("def");

        Ok(())
    }
}

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
        file.write(b"abc123def")?;
        let path = file.into_temp_path();

        sd().args(&["abc\\d+", "", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path.to_path_buf(), "def");

        Ok(())
    }

    #[test]
    fn in_place_with_empty_result_file() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write(b"a7c")?;
        let path = file.into_temp_path();

        sd().args(&["a\\dc", "", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path.to_path_buf(), "");

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

        sd().args(&["abc\\d+", "", link.to_str().unwrap()])
            .assert()
            .success();

        assert_file(&file.to_path_buf(), "def");
        assert!(std::fs::symlink_metadata(link)?.file_type().is_symlink());

        Ok(())
    }

    #[test]
    fn replace_into_stdout() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write(b"abc123def")?;

        sd().args(&["-p", "abc\\d+", "", file.path().to_str().unwrap()])
            .assert()
            .success()
            .stdout(format!(
                "{}{}def\n",
                ansi_term::Color::Green.prefix().to_string(),
                ansi_term::Color::Green.suffix().to_string()
            ));

        assert_file(file.path(), "abc123def");

        Ok(())
    }

    #[test]
    fn stdin() -> Result<()> {
        sd().args(&["abc\\d+", ""])
            .write_stdin("abc123def")
            .assert()
            .success()
            .stdout("def");

        Ok(())
    }
}

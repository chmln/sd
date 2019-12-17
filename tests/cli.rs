use anyhow::Result;
use assert_cmd::Command;
use std::io::prelude::*;

fn sd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Error invoking sd")
}

fn assert_file(path: &std::path::Path, content: &str) {
    assert_eq!(content, std::fs::read_to_string(path).unwrap());
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
fn replace_into_stdout() -> Result<()> {
    let mut file = tempfile::NamedTempFile::new()?;
    file.write(b"abc123def")?;

    #[rustfmt::skip]
    sd()
        .args(&["-p", "abc\\d+", "", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("def");

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

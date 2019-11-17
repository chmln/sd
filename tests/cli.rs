use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn sd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Error invoking sd")
}

#[test]
fn in_place() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("test")?;
    file.write_str("abc123def")?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(&["abc\\d+", "", file.path().to_str().unwrap()]);
    cmd.assert().success();
    file.assert("def");

    Ok(())
}

#[test]
fn replace_into_stdout() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("test")?;
    file.write_str("abc123def")?;

    #[rustfmt::skip]
    sd()
        .args(&["-p", "abc\\d+", "", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("def");

    file.assert("abc123def");

    Ok(())
}

#[test]
fn stdin() -> Result<()> {
    sd().args(&["abc\\d+", ""])
        .with_stdin()
        .buffer("abc123def")
        .assert()
        .success()
        .stdout("def");

    Ok(())
}

use anyhow::Result;
use assert_cmd::{Command, pkg_name};

#[test]
fn execute_success_without_dir() -> Result<()> {
    Command::cargo_bin(pkg_name!())?.assert().success();
    Ok(())
}

#[test]
fn execute_success_with_dir() -> Result<()> {
    Command::cargo_bin(pkg_name!())?.arg(".").assert().success();
    Ok(())
}

#[test]
fn execute_failure_with_dir() -> Result<()> {
    Command::cargo_bin(pkg_name!())?
        .arg("non-existent")
        .assert()
        .failure()
        .code(1);
    Ok(())
}

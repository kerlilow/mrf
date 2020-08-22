use std::error::Error;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn mv_simple() -> Result<(), Box<dyn Error>> {
    let temp = assert_fs::TempDir::new()?;
    temp.child("test-001").touch()?;

    let mut cmd = Command::cargo_bin("mrf")?;
    cmd.current_dir(temp.path())
        .arg("mv")
        .arg("-y")
        .arg("test-001")
        .arg("{}{=_}{}");
    cmd.assert().success();

    temp.child("test-001").assert(predicate::path::missing());
    temp.child("test_001").assert(predicate::path::exists());

    Ok(())
}

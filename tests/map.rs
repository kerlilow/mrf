use std::error::Error;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn map_simple() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("mrf")?;
    cmd.arg("map").arg("test-001").arg("{}{=_}{}");
    cmd.assert()
        .success()
        .stdout(predicate::eq("test-001\0test_001\0"));
    Ok(())
}

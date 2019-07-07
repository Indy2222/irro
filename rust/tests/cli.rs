use assert_cmd::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn test_update_command() {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(6).collect();
    let path = format!("/tmp/irro-cli-test-{}", rand_string);

    let mut cmd = Command::cargo_bin("irro-cli").unwrap();
    let output = cmd.arg("update").arg("--path").arg(&path).output().unwrap();
    assert!(output.status.success());

    let metadata = fs::metadata(&path).unwrap();
    // TODO: This test should be improved beyion plain "is larger than MiB".
    // Possible direction: GCS XML API (the file is downloaded from GCS)
    // exposes MD5 ETags.
    assert!(metadata.len() > 1024 * 1024);
    let mode = metadata.permissions().mode();
    let exec = mode & 0o100;
    assert!(exec == 64);
}

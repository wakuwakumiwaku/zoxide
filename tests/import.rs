#![cfg(unix)]

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;

#[test]
fn failed_atuin_export_does_not_save_partial_import() {
    let fake_bin = tempfile::tempdir().unwrap();
    let atuin = fake_bin.path().join("atuin");
    fs::write(&atuin, "#!/bin/sh\nprintf '2024-01-02 03:04:05\\t/tmp\\0'\nexit 7\n").unwrap();
    fs::set_permissions(&atuin, fs::Permissions::from_mode(0o755)).unwrap();

    let data_dir = tempfile::tempdir().unwrap();
    let path = format!("{}:{}", fake_bin.path().display(), env::var("PATH").unwrap());
    let output = Command::cargo_bin("zoxide")
        .unwrap()
        .args(["import", "atuin"])
        .env("_ZO_DATA_DIR", data_dir.path())
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("atuin exited with status"));
    assert!(!data_dir.path().join("db.zo").exists());
}

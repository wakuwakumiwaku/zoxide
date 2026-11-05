#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;

fn zoxide(data: &Path, resolve: bool) -> Command {
    let mut cmd = cargo_bin_cmd!();
    cmd.env("_ZO_DATA_DIR", data)
        .env("_ZO_EXCLUDE_DIRS", "")
        .env("_ZO_MAXAGE", "10000")
        .env("_ZO_RESOLVE_SYMLINKS", if resolve { "1" } else { "0" });
    cmd
}

fn remove_alias(relative: bool) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let data = root.join("data");
    let target = root.join("target");
    let alias = root.join("alias");
    let keep = root.join("keep");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&keep).unwrap();
    symlink(&target, &alias).unwrap();
    zoxide(&data, true).arg("add").args([&alias, &keep]).assert().success();
    let mut cmd = zoxide(&data, true);
    cmd.current_dir(&root).arg("remove");
    cmd.arg(if relative { Path::new("./alias") } else { &alias });
    cmd.assert().success().stdout("").stderr("");
    zoxide(&data, true)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", keep.display()));
}

#[test]
fn remove_absolute_alias_resolves_the_path_used_by_add() {
    remove_alias(false);
}

#[test]
fn remove_relative_alias_resolves_the_path_used_by_add() {
    remove_alias(true);
}

#[test]
fn disabled_resolution_does_not_remove_a_different_stored_path() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("data");
    let target = temp.path().join("target");
    let alias = temp.path().join("alias");
    fs::create_dir(&target).unwrap();
    symlink(&target, &alias).unwrap();
    zoxide(&data, false).arg("add").arg(&target).assert().success();
    let before = fs::read(data.join("db.zo")).unwrap();
    zoxide(&data, false).arg("remove").arg(&alias).assert().failure();
    assert_eq!(fs::read(data.join("db.zo")).unwrap(), before);
    zoxide(&data, false).arg("add").arg(&alias).assert().success();
    zoxide(&data, false).arg("remove").arg(&alias).assert().success();
    zoxide(&data, false)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", target.display()));
}

#[test]
fn relative_deleted_path_can_still_be_removed_with_resolution_enabled() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("data");
    let gone = temp.path().join("gone");
    fs::create_dir(&gone).unwrap();
    zoxide(&data, true).arg("add").arg(&gone).assert().success();
    fs::remove_dir(&gone).unwrap();
    zoxide(&data, true).current_dir(temp.path()).args(["remove", "./gone"]).assert().success();
    zoxide(&data, true).args(["query", "--all", "--list"]).assert().success().stdout("");
}

#[test]
fn exact_stored_alias_wins_even_when_resolution_is_enabled_later() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("data");
    let target = temp.path().join("target");
    let alias = temp.path().join("alias");
    fs::create_dir(&target).unwrap();
    symlink(&target, &alias).unwrap();
    zoxide(&data, false).arg("add").args([&alias, &target]).assert().success();
    zoxide(&data, true).arg("remove").arg(&alias).assert().success();
    zoxide(&data, true)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", target.display()));
}

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use rstest::rstest;

fn zoxide(data_dir: &Path) -> Command {
    let mut cmd = cargo_bin_cmd!();
    cmd.env("_ZO_DATA_DIR", data_dir)
        .env("_ZO_EXCLUDE_DIRS", "")
        .env("_ZO_MAXAGE", "10000")
        .env("_ZO_RESOLVE_SYMLINKS", "0");
    cmd
}

#[rstest]
#[case("NaN")]
#[case("inf")]
#[case("-inf")]
#[case("1e309")]
#[case("-1e309")]
fn non_finite_score_does_not_modify_database(#[case] score: &str) {
    let tempdir = tempfile::tempdir().unwrap();
    let data_dir = tempdir.path();
    let existing_dir = data_dir.join("existing");
    let new_dir = data_dir.join("new");
    fs::create_dir(&existing_dir).unwrap();
    fs::create_dir(&new_dir).unwrap();

    zoxide(data_dir).arg("add").arg(&existing_dir).assert().success();
    let db_path = data_dir.join("db.zo");
    let db_before = fs::read(&db_path).unwrap();

    zoxide(data_dir)
        .arg("add")
        .arg(format!("--score={score}"))
        .arg(&new_dir)
        .assert()
        .failure()
        .stdout("")
        .stderr("zoxide: score must be finite\n");

    assert_eq!(fs::read(&db_path).unwrap(), db_before);
    zoxide(data_dir)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", existing_dir.display()));
}

#[rstest]
#[case("0")]
#[case("0.5")]
#[case("-0.5")]
fn finite_scores_are_accepted(#[case] score: &str) {
    let tempdir = tempfile::tempdir().unwrap();
    let data_dir = tempdir.path();
    let dir = data_dir.join("dir");
    fs::create_dir(&dir).unwrap();

    zoxide(data_dir).arg("add").arg(&dir).assert().success();
    zoxide(data_dir)
        .arg("add")
        .arg(format!("--score={score}"))
        .arg(&dir)
        .assert()
        .success()
        .stdout("")
        .stderr("");

    zoxide(data_dir)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", dir.display()));
}

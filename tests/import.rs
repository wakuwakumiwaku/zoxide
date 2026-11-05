use std::fs;
use std::path::Path;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use rstest::rstest;

fn zoxide(data: &Path) -> Command {
    let mut cmd = cargo_bin_cmd!();
    cmd.env("_ZO_DATA_DIR", data)
        .env("_ZO_EXCLUDE_DIRS", "")
        .env("_ZO_MAXAGE", "10000")
        .env("_ZO_RESOLVE_SYMLINKS", "0");
    cmd
}

fn source(data: &Path, importer: &str, contents: &str) -> (String, std::path::PathBuf) {
    let (env, path) = match importer {
        "z" => ("_Z_DATA", data.join("z.txt")),
        "fasd" => ("_FASD_DATA", data.join("fasd.txt")),
        "z.lua" => ("_ZL_DATA", data.join("zlua.txt")),
        "zsh-z" => ("ZSHZ_DATA", data.join("zsh-z.txt")),
        _ => panic!("unexpected importer"),
    };
    fs::write(&path, contents).unwrap();
    (env.into(), path)
}

#[rstest]
#[case("z")]
#[case("fasd")]
#[case("z.lua")]
#[case("zsh-z")]
fn non_finite_import_rows_are_skipped_without_poisoning_valid_rows(#[case] importer: &str) {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path();
    let valid = data.join(if cfg!(windows) { "valid" } else { "valid|with separator" });
    let invalid = data.join("invalid");
    fs::create_dir(&valid).unwrap();
    fs::create_dir(&invalid).unwrap();
    let input = format!(
        "{}|NaN|1\n{}|inf|1\n{}|-inf|1\n{}|1e309|1\n{}|4.5|1\n",
        invalid.display(),
        invalid.display(),
        invalid.display(),
        invalid.display(),
        valid.display()
    );
    let (env, path) = source(data, importer, &input);
    let result = zoxide(data)
        .args(["import", importer])
        .env(&env, &path)
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 4, "{stderr}");
    for line in 1..=4 {
        assert!(stderr.contains(&format!("{}:{line}:", path.display())), "{stderr}");
    }
    zoxide(data)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", valid.display()));
}

#[test]
fn invalid_only_import_preserves_existing_database_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path();
    let existing = data.join("existing");
    fs::create_dir(&existing).unwrap();
    zoxide(data).arg("add").arg(&existing).assert().success();
    let before = fs::read(data.join("db.zo")).unwrap();
    let (env, path) = source(data, "z", "/synthetic/invalid|NaN|1\n");
    let result = zoxide(data)
        .args(["import", "--merge", "z"])
        .env(env, path)
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(!result.stderr.is_empty(), "invalid rank must be diagnosed");
    assert_eq!(fs::read(data.join("db.zo")).unwrap(), before);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[test]
fn autojump_rejects_non_finite_source_ranks_before_normalization() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path();
    let valid = data.join("valid");
    fs::create_dir(&valid).unwrap();
    let input_dir = data.join("autojump");
    fs::create_dir(&input_dir).unwrap();
    fs::write(
        input_dir.join("autojump.txt"),
        format!("NaN\t/bad-nan\ninf\t/bad-inf\n-inf\t/bad-neg-inf\n0\t{}\n", valid.display()),
    )
    .unwrap();
    let result = zoxide(data)
        .args(["import", "autojump"])
        .env("XDG_DATA_HOME", data)
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 3, "{stderr}");
    zoxide(data)
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", valid.display()));
}

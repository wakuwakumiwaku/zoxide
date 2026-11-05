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

#[rstest]
#[case("1e308", 2)]
#[case("1.7976931348623157e308", 3)]
#[case("5000", 3)]
fn aging_large_finite_totals_preserves_directory_weights(
    #[case] score: &str,
    #[case] count: usize,
) {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path();
    let paths: Vec<_> = (0..count).map(|n| data.join(format!("directory-{n}"))).collect();
    for path in &paths {
        fs::create_dir(path).unwrap();
    }
    zoxide(data).arg("add").arg(format!("--score={score}")).args(&paths).assert().success();
    let output = zoxide(data)
        .args(["query", "--all", "--list", "--score"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), count, "aging must not erase the database: {output}");
    for path in &paths {
        assert!(lines.iter().any(|line| line.ends_with(path.to_str().unwrap())), "{output}");
    }
    assert!(!output.contains("NaN"), "{output}");
}

#[test]
fn overflowing_total_still_preserves_relative_ranks() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path();
    let source = data.join("z.txt");
    let heavy = data.join("heavy");
    let light = data.join("light");
    for path in [&heavy, &light] {
        fs::create_dir(path).unwrap();
    }
    fs::write(&source, format!("{}|1.6e308|1\n{}|8e307|1\n", heavy.display(), light.display()))
        .unwrap();
    zoxide(data).args(["import", "z"]).env("_Z_DATA", source).assert().success();
    let result = zoxide(data)
        .args(["query", "--all", "--list", "--score"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(result).unwrap();
    let scores: Vec<f64> =
        text.lines().map(|line| line.split_whitespace().next().unwrap().parse().unwrap()).collect();
    assert_eq!(scores.len(), 2, "{text}");
    // Both timestamps are old: scores are one quarter of normalized ranks.
    assert!((scores[0] - 1500.0).abs() < 0.1, "{text}");
    assert!((scores[1] - 750.0).abs() < 0.1, "{text}");
}

use std::path::PathBuf;
use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn config_path() -> String {
    project_root()
        .join("training_max.toml")
        .to_string_lossy()
        .into_owned()
}

fn run_cli_with_seed(args: &[&str]) -> String {
    let exe = env::var("CARGO_BIN_EXE_five_three_one")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let debug_exe = Path::new(&project_root()).join("target").join("debug").join("five-three-one");
            let release_exe = Path::new(&project_root())
                .join("target")
                .join("release")
                .join("five-three-one");

            if fs::metadata(&debug_exe).is_ok() {
                debug_exe
            } else if fs::metadata(&release_exe).is_ok() {
                release_exe
            } else {
                PathBuf::from("target/debug/five-three-one")
            }
        });

    let output = Command::new(exe)
        .args(args)
        .output()
        .expect("failed to execute five-three-one binary");

    assert!(output.status.success(), "command failed with status {:?}", output.status);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    stdout.replace("\r\n", "\n")
}

#[test]
fn snapshot_squat_week_one_seed123() {
    let output = run_cli_with_seed(&[
        "--primary-lift",
        "squat",
        "--week",
        "1",
        "--seed",
        "123",
        "--config",
        &config_path(),
    ]);
    let expected = include_str!("fixtures/squat-week1-seed123.txt");
    assert_eq!(output, expected);
}

#[test]
fn snapshot_deadlift_week_two_with_warmup_mobility_core_seed321() {
    let output = run_cli_with_seed(&[
        "--primary-lift",
        "deadlift",
        "--week",
        "2",
        "--warmup",
        "--mobility",
        "--core-exercises",
        "3",
        "--seed",
        "321",
        "--config",
        &config_path(),
    ]);
    let expected = include_str!("fixtures/deadlift-week2-warmup-mobility-core-seed321.txt");
    assert_eq!(output, expected);
}

#[test]
fn snapshot_overhead_press_week_four_seed7() {
    let output = run_cli_with_seed(&[
        "--primary-lift",
        "overhead-press",
        "--week",
        "4",
        "--seed",
        "7",
        "--config",
        &config_path(),
    ]);
    let expected = include_str!("fixtures/overhead-press-week4-seed7.txt");
    assert_eq!(output, expected);
}

use assert_cmd::prelude::*;
use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;

const BIN: &str = env!("CARGO_PKG_NAME");

static FIXTURES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"));
#[test]
fn can_run_in_kts_project_root() -> Result<(), Box<dyn Error>> {
    let fixture = "kts_files";
    let tmp_dir = tmp_dir(fixture)?;
    tmp_dir.copy_from(&*FIXTURES_DIR, &["kts_files/**"])?;

    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(tmp_dir.path().join(fixture));
    cmd.assert().success();
    Ok(())
}
#[test]
fn can_run_in_groovy_project_root() -> Result<(), Box<dyn Error>> {
    let fixture = "groovy_files";
    let tmp_dir = tmp_dir(fixture)?;

    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(tmp_dir.path().join(fixture));
    cmd.assert().success();
    Ok(())
}

#[test]
fn can_run_in_kts_subdirectory() -> Result<(), Box<dyn Error>> {
    let fixture = "kts_files";
    let tmp_dir = tmp_dir(fixture)?;

    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(tmp_dir.path().join(fixture).join("src/main/java"));
    cmd.assert().success();
    Ok(())
}
#[test]
fn can_run_in_groovy_subdirectory() -> Result<(), Box<dyn Error>> {
    let fixture = "groovy_files";
    let tmp_dir = tmp_dir(fixture)?;

    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(tmp_dir.path().join(fixture).join("src/main/java"));
    cmd.assert().success();
    Ok(())
}

fn tmp_dir(fixture: &str) -> Result<TempDir, Box<dyn Error>> {
    let temp_dir = TempDir::new()?.into_persistent_if(env::var_os("TEST_PERSIST_FILES").is_some());
    temp_dir.copy_from(&*FIXTURES_DIR, &[format!("{}/**", fixture)])?;
    Ok(temp_dir)
}

#[test]
fn can_pass_args() -> Result<(), Box<dyn Error>> {
    let fixture = "kts_files";
    let tmp_dir = tmp_dir(fixture)?;
    tmp_dir.copy_from(&*FIXTURES_DIR, &["kts_files/**"])?;

    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(tmp_dir.path().join(fixture));
    cmd.args(["tasks"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tasks runnable from root project"));
    Ok(())
}
#[test]
fn gradlew_failure_is_passed_through() -> Result<(), Box<dyn Error>> {
    let fixture = "kts_files";
    let tmp_dir = tmp_dir(fixture)?;
    tmp_dir.copy_from(&*FIXTURES_DIR, &["kts_files/**"])?;

    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(tmp_dir.path().join(fixture));
    cmd.args(["non-existent-task"]);
    cmd.assert().failure();
    Ok(())
}

// Fails when system has `gradle` on $PATH
#[test]
#[ignore]
fn can_fail_to_find_gradlew() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(BIN)?;
    cmd.current_dir(".");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Did not find build.gradle or build.gradle.kts file!",
    ));

    Ok(())
}

// fails on ci windows. TODO: enable
#[cfg(unix)]
#[test]
#[ignore]
fn uses_gradle_from_path() -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    let mut cmd = Command::new("sh");

    #[cfg(windows)]
    let mut cmd = Command::new("cmd");

    let dir_with_gradle_executable = env::current_dir().unwrap().join(PathBuf::from("tests"));

    #[cfg(unix)]
    let path = format!(
        "{}:{}",
        dir_with_gradle_executable.to_str().unwrap(),
        env::var("PATH").unwrap()
    );

    #[cfg(windows)]
    let path = format!(
        "{};{}",
        std::env::var("PATH").unwrap(),
        dir_with_gradle_executable.to_str().unwrap()
    );

    cmd.env("PATH", path);
    cmd.current_dir("./tests/gradle_project");

    #[cfg(windows)]
    cmd.arg("/C");

    #[cfg(unix)]
    cmd.arg("-c");

    let mut path1 = env::current_exe().unwrap();
    path1.pop();
    if path1.ends_with("deps") {
        path1.pop();
    }
    let exe = String::from(BIN) + env::consts::EXE_SUFFIX;
    path1.push(exe);
    cmd.arg(path1);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains(
            "Did not find gradlew wrapper! Trying gradle from $PATH",
        ))
        .stdout(
            predicate::str::contains("This is global gradle. You made it!")
                // in case you have installed gradle on your system
                .or(predicate::str::contains("Welcome to Gradle")),
        );

    Ok(())
}

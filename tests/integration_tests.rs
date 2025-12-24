//! Integration tests for vibeanvil CLI

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn vibeanvil() -> Command {
    Command::cargo_bin("vibeanvil").unwrap()
}

#[test]
fn test_help() {
    vibeanvil()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Contract-first vibe coding"));
}

#[test]
fn test_version() {
    vibeanvil()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("vibeanvil"));
}

#[test]
fn test_init_creates_workspace() {
    let temp = TempDir::new().unwrap();
    
    vibeanvil()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    // Check workspace was created
    assert!(temp.path().join(".vibeanvil").exists());
    assert!(temp.path().join(".vibeanvil/state.json").exists());
    assert!(temp.path().join(".vibeanvil/logs").exists());
    assert!(temp.path().join(".vibeanvil/sessions").exists());
}

#[test]
fn test_init_force_reinitializes() {
    let temp = TempDir::new().unwrap();
    
    // First init
    vibeanvil()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    // Second init without force should warn
    vibeanvil()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    // With force should succeed
    vibeanvil()
        .args(["init", "--force"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));
}

#[test]
fn test_status_without_init_fails() {
    let temp = TempDir::new().unwrap();
    
    vibeanvil()
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_status_after_init() {
    let temp = TempDir::new().unwrap();
    
    // Init first
    vibeanvil()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    // Status should work
    vibeanvil()
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("INIT"));
}

#[test]
fn test_full_workflow_to_contract_lock() {
    let temp = TempDir::new().unwrap();
    
    // Init
    vibeanvil()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    // Intake
    vibeanvil()
        .args(["intake", "--message", "Build a test project"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Intake captured"));

    // Blueprint
    vibeanvil()
        .args(["blueprint", "--auto"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blueprint drafted"));

    // Contract create
    vibeanvil()
        .args(["contract", "create"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Contract created"));

    // Contract validate
    vibeanvil()
        .args(["contract", "validate"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Contract lock
    vibeanvil()
        .args(["contract", "lock"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("LOCKED"));

    // Verify lock file exists
    assert!(temp.path().join(".vibeanvil/contract.lock").exists());
}

#[test]
fn test_log_empty() {
    let temp = TempDir::new().unwrap();
    
    vibeanvil()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    vibeanvil()
        .arg("log")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_brain_stats_empty() {
    vibeanvil()
        .args(["brain", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Total records"));
}

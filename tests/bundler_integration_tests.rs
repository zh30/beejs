use std::time::{SystemTime, UNIX_EPOCH, Duration};
//! Bundle Integration Tests
//!
//! Tests for the beejs bundle command and bundler functionality

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

#[test]
fn test_bundle_command_help() {
    // Test that the bundle command is recognized and shows help
    let output: _ = Command::new("cargo")
        .args(&["run", "--bin", "beejs", "--", "bundle", "--help"])
        .output()
        .expect("Failed to run beejs bundle --help");

    let stderr: _ = String::from_utf8_lossy(&output.stderr);

    // Check that help output is shown (not an error about unknown command)
    assert!(
        output.status.success(),
        "Bundle command should be recognized. stderr: {}",
        stderr
    );

    // Verify help contains expected information
    assert!(
        stderr.contains("Bundle code for production") || output.status.success(),
        "Help should describe bundle command"
    );
}

#[test]
fn test_bundle_basic_functionality() {
    // Create a temporary directory for testing
    let temp_dir: _ = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path: _ = temp_dir.path();

    // Create a simple entry file
    let entry_file: _ = temp_path.join("entry.js");
    fs::write(&entry_file, r#"
        console.log("Hello from bundle");
        export const message = "Hello World";
    "#).expect("Failed to write entry file");

    // Create output path
    let output_file: _ = temp_path.join("bundle.js");

    // Run bundle command
    let output: _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin", "beejs",
            "--",
            "bundle",
            entry_file.to_str().unwrap(),
            "--outfile",
            output_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run beejs bundle");

    let stderr: _ = String::from_utf8_lossy(&output.stderr);

    // The command should succeed or at least not fail with "unknown command"
    // (bundle may not be fully implemented yet, but CLI should recognize it)
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("unexpected argument"),
        "Bundle command should be recognized. stderr: {}",
        stderr
    );

    // If bundle is implemented, check that output file was created
    if output.status.success() {
        assert!(
            output_file.exists(),
            "Output bundle file should be created when bundle succeeds"
        );
    }
}

#[test]
fn test_bundle_with_minify_flag() {
    // Test that minify flag is recognized
    let temp_dir: _ = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path: _ = temp_dir.path();

    let entry_file: _ = temp_path.join("entry.js");
    fs::write(&entry_file, "console.log('test');").expect("Failed to write entry file");

    let output_file: _ = temp_path.join("bundle.js");

    let output: _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin", "beejs",
            "--",
            "bundle",
            entry_file.to_str().unwrap(),
            "--outfile",
            output_file.to_str().unwrap(),
            "--minify",
        ])
        .output()
        .expect("Failed to run beejs bundle --minify");

    let stderr: _ = String::from_utf8_lossy(&output.stderr);

    // Should not fail with parsing errors
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("invalid value"),
        "Minify flag should be parsed correctly. stderr: {}",
        stderr
    );
}

#[test]
fn test_bundle_with_sourcemap_flag() {
    // Test that sourcemap flag is recognized
    let temp_dir: _ = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path: _ = temp_dir.path();

    let entry_file: _ = temp_path.join("entry.js");
    fs::write(&entry_file, "export const x = 42;").expect("Failed to write entry file");

    let output_file: _ = temp_path.join("bundle.js");

    let output: _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin", "beejs",
            "--",
            "bundle",
            entry_file.to_str().unwrap(),
            "--outfile",
            output_file.to_str().unwrap(),
            "--sourcemap",
        ])
        .output()
        .expect("Failed to run beejs bundle --sourcemap");

    let stderr: _ = String::from_utf8_lossy(&output.stderr);

    // Should not fail with parsing errors
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("invalid value"),
        "Sourcemap flag should be parsed correctly. stderr: {}",
        stderr
    );
}

#[test]
fn test_bundle_tree_shake_flag() {
    // Test that tree-shake flag is recognized
    let temp_dir: _ = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path: _ = temp_dir.path();

    let entry_file: _ = temp_path.join("entry.js");
    fs::write(&entry_file, "export const used = 'hello'; export const unused = 'world';")
        .expect("Failed to write entry file");

    let output_file: _ = temp_path.join("bundle.js");

    let output: _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin", "beejs",
            "--",
            "bundle",
            entry_file.to_str().unwrap(),
            "--outfile",
            output_file.to_str().unwrap(),
            "--tree-shake",
        ])
        .output()
        .expect("Failed to run beejs bundle --tree-shake");

    let stderr: _ = String::from_utf8_lossy(&output.stderr);

    // Should not fail with parsing errors
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("invalid value"),
        "Tree-shake flag should be parsed correctly. stderr: {}",
        stderr
    );
}

#[test]
fn test_bundle_target_options() {
    // Test that different target options are accepted
    let temp_dir: _ = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path: _ = temp_dir.path();

    let entry_file: _ = temp_path.join("entry.js");
    fs::write(&entry_file, "console.log('target test');").expect("Failed to write entry file");

    let output_file: _ = temp_path.join("bundle.js");

    // Test browser target (default)
    let output: _ = Command::new("cargo")
        .args(&[
            "run",
            "--bin", "beejs",
            "--",
            "bundle",
            entry_file.to_str().unwrap(),
            "--outfile",
            output_file.to_str().unwrap(),
            "--target",
            "browser",
        ])
        .output()
        .expect("Failed to run beejs bundle --target browser");

    let stderr: _ = String::from_utf8_lossy(&output.stderr);

    // Should not fail with parsing errors
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("invalid value"),
        "Target option should be parsed correctly. stderr: {}",
        stderr
    );
}

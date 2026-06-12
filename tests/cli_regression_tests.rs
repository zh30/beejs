use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn bee_path() -> &'static str {
    env!("CARGO_BIN_EXE_bee")
}

fn js_string(value: &Path) -> String {
    serde_json::to_string(&value.to_string_lossy().to_string()).expect("path should encode as JSON")
}

#[test]
fn create_help_does_not_panic() {
    let output = Command::new(bee_path())
        .args(["create", "--help"])
        .output()
        .expect("failed to execute bee create --help");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "bee create --help should exit successfully. stderr: {stderr}"
    );
    assert!(
        !stderr.to_lowercase().contains("panic"),
        "bee create --help should not panic. stderr: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage:"),
        "help output should include a usage line. stdout: {stdout}"
    );
}

#[test]
fn test_command_executes_test_callbacks_and_reports_assertion_failure() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("failing_assertion.test.js");
    std::fs::write(
        &test_file,
        r#"
test("fails inside the test callback", () => {
  expect(1).toBe(2);
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    assert!(
        !output.status.success(),
        "bee test should return non-zero for a failing assertion"
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("Expected 1 to be 2"),
        "bee test should report the assertion failure, not a missing test global. output: {combined}"
    );
    assert!(
        !combined.contains("Tests passed"),
        "bee test must not report success for a failing test. output: {combined}"
    );
}

#[test]
fn run_file_resolves_commonjs_relative_to_script_directory() {
    let dir = tempdir().expect("failed to create tempdir");
    let app_dir = dir.path().join("app");
    let lib_dir = app_dir.join("lib");
    std::fs::create_dir_all(&lib_dir).expect("failed to create lib directory");
    std::fs::write(lib_dir.join("index.js"), "exports.value = 314;")
        .expect("failed to write lib module");
    let main_file = app_dir.join("main.js");
    std::fs::write(&main_file, "require('./lib').value;").expect("failed to write main file");

    let output = Command::new(bee_path())
        .arg("run")
        .arg(&main_file)
        .output()
        .expect("failed to execute bee run");

    assert!(
        output.status.success(),
        "bee run should resolve relative require from script dir. stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "314");
}

#[test]
fn test_command_reports_async_assertion_failure() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("failing_async_assertion.test.js");
    std::fs::write(
        &test_file,
        r#"
test("fails after an awaited microtask", async () => {
  await Promise.resolve();
  expect(1).toBe(2);
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    assert!(
        !output.status.success(),
        "bee test should return non-zero for an async assertion failure"
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("Expected 1 to be 2"),
        "bee test should report async assertion failures. output: {combined}"
    );
    assert!(
        !combined.contains("Tests passed"),
        "bee test must not report success for a failing async test. output: {combined}"
    );
}

#[test]
fn run_passes_script_args_to_process_argv() {
    let dir = tempdir().expect("failed to create tempdir");
    let script = dir.path().join("argv.js");
    std::fs::write(
        &script,
        r#"
console.log(JSON.stringify(process.argv.slice(2)));
"#,
    )
    .expect("failed to write script");

    let output = Command::new(bee_path())
        .arg("run")
        .arg(&script)
        .args(["alpha", "beta"])
        .output()
        .expect("failed to execute bee run");

    assert!(
        output.status.success(),
        "bee run should exit successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        r#"["alpha","beta"]"#
    );
}

#[test]
fn test_command_sets_process_argv_for_test_file() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("argv.test.js");
    std::fs::write(
        &test_file,
        r#"
test("has test file in process argv", () => {
  if (!process.argv[1] || !process.argv[1].endsWith("argv.test.js")) {
    throw new Error(`unexpected argv: ${JSON.stringify(process.argv)}`);
  }
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    assert!(
        output.status.success(),
        "bee test should set process.argv for test files. stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_command_test_name_pattern_runs_only_matching_file_tests() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("name_pattern.test.js");
    std::fs::write(
        &test_file,
        r#"
test("selected case", () => {
  console.log("ran selected");
});

test("unmatched case", () => {
  throw new Error("unmatched test should not run");
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .args(["test", "--test-name-pattern", "selected"])
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "--test-name-pattern should skip non-matching tests. output: {combined}"
    );
    assert!(
        combined.contains("1 passed, 0 failed, 1 skipped"),
        "--test-name-pattern should report one skipped test. output: {combined}"
    );
    assert!(
        combined.contains("ran selected"),
        "matching test should run. output: {combined}"
    );
    assert!(
        !combined.contains("unmatched test should not run"),
        "non-matching test should not execute. output: {combined}"
    );
}

#[test]
fn test_command_test_skip_skips_matching_file_tests() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("skip_pattern.test.js");
    std::fs::write(
        &test_file,
        r#"
test("fast case", () => {
  console.log("ran fast");
});

test("slow case", () => {
  throw new Error("slow test should not run");
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .args(["test", "--test-skip", "slow"])
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "--test-skip should skip matching tests. output: {combined}"
    );
    assert!(
        combined.contains("1 passed, 0 failed, 1 skipped"),
        "--test-skip should report one skipped test. output: {combined}"
    );
    assert!(
        combined.contains("ran fast"),
        "non-skipped test should run. output: {combined}"
    );
    assert!(
        !combined.contains("slow test should not run"),
        "skipped test should not execute. output: {combined}"
    );
}

#[test]
fn test_command_bail_stops_after_first_file_test_failure() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("bail.test.js");
    std::fs::write(
        &test_file,
        r#"
test("first failure", () => {
  throw new Error("first failed");
});

test("second should not run", () => {
  console.log("ran after first failure");
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .args(["test", "--bail"])
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "--bail should still fail the command on the first failure. output: {combined}"
    );
    assert!(
        combined.contains("first failed"),
        "first failure should be reported. output: {combined}"
    );
    assert!(
        !combined.contains("ran after first failure"),
        "--bail should prevent later tests from executing. output: {combined}"
    );
}

#[test]
fn test_command_timeout_fails_slow_file_promise() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("timeout.test.js");
    std::fs::write(
        &test_file,
        r#"
test("slow promise", () => new Promise((resolve) => {
  setTimeout(resolve, 250);
}));
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .args(["test", "--timeout", "0"])
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "--timeout should fail a slow promise. output: {combined}"
    );
    assert!(
        combined.contains("timed out after 0s"),
        "timeout failure should mention the configured timeout. output: {combined}"
    );
}

#[test]
fn test_command_default_timeout_fails_pending_timer_promise() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("default_timeout.test.js");
    std::fs::write(
        &test_file,
        r#"
test("pending promise must not be treated as passed", () => new Promise((resolve) => {
  setTimeout(resolve, 250);
}));
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "bee test must fail unresolved timer promises instead of printing success. output: {combined}"
    );
    assert!(
        combined.contains("timed out"),
        "default timeout failure should explain the pending async test. output: {combined}"
    );
}

#[test]
fn test_command_timeout_seconds_are_not_capped_to_75ms() {
    let dir = tempdir().expect("failed to create tempdir");
    let test_file = dir.path().join("timeout_seconds.test.js");
    std::fs::write(
        &test_file,
        r#"
test("settles before one second", () => new Promise((resolve) => {
  setTimeout(resolve, 120);
}));
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .args(["test", "--timeout", "1"])
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "--timeout 1 should not be capped to 75ms. output: {combined}"
    );
    assert!(
        combined.contains("1 passed, 0 failed"),
        "the 120ms test should complete within the configured second. output: {combined}"
    );
}

#[test]
fn test_command_without_file_discovers_and_fails_project_tests() {
    let dir = tempdir().expect("failed to create tempdir");
    std::fs::write(
        dir.path().join("failing.test.js"),
        r#"
test("real project test fails", () => {
  expect(1).toBe(2);
});
"#,
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .current_dir(dir.path())
        .arg("test")
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "bee test should fail discovered project tests. output: {combined}"
    );
    assert!(
        combined.contains("Expected 1 to be 2"),
        "discovered test failure should be reported. output: {combined}"
    );
}

#[test]
fn eval_deny_fs_blocks_read_file_sync() {
    let dir = tempdir().expect("failed to create tempdir");
    let secret = dir.path().join("secret.txt");
    std::fs::write(&secret, "secret-value").expect("failed to write secret");

    let code = format!(
        r#"
try {{
  require("fs").readFileSync({}, "utf8");
  console.log("allowed");
}} catch (error) {{
  console.log(String(error && error.message || error));
}}
"#,
        js_string(&secret)
    );

    let output = Command::new(bee_path())
        .args(["eval", "--deny-fs"])
        .arg(code)
        .output()
        .expect("failed to execute bee eval");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "caught permission errors should keep eval process successful. output: {combined}"
    );
    assert!(
        combined.contains("permission denied"),
        "--deny-fs should deny fs.readFileSync. output: {combined}"
    );
    assert!(
        !combined.contains("secret-value") && !combined.contains("allowed"),
        "denied read must not leak the file contents. output: {combined}"
    );
}

#[test]
fn eval_deny_fs_allows_explicit_read_exception() {
    let dir = tempdir().expect("failed to create tempdir");
    let secret = dir.path().join("secret.txt");
    std::fs::write(&secret, "secret-value").expect("failed to write secret");

    let code = format!(
        r#"require("fs").readFileSync({}, "utf8");"#,
        js_string(&secret)
    );

    let output = Command::new(bee_path())
        .arg("eval")
        .arg("--deny-fs")
        .arg("--allow-read")
        .arg(&secret)
        .arg(code)
        .output()
        .expect("failed to execute bee eval");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "--allow-read should allow the exact path after --deny-fs. output: {combined}"
    );
    assert!(
        combined.contains("secret-value"),
        "allowed read should print the file contents. output: {combined}"
    );
}

#[test]
fn eval_deny_fs_blocks_write_file_sync() {
    let dir = tempdir().expect("failed to create tempdir");
    let output_file = dir.path().join("created.txt");

    let code = format!(
        r#"
try {{
  require("fs").writeFileSync({}, "created");
  console.log("allowed");
}} catch (error) {{
  console.log(String(error && error.message || error));
}}
"#,
        js_string(&output_file)
    );

    let output = Command::new(bee_path())
        .args(["eval", "--deny-fs"])
        .arg(code)
        .output()
        .expect("failed to execute bee eval");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "caught permission errors should keep eval process successful. output: {combined}"
    );
    assert!(
        combined.contains("permission denied"),
        "--deny-fs should deny fs.writeFileSync. output: {combined}"
    );
    assert!(
        !output_file.exists(),
        "denied write must not create the target file. output: {combined}"
    );
}

#[test]
fn eval_deny_fs_allows_explicit_write_exception() {
    let dir = tempdir().expect("failed to create tempdir");
    let output_file = dir.path().join("created.txt");

    let code = format!(
        r#"require("fs").writeFileSync({}, "created"); "done";"#,
        js_string(&output_file)
    );

    let output = Command::new(bee_path())
        .arg("eval")
        .arg("--deny-fs")
        .arg("--allow-write")
        .arg(&output_file)
        .arg(code)
        .output()
        .expect("failed to execute bee eval");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "--allow-write should allow the exact path after --deny-fs. output: {combined}"
    );
    assert_eq!(
        std::fs::read_to_string(&output_file).expect("allowed write should create the file"),
        "created"
    );
}

#[test]
fn run_deny_fs_blocks_script_fs_read() {
    let dir = tempdir().expect("failed to create tempdir");
    let secret = dir.path().join("secret.txt");
    let script = dir.path().join("read-secret.js");
    std::fs::write(&secret, "secret-value").expect("failed to write secret");
    std::fs::write(
        &script,
        format!(
            r#"
try {{
  console.log(require("fs").readFileSync({}, "utf8"));
}} catch (error) {{
  console.log(String(error && error.message || error));
}}
"#,
            js_string(&secret)
        ),
    )
    .expect("failed to write script");

    let output = Command::new(bee_path())
        .args(["run", "--deny-fs"])
        .arg(&script)
        .output()
        .expect("failed to execute bee run");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "caught permission errors should keep run process successful. output: {combined}"
    );
    assert!(
        combined.contains("permission denied"),
        "--deny-fs should deny fs.readFileSync from scripts. output: {combined}"
    );
    assert!(
        !combined.contains("secret-value"),
        "denied read must not leak the file contents. output: {combined}"
    );
}

#[test]
fn run_deny_fs_blocks_relative_require_module_file() {
    let dir = tempdir().expect("failed to create tempdir");
    let lib = dir.path().join("lib.js");
    let script = dir.path().join("main.js");
    std::fs::write(&lib, "module.exports = { value: 7 };").expect("failed to write module");
    std::fs::write(&script, "require('./lib').value;").expect("failed to write script");

    let output = Command::new(bee_path())
        .args(["run", "--deny-fs"])
        .arg(&script)
        .output()
        .expect("failed to execute bee run");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "uncaught denied module load should fail bee run. output: {combined}"
    );
    assert!(
        combined.contains("permission denied"),
        "--deny-fs should deny CommonJS module file reads. output: {combined}"
    );
}

#[test]
fn test_command_deny_fs_blocks_file_test_fs_read() {
    let dir = tempdir().expect("failed to create tempdir");
    let secret = dir.path().join("secret.txt");
    let test_file = dir.path().join("permission.test.js");
    std::fs::write(&secret, "secret-value").expect("failed to write secret");
    std::fs::write(
        &test_file,
        format!(
            r#"
test("fs read is denied", () => {{
  require("fs").readFileSync({}, "utf8");
}});
"#,
            js_string(&secret)
        ),
    )
    .expect("failed to write test file");

    let output = Command::new(bee_path())
        .args(["test", "--deny-fs"])
        .arg(&test_file)
        .output()
        .expect("failed to execute bee test");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "bee test should fail when a test body hits denied fs. output: {combined}"
    );
    assert!(
        combined.contains("permission denied"),
        "--deny-fs should deny fs reads inside bee test callbacks. output: {combined}"
    );
}

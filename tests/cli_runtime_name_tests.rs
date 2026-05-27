use std::process::Command;

fn bee_path() -> &'static str {
    env!("CARGO_BIN_EXE_bee")
}

#[test]
fn runtime_binary_is_named_bee() {
    let output = Command::new(bee_path())
        .arg("--version")
        .output()
        .expect("failed to execute bee --version");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("bee {}", env!("CARGO_PKG_VERSION"))
    );
}

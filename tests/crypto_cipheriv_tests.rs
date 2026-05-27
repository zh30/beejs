// crypto.createCipheriv Tests - v0.3.15
// Tests for crypto.createCipheriv symmetric encryption function with explicit key and IV

use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn beejs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bee"))
}

fn run_js_test(code: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.js");
    fs::write(&test_file, code).unwrap();

    let output = Command::new(beejs_path())
        .arg("run")
        .arg(&test_file)
        .output()
        .expect("Failed to execute bee");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with("🐝") && !line.starts_with("Result:"))
        .collect();
    lines.join("\n")
}

// ==================== createCipheriv Tests ====================

#[test]
#[serial]
fn test_create_cipheriv_function_exists() {
    let code = r#"
console.log(typeof crypto.createCipheriv === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.trim() == "PASS",
        "Expected createCipheriv to exist: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_aes_256_cbc() {
    // AES-256-CBC requires 32-byte key and 16-byte IV
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'; // 64 hex chars = 32 bytes
const iv = 'abcdef0123456789abcdef0123456789'; // 32 hex chars = 16 bytes
const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
console.log(cipher && typeof cipher.update === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected AES-256-CBC cipheriv to work: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_aes_128_cbc() {
    // AES-128-CBC requires 16-byte key and 16-byte IV (32 hex chars for key, 32 for IV)
    let code = r#"
const key = '0123456789abcdef0123456789abcdef'; // 32 hex chars = 16 bytes
const iv = 'abcdef0123456789abcdef0123456789'; // 32 hex chars = 16 bytes
const cipher = crypto.createCipheriv('aes-128-cbc', key, iv);
console.log(cipher && typeof cipher.update === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected AES-128-CBC cipheriv to work: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_aes_192_cbc() {
    // AES-192-CBC requires 24-byte key and 16-byte IV (48 hex chars for key, 32 for IV)
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef'; // 48 hex chars = 24 bytes
const iv = 'abcdef0123456789abcdef0123456789'; // 32 hex chars = 16 bytes
const cipher = crypto.createCipheriv('aes-192-cbc', key, iv);
console.log(cipher && typeof cipher.update === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected AES-192-CBC cipheriv to work: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_has_update_method() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
console.log(typeof cipher.update === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected cipher to have update method: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_has_finalize_method() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
console.log(typeof cipher.final === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected cipher to have final method: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_invalid_algorithm() {
    let code = r#"
try {
    crypto.createCipheriv('invalid-alg', 'key', 'iv');
    console.log('FAIL');
} catch (e) {
    console.log(e.message.includes('unsupported algorithm') ? 'PASS' : 'FAIL');
}
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected error for invalid algorithm: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_invalid_key_length() {
    let code = r#"
try {
    // AES-256 requires 32-byte key, using 16 bytes should fail
    crypto.createCipheriv('aes-256-cbc', '0123456789abcdef', 'abcdef0123456789abcdef0123456789');
    console.log('FAIL');
} catch (e) {
    console.log(e.message.includes('invalid') || e.message.includes('key') ? 'PASS' : 'FAIL');
}
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected error for invalid key length: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_invalid_iv_length() {
    let code = r#"
try {
    // CBC mode requires 16-byte IV, using 8 bytes should fail
    crypto.createCipheriv('aes-256-cbc', '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef', 'shortiv');
    console.log('FAIL');
} catch (e) {
    console.log(e.message.includes('iv') || e.message.includes('invalid') ? 'PASS' : 'FAIL');
}
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected error for invalid IV length: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_update_and_final() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
const encrypted = cipher.update('Hello, World!', 'utf8', 'hex') + cipher.final('hex');
console.log(encrypted.length > 0 ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected cipher to produce encrypted output: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_update_buffer() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
const encrypted = cipher.update(Buffer.from('Hello'), 'utf8', 'hex') + cipher.final('hex');
console.log(encrypted.length > 0 ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected cipher to work with Buffer input: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_cipheriv_decrypt_round_trip() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const original = 'Hello, World!';

// Encrypt
const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
const encrypted = cipher.update(original, 'utf8', 'hex') + cipher.final('hex');

// Decrypt using createDecipheriv
const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
const decrypted = decipher.update(encrypted, 'hex', 'utf8') + decipher.final('utf8');

console.log(decrypted === original ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected encrypt/decrypt round trip to work: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_decipheriv_function_exists() {
    let code = r#"
console.log(typeof crypto.createDecipheriv === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.trim() == "PASS",
        "Expected createDecipheriv to exist: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_decipheriv_has_update_method() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
console.log(typeof decipher.update === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected decipher to have update method: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_decipheriv_has_finalize_method() {
    let code = r#"
const key = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const iv = 'abcdef0123456789abcdef0123456789';
const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
console.log(typeof decipher.final === 'function' ? 'PASS' : 'FAIL');
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected decipher to have final method: {}",
        output
    );
}

#[test]
#[serial]
fn test_create_decipheriv_invalid_algorithm() {
    let code = r#"
try {
    crypto.createDecipheriv('invalid-alg', 'key', 'iv');
    console.log('FAIL');
} catch (e) {
    console.log(e.message.includes('unsupported algorithm') ? 'PASS' : 'FAIL');
}
"#;
    let output = run_js_test(code);
    assert!(
        output.contains("PASS"),
        "Expected error for invalid algorithm: {}",
        output
    );
}

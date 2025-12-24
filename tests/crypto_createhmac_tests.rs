//! Tests for crypto.createHmac module (v0.3.9)
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_crypto_module_exists_hmac() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_create_hmac_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.createHmac");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_create_hmac_md5() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('md5', 'secret_key');
        hmac.update('hello');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // HMAC-MD5("hello", "secret_key") - 32 hex chars
    assert_eq!(output.len(), 32);
}

#[test]
#[serial]
fn test_create_hmac_sha256() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('sha256', 'my_secret_key');
        hmac.update('message');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // HMAC-SHA256("message", "my_secret_key") - length check only for now
    assert_eq!(output.len(), 64);
}

#[test]
#[serial]
fn test_create_hmac_sha512() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('sha512', 'key');
        hmac.update('data');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // SHA512 produces 128 hex chars
    assert_eq!(output.len(), 128);
}

#[test]
#[serial]
fn test_create_hmac_blake3() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('blake3', 'secret');
        hmac.update('test');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // BLAKE3 produces 64 hex chars
    assert_eq!(output.len(), 64);
}

#[test]
#[serial]
fn test_hmac_chain_update() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('md5', 'key');
        hmac.update('part1').update('part2');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // HMAC-MD5("part1part2", "key") - 32 hex chars
    assert_eq!(output.len(), 32);
}

#[test]
#[serial]
fn test_hmac_base64_encoding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('sha256', 'secret');
        hmac.update('hello');
        hmac.digest('base64');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // SHA256 produces 32 bytes = ~44 base64 chars
    assert!(output.len() >= 40 && output.len() <= 50);
}

#[test]
#[serial]
fn test_hmac_unsupported_algorithm() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.createHmac('unsupported', 'key');
        } catch (e) {
            e.message;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().trim().contains("unsupported"));
}

#[test]
#[serial]
fn test_hmac_update_returns_hmac_object() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('md5', 'key');
        const result = hmac.update('test');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_hmac_algorithm_property() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('sha256', 'key');
        hmac._algorithm;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "sha256");
}

#[test]
#[serial]
fn test_hmac_key_property() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('sha256', 'my_secret_key');
        hmac._key;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "my_secret_key");
}

#[test]
#[serial]
fn test_hmac_empty_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('md5', '');
        hmac.update('message');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim().len(), 32);
}

#[test]
#[serial]
fn test_hmac_empty_message() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hmac = crypto.createHmac('md5', 'key');
        hmac.update('');
        hmac.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim().len(), 32);
}

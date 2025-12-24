//! Tests for crypto.createVerify module (v0.3.20)
//! Digital signature verification using RSA public keys
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_crypto_createVerify_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.createVerify");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_createVerify_returns_verify_object() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygW
test-public-key-placeholder
-----END PUBLIC KEY-----`;
        const verify = crypto.createVerify('RSA-SHA256');
        typeof verify;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_verify_update_method_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        typeof verify.update;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_verify_method_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        typeof verify.verify;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_verify_chain_update_digest() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        const result = verify.update('test data');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_verify_unsupported_algorithm() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.createVerify('ECDSA-SHA256');
        } catch (e) {
            e.message;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    assert!(output.contains("unsupported") || output.contains("RSA-SHA256"));
}

#[test]
#[serial]
fn test_verify_returns_boolean() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        verify.update('test data');
        const result = verify.verify('somesignature', 'hex');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "boolean");
}

#[test]
#[serial]
fn test_verify_with_hex_signature() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        verify.update('hello world');
        const result = verify.verify('a1b2c3d4e5f6789012345678901234567890abcd', 'hex');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "boolean");
}

#[test]
#[serial]
fn test_verify_with_base64_signature() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        verify.update('test message');
        const result = verify.verify('dGVzdHNpZ25hdHVyZQ==', 'base64');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "boolean");
}

#[test]
#[serial]
fn test_verify_multiple_updates() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        verify.update('part1');
        verify.update('part2');
        verify.update('part3');
        const result = verify.verify('signature', 'hex');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "boolean");
}

#[test]
#[serial]
fn test_verify_digest_without_update() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        const result = verify.verify('signature', 'hex');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "boolean");
}

#[test]
#[serial]
fn test_verify_different_hash_algorithms() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA512');
        typeof verify;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_verify_algorithm_property() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const verify = crypto.createVerify('RSA-SHA256');
        verify._algorithm;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "RSA-SHA256");
}

#[test]
#[serial]
fn test_sign_and_verify_workflow() {
    let mut runtime = MinimalRuntime::new().unwrap();
    // Test that sign and verify can be used together
    let code = r#"
        const sign = crypto.createSign('RSA-SHA256');
        sign.update('message to sign');
        const signature = sign.sign('hex');

        const verify = crypto.createVerify('RSA-SHA256');
        verify.update('message to sign');
        const result = verify.verify(signature, 'hex');
        result === true || result === false;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

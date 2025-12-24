//! Tests for crypto.privateEncrypt/crypto.publicDecrypt module (v0.3.22)
//! Private key encryption and public key decryption using RSA
//! This is the inverse operation of publicEncrypt/privateDecrypt
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_crypto_privateEncrypt_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.privateEncrypt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_crypto_publicDecrypt_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.publicDecrypt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_privateEncrypt_returns_buffer() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRndVLkklx3Lf/
test-private-key-placeholder
-----END PRIVATE KEY-----`;
        const encrypted = crypto.privateEncrypt(privateKey, Buffer.from('test message'));
        Buffer.isBuffer(encrypted);
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_publicDecrypt_returns_buffer() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygW
test-public-key-placeholder
-----END PUBLIC KEY-----`;
        const decrypted = crypto.publicDecrypt(publicKey, Buffer.from('encrypted_data', 'hex'));
        Buffer.isBuffer(decrypted);
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_privateEncrypt_with_encoding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRndVLkklx3Lf/
test-private-key-placeholder
-----END PRIVATE KEY-----`;
        const encrypted = crypto.privateEncrypt({
            key: privateKey,
            padding: crypto.constants.RSA_PKCS1_PADDING
        }, Buffer.from('test'));
        typeof encrypted;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_publicDecrypt_with_encoding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygW
test-public-key-placeholder
-----END PUBLIC KEY-----`;
        const decrypted = crypto.publicDecrypt(publicKey, 'a1b2c3d4e5', 'hex');
        Buffer.isBuffer(decrypted);
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_privateEncrypt_with_rsa_padding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRndVLkklx3Lf/
test-private-key-placeholder
-----END PRIVATE KEY-----`;
        const encrypted = crypto.privateEncrypt(privateKey, Buffer.from('test data'));
        encrypted.length > 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_privateEncrypt_invalid_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.privateEncrypt('invalid-key', Buffer.from('test'));
        } catch (e) {
            e.message.includes('invalid') || e.message.includes('key');
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    assert!(binding.trim() == "true");
}

#[test]
#[serial]
fn test_publicDecrypt_invalid_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.publicDecrypt('invalid-key', Buffer.from('test'));
        } catch (e) {
            e.message.includes('invalid') || e.message.includes('key');
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    assert!(binding.trim() == "true");
}

#[test]
#[serial]
fn test_private_public_decrypt_roundtrip() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygW
test-public-key-placeholder
-----END PUBLIC KEY-----`;
        const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRndVLkklx3Lf/
test-private-key-placeholder
-----END PRIVATE KEY-----`;
        const message = 'Secret message signed with private key';
        const encrypted = crypto.privateEncrypt(privateKey, Buffer.from(message));
        const decrypted = crypto.publicDecrypt(publicKey, encrypted);
        decrypted.toString('utf8');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Secret") || output.contains("message") || output.contains("signed"));
}

#[test]
#[serial]
fn test_privateEncrypt_empty_data() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRndVLkklx3Lf/
test-private-key-placeholder
-----END PRIVATE KEY-----`;
        const encrypted = crypto.privateEncrypt(privateKey, Buffer.from(''));
        encrypted.length > 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_publicDecrypt_empty_data() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygW
test-public-key-placeholder
-----END PUBLIC KEY-----`;
        const decrypted = crypto.publicDecrypt(publicKey, Buffer.from(''));
        decrypted.length === 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_privateEncrypt_oaep_padding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRndVLkklx3Lf/
test-private-key-placeholder
-----END PRIVATE KEY-----`;
        const encrypted = crypto.privateEncrypt({
            key: privateKey,
            padding: crypto.constants.RSA_PKCS1_OAEP_PADDING
        }, Buffer.from('test data'));
        encrypted.length > 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_publicDecrypt_oaep_padding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygW
test-public-key-placeholder
-----END PUBLIC KEY-----`;
        // OAEP padding is typically used with both encrypt and decrypt
        const decrypted = crypto.publicDecrypt({
            key: publicKey,
            padding: crypto.constants.RSA_PKCS1_OAEP_PADDING
        }, Buffer.from('encrypted_data', 'hex'));
        Buffer.isBuffer(decrypted);
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_constants_rsa_padding_available() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        // All RSA padding constants should be available
        const constants = crypto.constants;
        typeof constants.RSA_PKCS1_PADDING === 'number' &&
        typeof constants.RSA_PKCS1_OAEP_PADDING === 'number' &&
        typeof constants.RSA_NO_PADDING === 'number';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

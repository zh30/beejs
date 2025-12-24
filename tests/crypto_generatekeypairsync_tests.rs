// Tests for crypto.generateKeyPairSync module (v0.3.23)
// RSA and EC key pair generation for digital signatures and encryption
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_crypto_generateKeyPairSync_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.generateKeyPairSync");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_generateKeyPairSync_rsa_returns_object() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048,
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        });
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_generateKeyPairSync_rsa_has_keys() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048,
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        });
        typeof result.publicKey === 'string' && typeof result.privateKey === 'string';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_rsa_key_format() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048,
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        });
        result.publicKey.indexOf('-----BEGIN PUBLIC KEY-----') >= 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_ec_returns_object() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('ec', {
            namedCurve: 'prime256v1',
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        });
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_generateKeyPairSync_ec_has_keys() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('ec', {
            namedCurve: 'prime256v1',
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        });
        typeof result.publicKey === 'string' && typeof result.privateKey === 'string';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_ec_key_format() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('ec', {
            namedCurve: 'prime256v1'
        });
        result.publicKey.indexOf('-----BEGIN PUBLIC KEY-----') >= 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_rsa_different_modulus_lengths() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('rsa', {
            modulusLength: 4096
        });
        result.privateKey.length > 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_unsupported_type() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.generateKeyPairSync('dsa', { modulusLength: 2048 });
            false;
        } catch (e) {
            e.message.indexOf('unsupported') >= 0 || e.message.indexOf('not supported') >= 0;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_missing_options() {
    // Node.js: generateKeyPairSync('rsa') should use default options (modulusLength: 2048)
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.generateKeyPairSync('rsa');
        typeof result === 'object' && typeof result.publicKey === 'string' && typeof result.privateKey === 'string';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_key_usage_in_signing() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048
        });
        const sign = crypto.createSign('RSA-SHA256');
        sign.update('test message');
        const signature = sign.sign(privateKey);
        typeof signature === 'string' && signature.length > 0;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_key_usage_in_verification() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048
        });
        const sign = crypto.createSign('RSA-SHA256');
        sign.update('test message');
        const signature = sign.sign(privateKey);
        const verify = crypto.createVerify('RSA-SHA256');
        verify.update('test message');
        verify.verify(publicKey, signature) === true;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPairSync_multiple_calls_consistent() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result1 = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048
        });
        const result2 = crypto.generateKeyPairSync('rsa', {
            modulusLength: 2048
        });
        // Each call should generate unique keys
        result1.publicKey !== result2.publicKey;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

//! Tests for crypto.generateKeyPair module (v0.3.24)
//! Asynchronous RSA/EC key pair generation with callback pattern
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_crypto_generateKeyPair_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.generateKeyPair");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_generateKeyPair_rsa_with_callback() {
    let mut runtime = MinimalRuntime::new().unwrap();
    // Simple test - just verify the callback is invoked
    let code = r#"
        crypto.generateKeyPair('rsa', {
            modulusLength: 2048
        }, function(err, publicKey, privateKey) {
            callbackInvoked = true;
        });
        callbackInvoked;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_ec_with_callback() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('ec', {
            namedCurve: 'prime256v1'
        }, function(err, publicKey, privateKey) {
            callbackInvoked = true;
        });
        callbackInvoked;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_rsa_key_format() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('rsa', {
            modulusLength: 2048
        }, function(err, publicKey, privateKey) {
            hasPublicKey = (publicKey && publicKey.indexOf('-----BEGIN PUBLIC KEY-----') >= 0);
        });
        hasPublicKey;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_ec_key_format() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('ec', {
            namedCurve: 'prime256v1'
        }, function(err, publicKey, privateKey) {
            hasPublicKey = (publicKey && publicKey.indexOf('-----BEGIN PUBLIC KEY-----') >= 0);
        });
        hasPublicKey;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_unsupported_type() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('dsa', { modulusLength: 2048 }, function(err, publicKey, privateKey) {
            gotError = (err && err.message && err.message.indexOf('unsupported') >= 0);
        });
        gotError;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_missing_callback_throws() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.generateKeyPair('rsa', {});
            false;
        } catch (e) {
            e.message.indexOf('callback') >= 0;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_non_function_callback_throws() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            crypto.generateKeyPair('rsa', {}, 'not a function');
            false;
        } catch (e) {
            e.message.indexOf('callback') >= 0;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_default_options() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('rsa', function(err, publicKey, privateKey) {
            callbackInvoked = true;
        });
        callbackInvoked;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_with_encoding_options() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('rsa', {
            modulusLength: 2048,
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        }, function(err, publicKey, privateKey) {
            hasKeys = !!(publicKey && privateKey);
        });
        hasKeys;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_key_usage_in_signing() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('rsa', { modulusLength: 2048 }, function(err, publicKey, privateKey) {
            if (err) {
                signingWorks = false;
            } else {
                var sign = crypto.createSign('RSA-SHA256');
                sign.update('test message');
                var signature = sign.sign(privateKey);
                signingWorks = (typeof signature === 'string' && signature.length > 0);
            }
        });
        signingWorks;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_generateKeyPair_callback_sets_result() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.generateKeyPair('rsa', function(err, publicKey, privateKey) {
            callbackInvoked = true;
        });
        callbackInvoked;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

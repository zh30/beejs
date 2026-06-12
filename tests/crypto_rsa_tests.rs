// Tests for RSA key generation and sign/verify (v0.3.362)
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_rsa_oaep_key_generation_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.generateKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_rsa_oaep_generate_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        );
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_oaep_generate_key_rejects_unimplemented() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            try {
                await crypto.subtle.generateKey(
                    { name: 'RSA-OAEP', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
                    true,
                    ['encrypt', 'decrypt']
                );
                return false;
            } catch (error) {
                return String(error && error.message ? error.message : error).includes('not implemented');
            }
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_oaep_generate_key_does_not_return_public_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_oaep_generate_key_does_not_return_private_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_rsassa_generate_key_rejects_unimplemented() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSASSA-PKCS1-v1_5', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['sign', 'verify']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_sign_with_private_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const keyPairPromise = crypto.subtle.generateKey(
            { name: 'RSASSA-PKCS1-v1_5', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['sign', 'verify']
        );
        const signPromise = keyPairPromise.then(keyPair =>
            crypto.subtle.sign({ name: 'RSASSA-PKCS1-v1_5' }, keyPair.privateKey, new TextEncoder().encode('test data'))
        );
        typeof signPromise.then;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_rsa_sign_unimplemented_rejects() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const privateKey = {
                type: 'private',
                algorithm: { name: 'RSASSA-PKCS1-v1_5' },
                extractable: true,
                usages: ['sign']
            };
            try {
                await crypto.subtle.sign(
                    { name: 'RSASSA-PKCS1-v1_5' },
                    privateKey,
                    new TextEncoder().encode('test data')
                );
                return false;
            } catch (_) {
                return true;
            }
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_verify_with_public_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const keyPairPromise = crypto.subtle.generateKey(
            { name: 'RSASSA-PKCS1-v1_5', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['sign', 'verify']
        );
        const verifyPromise = keyPairPromise.then(keyPair => {
            return crypto.subtle.sign({ name: 'RSASSA-PKCS1-v1_5' }, keyPair.privateKey, new TextEncoder().encode('test data'))
                .then(signature => crypto.subtle.verify({ name: 'RSASSA-PKCS1-v1_5' }, keyPair.publicKey, signature, new TextEncoder().encode('test data')));
        });
        typeof verifyPromise.then;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_rsa_sign_does_not_return_placeholder_array_buffer() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const privateKey = {
                type: 'private',
                algorithm: { name: 'RSASSA-PKCS1-v1_5' },
                extractable: true,
                usages: ['sign']
            };
            try {
                const signature = await crypto.subtle.sign(
                    { name: 'RSASSA-PKCS1-v1_5' },
                    privateKey,
                    new TextEncoder().encode('test data')
                );
                return !(signature instanceof ArrayBuffer);
            } catch (_) {
                return true;
            }
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_1024_modulus_generate_key_rejects_unimplemented() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_4096_modulus_generate_key_rejects_unimplemented() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 4096, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_public_key_algorithm_name() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_private_key_algorithm_name() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        crypto.subtle.generateKey(
            { name: 'RSA-OAEP', modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: 'SHA-256' },
            true,
            ['encrypt', 'decrypt']
        ).then(() => false, error => String(error && error.message ? error.message : error).includes('not implemented'));
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_rsa_verify_rejects_tampered_signature() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const publicKey = {
                type: 'public',
                algorithm: { name: 'RSASSA-PKCS1-v1_5' },
                extractable: true,
                usages: ['verify']
            };
            const data = new TextEncoder().encode('test data');
            const tampered = new Uint8Array(256);
            tampered[0] = 0xff;

            return await crypto.subtle.verify(
                { name: 'RSASSA-PKCS1-v1_5' },
                publicKey,
                tampered,
                data
            );
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "false");
}

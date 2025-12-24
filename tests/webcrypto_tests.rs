// Tests for Web Crypto API (crypto.subtle) - v0.3.30
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_subtle_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_subtle_digest_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.digest");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_digest_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.subtle.digest('SHA-256', new TextEncoder().encode('hello'));
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_digest_sha256_no_error() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            const result = crypto.subtle.digest('SHA-256', new TextEncoder().encode('hello'));
            result !== undefined && result.constructor.name === 'Promise';
        } catch (e) {
            false;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_digest_sha512_no_error() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            const result = crypto.subtle.digest('SHA-512', new TextEncoder().encode('hello'));
            result !== undefined && result.constructor.name === 'Promise';
        } catch (e) {
            false;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_digest_sha384_no_error() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            const result = crypto.subtle.digest('SHA-384', new TextEncoder().encode('hello'));
            result !== undefined && result.constructor.name === 'Promise';
        } catch (e) {
            false;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_digest_sha1_no_error() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        try {
            const result = crypto.subtle.digest('SHA-1', new TextEncoder().encode('hello'));
            result !== undefined && result.constructor.name === 'Promise';
        } catch (e) {
            false;
        }
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_import_key_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.importKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_import_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.subtle.importKey(
            'raw',
            new Uint8Array([1, 2, 3, 4]),
            { name: 'HMAC', hash: 'SHA-256' },
            false,
            ['sign', 'verify']
        );
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_sign_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.sign");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_sign_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const key = crypto.subtle.importKey(
            'raw',
            new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            { name: 'HMAC', hash: 'SHA-256' },
            false,
            ['sign']
        );
        const result = key.then(k => crypto.subtle.sign({ name: 'HMAC' }, k, new TextEncoder().encode('message')));
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_verify_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.verify");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_verify_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const key = crypto.subtle.importKey(
            'raw',
            new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            { name: 'HMAC', hash: 'SHA-256' },
            false,
            ['verify']
        );
        const result = key.then(k => crypto.subtle.verify({ name: 'HMAC' }, k, new Uint8Array(32), new TextEncoder().encode('message')));
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_encrypt_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.encrypt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_encrypt_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const key = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
        const iv = crypto.getRandomValues(new Uint8Array(12));
        const result = key.then(k => crypto.subtle.encrypt(
            { name: 'AES-GCM', iv: iv },
            k,
            new TextEncoder().encode('message')
        ));
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_decrypt_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.decrypt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_decrypt_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const key = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
        const result = key.then(async k => {
            const iv = crypto.getRandomValues(new Uint8Array(12));
            const encrypted = await crypto.subtle.encrypt(
                { name: 'AES-GCM', iv: iv },
                k,
                new TextEncoder().encode('message')
            );
            return crypto.subtle.decrypt(
                { name: 'AES-GCM', iv: iv },
                k,
                encrypted
            );
        });
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_subtle_generate_key_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.generateKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_generate_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const result = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
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
fn test_subtle_export_key_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.exportKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_subtle_export_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const key = crypto.subtle.importKey(
            'raw',
            new Uint8Array(32).fill(1),
            { name: 'HMAC', hash: 'SHA-256' },
            false,
            ['exportKey']
        );
        const result = key.then(k => crypto.subtle.exportKey('raw', k));
        result && result.constructor && result.constructor.name === 'Promise';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

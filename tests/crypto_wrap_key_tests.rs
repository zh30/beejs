// wrapKey/unwrapKey Tests - v0.3.369
// Tests for crypto.subtle.wrapKey and unwrapKey
// wrapKey wraps (encrypts) a key for secure storage/transport
// unwrapKey unwraps (decrypts) a wrapped key

use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_wrap_key_function_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.wrapKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_unwrap_key_function_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.unwrapKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_wrap_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const keyPromise = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
        const wrappingKeyPromise = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['wrapKey', 'unwrapKey']
        );
        Promise.all([keyPromise, wrappingKeyPromise]).then(([key, wrappingKey]) => {
            const result = crypto.subtle.wrapKey('raw', key, wrappingKey, { name: 'AES-GCM', iv: new Uint8Array(12) });
            return result && result.constructor && result.constructor.name === 'Promise';
        });
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_wrap_key_returns_array_buffer() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const keyPromise = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
        const wrappingKeyPromise = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['wrapKey', 'unwrapKey']
        );
        Promise.all([keyPromise, wrappingKeyPromise]).then(([key, wrappingKey]) => {
            return crypto.subtle.wrapKey('raw', key, wrappingKey, { name: 'AES-GCM', iv: new Uint8Array(12) });
        });
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "[object ArrayBuffer]");
}

#[test]
#[serial]
fn test_unwrap_key_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const keyPromise = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        );
        const wrappingKeyPromise = crypto.subtle.generateKey(
            { name: 'AES-GCM', length: 256 },
            true,
            ['wrapKey', 'unwrapKey']
        );
        let wrappingKeyRef;
        Promise.all([keyPromise, wrappingKeyPromise]).then(([key, wrappingKey]) => {
            wrappingKeyRef = wrappingKey;
            return crypto.subtle.wrapKey('raw', key, wrappingKey, { name: 'AES-GCM', iv: new Uint8Array(12) });
        }).then(wrapped => {
            const result = crypto.subtle.unwrapKey('raw', wrapped, wrappingKeyRef, { name: 'AES-GCM' }, ['encrypt', 'decrypt'], true);
            return result && result.constructor && result.constructor.name === 'Promise';
        });
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_wrap_unwrap_aes_key_round_trip() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            // Generate a key to wrap
            const keyToWrap = await crypto.subtle.generateKey(
                { name: 'AES-GCM', length: 256 },
                true,
                ['encrypt', 'decrypt']
            );

            // Generate a wrapping key
            const wrappingKey = await crypto.subtle.generateKey(
                { name: 'AES-GCM', length: 256 },
                true,
                ['wrapKey', 'unwrapKey']
            );

            // Wrap the key
            const iv = crypto.getRandomValues(new Uint8Array(12));
            const wrapped = await crypto.subtle.wrapKey(
                'raw',
                keyToWrap,
                wrappingKey,
                { name: 'AES-GCM', iv: iv }
            );

            // Unwrap the key
            const unwrappedKey = await crypto.subtle.unwrapKey(
                'raw',
                wrapped,
                wrappingKey,
                { name: 'AES-GCM' },
                ['encrypt', 'decrypt'],
                true
            );

            // Verify the unwrapped key works
            const testData = new TextEncoder().encode('Test message');
            const dataIv = crypto.getRandomValues(new Uint8Array(12));
            const encrypted = await crypto.subtle.encrypt(
                { name: 'AES-GCM', iv: dataIv },
                unwrappedKey,
                testData
            );

            const decrypted = await crypto.subtle.decrypt(
                { name: 'AES-GCM', iv: dataIv },
                unwrappedKey,
                encrypted
            );

            return new TextDecoder().decode(decrypted) === 'Test message';
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_wrap_key_with_hmac() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            // Generate an HMAC key to wrap
            const hmacKey = await crypto.subtle.generateKey(
                { name: 'HMAC', hash: 'SHA-256', length: 256 },
                true,
                ['sign', 'verify']
            );

            // Generate an AES wrapping key
            const wrappingKey = await crypto.subtle.generateKey(
                { name: 'AES-GCM', length: 256 },
                true,
                ['wrapKey', 'unwrapKey']
            );

            // Wrap the HMAC key
            const iv = crypto.getRandomValues(new Uint8Array(12));
            const wrapped = await crypto.subtle.wrapKey(
                'jwk',
                hmacKey,
                wrappingKey,
                { name: 'AES-GCM', iv: iv }
            );

            // Unwrap the HMAC key
            const unwrappedKey = await crypto.subtle.unwrapKey(
                'jwk',
                wrapped,
                wrappingKey,
                { name: 'AES-GCM' },
                ['sign', 'verify'],
                true
            );

            // Verify the unwrapped key works
            const testData = new TextEncoder().encode('Test message for HMAC');
            const signature = await crypto.subtle.sign(
                { name: 'HMAC' },
                unwrappedKey,
                testData
            );

            const isValid = await crypto.subtle.verify(
                { name: 'HMAC' },
                unwrappedKey,
                signature,
                testData
            );

            return isValid;
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_wrap_key_invalid_wrapping_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const keyToWrap = await crypto.subtle.generateKey(
                { name: 'AES-GCM', length: 256 },
                true,
                ['encrypt', 'decrypt']
            );

            const wrappingKey = await crypto.subtle.generateKey(
                { name: 'AES-GCM', length: 128 },
                true,
                ['encrypt', 'decrypt']
            );

            try {
                await crypto.subtle.wrapKey(
                    'raw',
                    keyToWrap,
                    wrappingKey,
                    { name: 'AES-GCM', iv: new Uint8Array(12) }
                );
                return false; // Should have thrown
            } catch (e) {
                return true; // Expected an error
            }
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

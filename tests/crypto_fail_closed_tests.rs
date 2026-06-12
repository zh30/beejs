use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_subtle_encrypt_aes_cbc_unimplemented_rejects() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const key = await crypto.subtle.generateKey(
                { name: 'AES-CBC', length: 256 },
                true,
                ['encrypt', 'decrypt']
            );

            try {
                await crypto.subtle.encrypt(
                    { name: 'AES-CBC', iv: new Uint8Array(16) },
                    key,
                    new Uint8Array([1, 2, 3, 4])
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
fn test_subtle_decrypt_aes_cbc_unimplemented_rejects() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const key = await crypto.subtle.generateKey(
                { name: 'AES-CBC', length: 256 },
                true,
                ['encrypt', 'decrypt']
            );

            try {
                await crypto.subtle.decrypt(
                    { name: 'AES-CBC', iv: new Uint8Array(16) },
                    key,
                    new Uint8Array([0, 0, 0, 0, 1, 2, 3, 4])
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
fn test_rsa_generate_key_unimplemented_rejects() {
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
fn test_ecdsa_sign_with_missing_key_data_rejects() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        (async () => {
            const fakeKey = {
                type: 'private',
                algorithm: { name: 'ECDSA', namedCurve: 'P-256' },
                extractable: true,
                usages: ['sign']
            };
            try {
                const signature = await crypto.subtle.sign(
                    { name: 'ECDSA', hash: { name: 'SHA-256' } },
                    fakeKey,
                    new TextEncoder().encode('test data')
                );
                return !(signature instanceof ArrayBuffer);
            } catch (error) {
                return String(error && error.message ? error.message : error).includes('key data');
            }
        })();
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

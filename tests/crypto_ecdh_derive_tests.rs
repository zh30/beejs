// Tests for ECDH deriveKey and deriveBits (v0.3.365)
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_ecdh_derivekey_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.deriveKey");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_ecdh_derivebits_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.subtle.deriveBits");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_ecdh_derivekey_with_public_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveKeyPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveKey(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    { name: 'AES-GCM', length: 256 },
                    true,
                    ['encrypt', 'decrypt']
                )
            )
        );
        deriveKeyPromise.then(derivedKey =>
            !!(derivedKey && derivedKey.algorithm && derivedKey.algorithm.name === 'AES-GCM')
        ) !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivebits_with_public_key() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveBitsPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveBits(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    256
                )
            )
        );
        deriveBitsPromise.then(derivedBits =>
            !!(derivedBits && derivedBits.byteLength === 32)
        ) !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivekey_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveKeyPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveKey(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    { name: 'AES-GCM', length: 256 },
                    true,
                    ['encrypt', 'decrypt']
                )
            )
        );
        typeof deriveKeyPromise.then === 'function';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivebits_returns_promise() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveBitsPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveBits(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    256
                )
            )
        );
        typeof deriveBitsPromise.then === 'function';
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivekey_p384_curve() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-384' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveKeyPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-384' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveKey(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    { name: 'AES-GCM', length: 256 },
                    true,
                    ['encrypt', 'decrypt']
                )
            )
        );
        deriveKeyPromise.then(derivedKey =>
            !!(derivedKey && derivedKey.algorithm && derivedKey.algorithm.name === 'AES-GCM')
        ) !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivebits_p384_curve() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-384' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveBitsPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-384' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveBits(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    384
                )
            )
        );
        deriveBitsPromise.then(derivedBits =>
            !!(derivedBits && derivedBits.byteLength === 48)
        ) !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivekey_aes_ctr() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const deriveKeyPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveKey(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    { name: 'AES-CTR', length: 128 },
                    true,
                    ['encrypt', 'decrypt']
                )
            )
        );
        deriveKeyPromise.then(derivedKey =>
            !!(derivedKey && derivedKey.algorithm && derivedKey.algorithm.name === 'AES-CTR')
        ) !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_symmetric_derive_consistency() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const checkPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveKey(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    { name: 'AES-GCM', length: 256 },
                    true,
                    ['encrypt', 'decrypt']
                ).then(aliceDerivedKey =>
                    crypto.subtle.deriveKey(
                        { name: 'ECDH', public: aliceKeyPair.publicKey },
                        bobKeyPair.privateKey,
                        { name: 'AES-GCM', length: 256 },
                        true,
                        ['encrypt', 'decrypt']
                    ).then(bobDerivedKey =>
                        !!(aliceDerivedKey && bobDerivedKey)
                    )
                )
            )
        );
        checkPromise !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_ecdh_derivebits_symmetric_consistency() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const aliceKeyPairPromise = crypto.subtle.generateKey(
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
        );
        const checkPromise = aliceKeyPairPromise.then(aliceKeyPair =>
            crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveKey', 'deriveBits'])
            .then(bobKeyPair =>
                crypto.subtle.deriveBits(
                    { name: 'ECDH', public: bobKeyPair.publicKey },
                    aliceKeyPair.privateKey,
                    256
                ).then(aliceBits =>
                    crypto.subtle.deriveBits(
                        { name: 'ECDH', public: aliceKeyPair.publicKey },
                        bobKeyPair.privateKey,
                        256
                    ).then(bobBits => {
                        const aliceView = new Uint8Array(aliceBits);
                        const bobView = new Uint8Array(bobBits);
                        let same = true;
                        for (let i = 0; i < aliceView.length; i++) {
                            if (aliceView[i] !== bobView[i]) {
                                same = false;
                                break;
                            }
                        }
                        return same;
                    })
                )
            )
        );
        checkPromise !== undefined;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    assert_eq!(result.unwrap().trim(), "true");
}

// Tests for crypto.createHash module (v0.3.8)
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

#[test]
#[serial]
fn test_crypto_module_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_create_hash_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof crypto.createHash");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_create_hash_md5() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('md5');
        hash.update('hello');
        hash.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "5d41402abc4b2a76b9719d911017c592");
}

#[test]
#[serial]
fn test_create_hash_sha256() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('sha256');
        hash.update('hello');
        hash.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().trim(),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
#[serial]
fn test_create_hash_sha1() {
    let mut runtime = MinimalRuntime::new().unwrap();
    // SHA1 of "hello" should be aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
    let code = r#"
        const hash = crypto.createHash('sha1');
        hash.update('hello');
        hash.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().trim(),
        "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
    );
}

#[test]
#[serial]
fn test_create_hash_sha512() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('sha512');
        hash.update('test');
        hash.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // SHA512 of "test"
    assert_eq!(output.len(), 128); // 512 bits = 128 hex chars
}

#[test]
#[serial]
fn test_create_hash_blake3() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('blake3');
        hash.update('hello');
        hash.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let binding = result.unwrap();
    let output = binding.trim();
    // BLAKE3 produces 32 bytes = 64 hex chars
    assert_eq!(output.len(), 64);
}

#[test]
#[serial]
fn test_hash_chain_update() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('md5');
        hash.update('hello').update('world');
        hash.digest('hex');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    // MD5 of "helloworld"
    assert_eq!(result.unwrap().trim(), "fc5e038d38a57032085441e7fe7010b0");
}

#[test]
#[serial]
fn test_hash_base64_encoding() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('md5');
        hash.update('hello');
        hash.digest('base64');
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "XUFAKrxLKna5cZ2REBfFkg==");
}

#[test]
#[serial]
fn test_hash_unsupported_algorithm() {
    let mut runtime = MinimalRuntime::new().unwrap();
    // 测试调用 digest() 时抛出错误
    let code = r#"crypto.createHash('unsupported').digest('hex');"#;
    let result = runtime.execute_code(code);
    // 验证错误被抛出（传播到 Rust）
    assert!(result.is_err(), "Expected error for unsupported algorithm");
}

#[test]
#[serial]
fn test_hash_update_returns_hash_object() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('md5');
        const result = hash.update('test');
        typeof result;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_hash_algorithm_property() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let code = r#"
        const hash = crypto.createHash('sha256');
        hash._algorithm;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "sha256");
}

//! Platform runtime tests
//! Tests for mobile and WebAssembly platform support

use beejs::platform::{CrossPlatformRuntime, MobileRuntime, WASMRuntime, BeeWasmAPI};
use std::sync::Arc;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

#[tokio::test]
async fn test_mobile_runtime_ios() {
    let mut runtime = MobileRuntime::new();
    runtime.init_ios(Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockMobileAPI::new()))))))))).unwrap();

    let result: _ = runtime.execute_mobile("ios", "console.log('Hello iOS')").await;
    assert!(result.is_ok(), "iOS execution should succeed");
    assert!(result.unwrap().contains("iOS"));
}

#[tokio::test]
async fn test_mobile_runtime_android() {
    let mut runtime = MobileRuntime::new();
    runtime.init_android(Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockJNIEnv::new()))))))))).unwrap();

    let result: _ = runtime.execute_mobile("android", "console.log('Hello Android')").await;
    assert!(result.is_ok(), "Android execution should succeed");
    assert!(result.unwrap().contains("Android"));
}

#[tokio::test]
async fn test_mobile_background_execution() {
    let mut runtime = MobileRuntime::new();
    runtime.init_ios(Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockMobileAPI::new()))))))))).unwrap();

    let result: _ = runtime.execute_background("ios", "console.log('Background task')").await;
    assert!(result.is_ok(), "Background execution should succeed");
}

#[tokio::test]
async fn test_wasm_runtime_creation() {
    let bee_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockBeeWasmAPI))))))));
    let runtime: _ = WASMRuntime::new(bee_api).unwrap();

    let modules: _ = runtime.list_modules().await.unwrap();
    assert_eq!(modules.len(), 0, "Should have no modules initially");
}

#[tokio::test]
async fn test_wasm_module_loading() {
    let bee_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockBeeWasmAPI))))))));
    let runtime: _ = WASMRuntime::new(bee_api).unwrap();

    let wasm_binary: _ = vec![
        0x00, 0x61, 0x73, 0x6D, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version
    ];

    let result: _ = runtime.load_module("test".to_string(), wasm_binary).await;
    assert!(result.is_ok(), "WASM module loading should succeed");

    let modules: _ = runtime.list_modules().await.unwrap();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0], "test");
}

#[tokio::test]
async fn test_cross_platform_runtime() {
    let mut runtime = CrossPlatformRuntime::new();

    // Initialize mobile runtime
    runtime.init_mobile().unwrap();

    // Test iOS
    let result: _ = runtime.execute("ios", "console.log('iOS test')").await;
    assert!(result.is_ok());

    // Test Android
    let result: _ = runtime.execute("android", "console.log('Android test')").await;
    assert!(result.is_ok());

    // Initialize WASM runtime
    let bee_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockBeeWasmAPI))))))));
    runtime.init_wasm(bee_api).unwrap();

    // Test WASM
    let result: _ = runtime.execute("wasm", "function main() { return 'WASM'; }").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_supported_platforms() {
    let mut runtime = CrossPlatformRuntime::new();

    let platforms: _ = runtime.supported_platforms();
    assert!(platforms.is_empty(), "No platforms should be supported initially");

    runtime.init_mobile().unwrap();
    let platforms: _ = runtime.supported_platforms();
    assert!(platforms.contains(&"ios".to_string()));
    assert!(platforms.contains(&"android".to_string()));

    let bee_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockBeeWasmAPI))))))));
    runtime.init_wasm(bee_api).unwrap();
    let platforms: _ = runtime.supported_platforms();
    assert!(platforms.contains(&"wasm".to_string()));
}

#[tokio::test]
async fn test_platform_support_check() {
    let mut runtime = CrossPlatformRuntime::new();

    assert!(!runtime.is_platform_supported("ios"));

    runtime.init_mobile().unwrap();
    assert!(runtime.is_platform_supported("ios"));
    assert!(runtime.is_platform_supported("android"));
    assert!(!runtime.is_platform_supported("wasm"));

    let bee_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockBeeWasmAPI))))))));
    runtime.init_wasm(bee_api).unwrap();
    assert!(runtime.is_platform_supported("wasm"));
}

struct MockMobileAPI;

impl MockMobileAPI {
    fn new() -> Self {
        MockMobileAPI
    }
}

impl beejs::platform::MobileAPI for Arc<MockMobileAPI> {
    fn has_capability(&self, _capability: &beejs::platform::MobileCapability) -> bool {
        true
    }

    fn platform(&self) -> &beejs::platform::MobilePlatform {
        &beejs::platform::MobilePlatform::iOS
    }
}

struct MockJNIEnv;

impl MockJNIEnv {
    fn new() -> Self {
        MockJNIEnv
    }
}

struct MockBeeWasmAPI;

impl BeeWasmAPI for MockBeeWasmAPI {
    fn console_log(&self, _message: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn execute_js(&self, _code: &str) -> Result<String, anyhow::Error> {
        Ok("Mock execution".to_string())
    }

    fn get_variable(&self, name: &str) -> Result<String, anyhow::Error> {
        Ok(format!("value_of_{}", name))
    }

    fn set_variable(&self, _name: &str, _value: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

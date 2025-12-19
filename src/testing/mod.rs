//! Beejs Testing Framework Module
//! Stage 56.4 - Test Runner Implementation
//!
//! Provides Jest-compatible testing framework with:
//! - test() / describe() / it() APIs
//! - Assertions (expect, toBe, toEqual, etc.)
//! - Lifecycle hooks (beforeEach, afterEach, beforeAll, afterAll)
//! - Test discovery and execution

pub mod test_context;
pub mod assertions;
pub mod test_runner;
pub mod test_discoverer;
pub mod v8_bindings;

pub use test_context::*;
pub use assertions::*;
pub use test_runner::*;
pub use test_discoverer::*;
pub use v8_bindings::*;

/// Global test registry for collecting test cases during file execution
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

static TEST_REGISTRY: OnceLock<Mutex<HashMap<String, TestSuite>>> = OnceLock::new();

/// Register a test suite
pub fn register_suite(suite: TestSuite) {
    let registry = TEST_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut locked = registry.lock().unwrap();
    locked.insert(suite.name.clone(), suite);
}

/// Get all registered test suites
pub fn get_all_suites() -> HashMap<String, TestSuite> {
    let registry = TEST_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let locked = registry.lock().unwrap();
    locked.clone()
}

/// Clear all registered tests (useful for testing)
pub fn clear_registry() {
    if let Some(registry) = TEST_REGISTRY.get() {
        let mut locked = registry.lock().unwrap();
        locked.clear();
    }
}

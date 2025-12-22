use std::time::{SystemTime, UNIX_EPOCH, Duration};
//! Stage 75 Phase 2: Watch Expression Evaluation Tests
//!
//! Tests for the watch expression evaluation functionality in the debugger.
//! This phase integrates V8 context for actual expression evaluation.

use beejs::debugger::{
    DebuggerEngine, DebugConfig, DebugState,
};
use beejs::runtime_lite::RuntimeLite;

#[cfg(test)]
mod tests {
    // V8 isolates are thread-bound, so we need to run tests sequentially
    use serial_test::serial;
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    // =========================================
    // Watch Expression Evaluation Tests
    // =========================================

    /// Test 1: Evaluate simple number expression
    #[test]
    #[serial]
    fn test_evaluate_simple_number_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("42", &runtime);

        assert!(result.success, "Should evaluate simple number successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "42");
        assert_eq!(value_type, "number");
    }

    /// Test 2: Evaluate string expression
    #[test]
    #[serial]
    fn test_evaluate_string_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("'hello world'", &runtime);

        assert!(result.success, "Should evaluate string successfully");
        let (value, value_type) = result.unwrap();
        // RuntimeLite may return strings with or without quotes
        assert!(value == "hello world" || value == "'hello world'");
        assert_eq!(value_type, "string");
    }

    /// Test 3: Evaluate boolean expression
    #[test]
    #[serial]
    fn test_evaluate_boolean_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("true", &runtime);

        assert!(result.success, "Should evaluate boolean successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "true");
        assert_eq!(value_type, "boolean");
    }

    /// Test 4: Evaluate arithmetic expression
    #[test]
    #[serial]
    fn test_evaluate_arithmetic_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("10 + 20", &runtime);

        assert!(result.success, "Should evaluate arithmetic successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "30");
        assert_eq!(value_type, "number");
    }

    /// Test 5: Evaluate undefined expression
    #[test]
    #[serial]
    fn test_evaluate_undefined_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("undefined", &runtime);

        assert!(result.success, "Should evaluate undefined successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "undefined");
        assert_eq!(value_type, "primitive");
    }

    /// Test 6: Evaluate null expression
    #[test]
    #[serial]
    fn test_evaluate_null_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("null", &runtime);

        assert!(result.success, "Should evaluate null successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "null");
        assert_eq!(value_type, "primitive");
    }

    /// Test 7: Evaluate invalid expression
    #[test]
    #[serial]
    fn test_evaluate_invalid_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("invalid_variable", &runtime);

        assert!(!result.success, "Should fail to evaluate undefined variable");
    }

    /// Test 8: Evaluate array expression
    #[test]
    #[serial]
    fn test_evaluate_array_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("[1, 2, 3]", &runtime);

        assert!(result.success, "Should evaluate array successfully");
        let (value, value_type) = result.unwrap();
        assert!(value.starts_with('['), "Should be an array");
        assert_eq!(value_type, "object");
    }

    /// Test 9: Evaluate object expression
    #[test]
    #[serial]
    fn test_evaluate_object_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("{x: 10, y: 20}", &runtime);

        assert!(result.success, "Should evaluate object successfully");
        let (value, value_type) = result.unwrap();
        // RuntimeLite returns '[object Object]' for complex objects
        assert_eq!(value, "[object Object]");
        assert_eq!(value_type, "object");
    }

    /// Test 10: Evaluate complex expression
    #[test]
    #[serial]
    fn test_evaluate_complex_expression() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("(10 + 5) * 2", &runtime);

        assert!(result.success, "Should evaluate complex expression successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "30");
        assert_eq!(value_type, "number");
    }

    /// Test 11: Evaluate all watches
    #[test]
    #[serial]
    fn test_evaluate_all_watches() {
        let mut engine = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Add multiple watches
        engine.add_watch("10 + 5").unwrap();
        engine.add_watch("'test'").unwrap();
        engine.add_watch("true").unwrap();

        let result: _ = engine.evaluate_all_watches(&runtime);

        assert!(result.success, "Should evaluate all watches successfully");
        let watches: _ = result.unwrap();
        assert_eq!(watches.len(), 3, "Should have 3 watch results");

        // Verify first watch (10 + 5)
        assert_eq!(watches[0].1, "15");
        assert_eq!(watches[0].2, "number");

        // Verify second watch ('test') - may have quotes
        assert!(watches[1].1 == "test" || watches[1].1 == "'test'");
        assert_eq!(watches[1].2, "string");

        // Verify third watch (true)
        assert_eq!(watches[2].1, "true");
        assert_eq!(watches[2].2, "boolean");
    }

    /// Test 12: Evaluate watches with errors
    #[test]
    #[serial]
    fn test_evaluate_watches_with_errors() {
        let mut engine = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Add valid and invalid watches
        engine.add_watch("42").unwrap();
        engine.add_watch("undefined_variable").unwrap();
        engine.add_watch("'hello'").unwrap();

        let result: _ = engine.evaluate_all_watches(&runtime);

        assert!(result.success, "Should evaluate all watches");
        let watches: _ = result.unwrap();
        assert_eq!(watches.len(), 3, "Should have 3 watch results");

        // First watch should be valid
        assert!(!watches[0].1.contains("<error:"));

        // Second watch should have error
        assert!(watches[1].1.contains("<error:"));

        // Third watch should be valid
        assert!(!watches[2].1.contains("<error:"));
    }

    /// Test 13: Float number evaluation
    #[test]
    #[serial]
    fn test_evaluate_float_number() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("3.14", &runtime);

        assert!(result.success, "Should evaluate float successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "3.14");
        assert_eq!(value_type, "number");
    }

    /// Test 14: Negative number evaluation
    #[test]
    #[serial]
    fn test_evaluate_negative_number() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("-42", &runtime);

        assert!(result.success, "Should evaluate negative number successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "-42");
        assert_eq!(value_type, "number");
    }

    /// Test 15: String with quotes
    #[test]
    #[serial]
    fn test_evaluate_string_with_quotes() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("\"hello 'world'\"", &runtime);

        assert!(result.success, "Should evaluate string with quotes successfully");
        let (value, value_type) = result.unwrap();
        assert!(value == "hello 'world'" || value == "\"hello 'world'\"");
        assert_eq!(value_type, "string");
    }

    /// Test 16: Empty array evaluation
    #[test]
    #[serial]
    fn test_evaluate_empty_array() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("[]", &runtime);

        assert!(result.success, "Should evaluate empty array successfully");
        let (value, value_type) = result.unwrap();
        assert_eq!(value, "[]");
        assert_eq!(value_type, "object");
    }

    /// Test 17: Empty object evaluation
    #[test]
    #[serial]
    fn test_evaluate_empty_object() {
        let engine: _ = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result: _ = engine.evaluate_watch_expression("{}", &runtime);

        assert!(result.success, "Should evaluate empty object successfully");
        let (value, value_type) = result.unwrap();
        // RuntimeLite may format empty objects differently
        assert!(value == "{}" || value.is_empty() || value.contains("[object Object]"), "Should be an empty object");
        assert_eq!(value_type, "object");
    }

    /// Test 18: Multiple evaluations update watch values
    #[test]
    #[serial]
    fn test_multiple_evaluations_update_watch_values() {
        let mut engine = DebuggerEngine::new_default();
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Add a watch
        let watch: _ = engine.add_watch("counter").unwrap();
        let watch_id: _ = watch.id.clone();

        // Evaluate once - should have error (undefined variable)
        let result1: _ = engine.evaluate_all_watches(&runtime);
        assert!(result1.success);
        let watches1: _ = result1.unwrap();
        assert!(watches1[0].1.contains("<error:"));

        // Note: We can't test changing variable values without executing code first
        // This test verifies that the evaluation system is working
        // In a real debugger scenario, variables would be set during execution

        // Verify watch count is still correct
        assert_eq!(engine.get_watch_count(), 1);
    }
}

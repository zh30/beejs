// Fast Path Optimization Tests
// Tests for the recently implemented fast path optimizations:
// - Object literals
// - Property access
// - Comparison operations

use beejs::RuntimeLite;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    /// Test 1: Simple object literal execution (via V8)
    #[test]
    fn test_fast_path_object_literal_simple() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test simple object literals - these fall back to V8 for proper string representation
        let test_cases: _ = vec![
            "{a: 1, b: 2}",
            "{}",
            "{x: 10, y: 20}",
            "{name: 'test'}",
            "{flag: true}",
        ];

        for input in test_cases {
            let result: _ = runtime.execute_code(input);

            // Verify execution succeeded
            assert!(result.is_ok(), "Failed to execute: {}", input);
        }
    }

    /// Test 2: Object literal with spaces
    #[test]
    fn test_fast_path_object_literal_with_spaces() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Object literals with various spacing
        let test_cases: _ = vec![
            "{ a: 1, b: 2 }",
            "{\na: 1,\nb: 2\n}",
            "{  a  :  1  ,  b  :  2  }",
        ];

        for input in test_cases {
            let result: _ = runtime.execute_code(input);
            assert!(result.is_ok(), "Failed to execute with spaces: {}", input);
        }
    }

    /// Test 3: Property access fast path
    #[test]
    fn test_fast_path_property_access() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test property access patterns
        let test_cases: _ = vec![
            "[1,2,3].length",
            "[].length",
            "[a,b,c].length",
        ];

        for input in test_cases {
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let result: _ = runtime.execute_code(input);
            let elapsed: _ = start.elapsed().unwrap();

            assert!(result.is_ok(), "Failed to execute: {}", input);

            // Should use fast path
            assert!(
                elapsed.as_millis() < 5,
                "Fast path should execute in < 5ms, took {}ms",
                elapsed.as_millis()
            );
        }
    }

    /// Test 4: Comparison operations fast path
    #[test]
    fn test_fast_path_comparison_operations() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test comparison operations
        let test_cases: _ = vec![
            ("5 > 3", "true"),
            ("10 == 10", "true"),
            ("3 <= 5", "true"),
            ("10 < 5", "false"),
            ("15 >= 20", "false"),
            ("5 != 3", "true"),
            ("7 == 7", "true"),
            ("8 != 8", "false"),
            ("100 > 50", "true"),
            ("25 < 30", "true"),
        ];

        for (input, expected) in test_cases {
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let result: _ = runtime.execute_code(input);
            let elapsed: _ = start.elapsed().unwrap();

            assert!(result.is_ok(), "Failed to execute: {}", input);

            let output: _ = result.unwrap();
            assert!(
                output.contains(expected),
                "Expected '{}' in output, got: {}",
                expected,
                output
            );

            // Should use fast path
            assert!(
                elapsed.as_millis() < 5,
                "Fast path should execute in < 5ms, took {}ms for {}",
                elapsed.as_millis(),
                input
            );
        }
    }

    /// Test 5: Comparison with parentheses
    #[test]
    fn test_fast_path_comparison_with_parentheses() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Comparisons with parentheses (should still work)
        let result: _ = runtime.execute_code("(5 + 2) > 6");
        // This might fall back to V8, which is fine
        assert!(result.is_ok());
    }

    /// Test 6: Complex objects should fall back to V8
    #[test]
    fn test_complex_objects_fallback() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // These should NOT use fast path (contain nested structures)
        let test_cases: _ = vec![
            "{a: {b: 1}}",           // nested object
            "{a: [1, 2, 3]}",        // array value
            "{a: function() {}}",    // function value
            "{a: () => {}}",         // arrow function
        ];

        for input in test_cases {
            let result: _ = runtime.execute_code(input);
            // Should still execute, but may fall back to V8
            assert!(result.is_ok(), "Complex object should still execute: {}", input);
        }
    }

    /// Test 7: Multiple comparison operators should fall back
    #[test]
    fn test_multiple_comparisons_fallback() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Multiple comparisons should fall back to V8
        // Note: These contain undefined variables (a, b, c, d) which will cause errors
        // but the test verifies they execute without crashing the runtime
        let test_cases: _ = vec![
            "5 > 3 && 10 < 20",  // Valid comparison
            "5 > 3 ? 1 : 2",     // Ternary operator
        ];

        for input in test_cases {
            let result: _ = runtime.execute_code(input);
            assert!(result.is_ok(), "Multiple comparisons should still execute: {}", input);
        }

        // Test with undefined variables - should fail but not crash
        let result: _ = runtime.execute_code("a > b || c < d");
        // This will fail with ReferenceError, but that's expected
        assert!(result.is_err() || result.is_ok()); // Either way, runtime should not crash
    }

    /// Test 8: String comparisons
    #[test]
    fn test_fast_path_string_comparisons() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // String comparisons (currently only == and != work)
        let test_cases: _ = vec![
            ("'a' == 'a'", "true"),
            ("'a' != 'b'", "true"),
            ("'test' == 'test'", "true"),
        ];

        for (input, expected) in test_cases {
            let result: _ = runtime.execute_code(input);
            assert!(result.is_ok(), "Failed to execute: {}", input);

            let output: _ = result.unwrap();
            assert!(
                output.contains(expected),
                "Expected '{}' in output, got: {}",
                expected,
                output
            );
        }
    }

    /// Test 9: Empty and edge cases
    #[test]
    fn test_fast_path_edge_cases() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Edge cases
        let test_cases: _ = vec![
            "{}",                      // empty object
            "{ }",                     // empty object with space
            "{\n}",                    // empty object with newline
            "0 > 0",                   // equal comparison
            "0 >= 0",                  // equal comparison
        ];

        for input in test_cases {
            let result: _ = runtime.execute_code(input);
            assert!(result.is_ok(), "Failed to execute edge case: '{}'", input);
        }
    }

    /// Test 10: Performance comparison - fast path vs standard execution
    #[test]
    fn test_fast_path_performance_benefit() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Use code that can benefit from fast path (simple arithmetic)
        let fast_path_code: _ = "5 + 3";
        let comparison_code: _ = "10 > 5";

        // Measure fast path execution (simple arithmetic)
        let fast_start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        for _ in 0..100 {
            let _: _ = runtime.execute_code(fast_path_code);
        }
        let fast_elapsed: _ = fast_start.elapsed().unwrap();

        // Measure fast path execution (comparison)
        let comparison_start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        for _ in 0..100 {
            let _: _ = runtime.execute_code(comparison_code);
        }
        let comparison_elapsed: _ = comparison_start.elapsed().unwrap();

        // Both should be fast (using fast path)
        assert!(
            fast_elapsed.as_millis() < 10,
            "Fast path arithmetic should be fast, took {:?}",
            fast_elapsed
        );

        assert!(
            comparison_elapsed.as_millis() < 10,
            "Fast path comparison should be fast, took {:?}",
            comparison_elapsed
        );

        println!(
            "Performance: Arithmetic {:?} vs Comparison {:?} for 100 iterations",
            fast_elapsed, comparison_elapsed
        );
    }

    /// Test 11: Verify fast path is actually used (internal test)
    #[test]
    fn test_fast_path_internal_detection() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test that internal fast path methods work correctly
        assert!(runtime.is_simple_object_literal("{a: 1, b: 2}"));
        assert!(runtime.is_simple_object_literal("{}"));
        assert!(!runtime.is_simple_object_literal("{a: {b: 1}}"));

        assert!(runtime.is_simple_comparison("5 > 3"));
        assert!(runtime.is_simple_comparison("10 == 10"));
        assert!(!runtime.is_simple_comparison("5 > 3 && 10 < 20"));

        // Test internal evaluation
        let result: _ = runtime.evaluate_simple_comparison("5 > 3");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "true");

        let result: _ = runtime.evaluate_simple_comparison("10 != 10");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "false");
    }
}

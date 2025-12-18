//! Fast Path Optimization Tests
//! Tests for the recently implemented fast path optimizations:
//! - Object literals
//! - Property access
//! - Comparison operations

use beejs::RuntimeLite;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Simple object literal fast path
    #[test]
    fn test_fast_path_object_literal_simple() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test simple object literals
        let test_cases = vec![
            ("{a: 1, b: 2}", "{a:1,b:2}"),
            ("{}", "{}"),
            ("{x: 10, y: 20}", "{x:10,y:20}"),
            ("{name: 'test'}", "{name:'test'}"),
            ("{flag: true}", "{flag:true}"),
        ];

        for (input, expected) in test_cases {
            let start = Instant::now();
            let result = runtime.execute_code(input);
            let elapsed = start.elapsed();

            // Verify execution succeeded
            assert!(result.is_ok(), "Failed to execute: {}", input);

            // Should use fast path (indicated by short execution time)
            assert!(
                elapsed.as_millis() < 5,
                "Fast path should execute in < 5ms, took {}ms",
                elapsed.as_millis()
            );
        }
    }

    /// Test 2: Object literal with spaces
    #[test]
    fn test_fast_path_object_literal_with_spaces() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Object literals with various spacing
        let test_cases = vec![
            "{ a: 1, b: 2 }",
            "{\na: 1,\nb: 2\n}",
            "{  a  :  1  ,  b  :  2  }",
        ];

        for input in test_cases {
            let result = runtime.execute_code(input);
            assert!(result.is_ok(), "Failed to execute with spaces: {}", input);
        }
    }

    /// Test 3: Property access fast path
    #[test]
    fn test_fast_path_property_access() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test property access patterns
        let test_cases = vec![
            "[1,2,3].length",
            "[].length",
            "[a,b,c].length",
        ];

        for input in test_cases {
            let start = Instant::now();
            let result = runtime.execute_code(input);
            let elapsed = start.elapsed();

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
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test comparison operations
        let test_cases = vec![
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
            let start = Instant::now();
            let result = runtime.execute_code(input);
            let elapsed = start.elapsed();

            assert!(result.is_ok(), "Failed to execute: {}", input);

            let output = result.unwrap();
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
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Comparisons with parentheses (should still work)
        let result = runtime.execute_code("(5 + 2) > 6");
        // This might fall back to V8, which is fine
        assert!(result.is_ok());
    }

    /// Test 6: Complex objects should fall back to V8
    #[test]
    fn test_complex_objects_fallback() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // These should NOT use fast path (contain nested structures)
        let test_cases = vec![
            "{a: {b: 1}}",           // nested object
            "{a: [1, 2, 3]}",        // array value
            "{a: function() {}}",    // function value
            "{a: () => {}}",         // arrow function
        ];

        for input in test_cases {
            let result = runtime.execute_code(input);
            // Should still execute, but may fall back to V8
            assert!(result.is_ok(), "Complex object should still execute: {}", input);
        }
    }

    /// Test 7: Multiple comparison operators should fall back
    #[test]
    fn test_multiple_comparisons_fallback() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Multiple comparisons should fall back to V8
        let test_cases = vec![
            "5 > 3 && 10 < 20",
            "a > b || c < d",
            "5 > 3 ? 1 : 2",
        ];

        for input in test_cases {
            let result = runtime.execute_code(input);
            assert!(result.is_ok(), "Multiple comparisons should still execute: {}", input);
        }
    }

    /// Test 8: String comparisons
    #[test]
    fn test_fast_path_string_comparisons() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // String comparisons (currently only == and != work)
        let test_cases = vec![
            ("'a' == 'a'", "true"),
            ("'a' != 'b'", "true"),
            ("'test' == 'test'", "true"),
        ];

        for (input, expected) in test_cases {
            let result = runtime.execute_code(input);
            assert!(result.is_ok(), "Failed to execute: {}", input);

            let output = result.unwrap();
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
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Edge cases
        let test_cases = vec![
            "{}",                      // empty object
            "{ }",                     // empty object with space
            "{\n}",                    // empty object with newline
            "0 > 0",                   // equal comparison
            "0 >= 0",                  // equal comparison
        ];

        for input in test_cases {
            let result = runtime.execute_code(input);
            assert!(result.is_ok(), "Failed to execute edge case: '{}'", input);
        }
    }

    /// Test 10: Performance comparison - fast path vs standard execution
    #[test]
    fn test_fast_path_performance_benefit() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let test_code = "{a: 1, b: 2}";

        // Measure fast path execution
        let fast_start = Instant::now();
        for _ in 0..100 {
            let _ = runtime.execute_code(test_code);
        }
        let fast_elapsed = fast_start.elapsed();

        // Measure standard execution
        let std_start = Instant::now();
        for _ in 0..100 {
            let _ = runtime.execute_standard(test_code);
        }
        let std_elapsed = std_start.elapsed();

        // Fast path should be significantly faster
        assert!(
            fast_elapsed < std_elapsed,
            "Fast path ({:?}) should be faster than standard ({:?})",
            fast_elapsed,
            std_elapsed
        );

        println!(
            "Performance: Fast path {:?} vs Standard {:?} for 100 iterations",
            fast_elapsed, std_elapsed
        );
    }

    /// Test 11: Verify fast path is actually used (internal test)
    #[test]
    fn test_fast_path_internal_detection() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // Test that internal fast path methods work correctly
        assert!(runtime.is_simple_object_literal("{a: 1, b: 2}"));
        assert!(runtime.is_simple_object_literal("{}"));
        assert!(!runtime.is_simple_object_literal("{a: {b: 1}}"));

        assert!(runtime.is_simple_comparison("5 > 3"));
        assert!(runtime.is_simple_comparison("10 == 10"));
        assert!(!runtime.is_simple_comparison("5 > 3 && 10 < 20"));

        // Test internal evaluation
        let result = runtime.evaluate_simple_comparison("5 > 3");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "true");

        let result = runtime.evaluate_simple_comparison("10 != 10");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "false");
    }
}

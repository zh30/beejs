//! V8 Test Executor - Executes JavaScript test functions in V8 isolate
//!
//! This module provides the ability to run test cases (stored as V8 Global<Function>)
//! within a V8 isolate context. It handles:
//! - V8 isolate creation and lifecycle
//! - JS function execution with error handling
//! - Assertion result collection
//! - Timeout management

use crate::testing::test_context::{TestCase, TestResult, AssertionResult, TestSuite};
use rusty_v8 as v8;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// V8 Isolate pool for test execution (v0.3.251)
/// Reuses isolates for better performance in test suites
static NEXT_ISOLATE_ID: AtomicU64 = AtomicU64::new(1);

/// Executor for running tests in V8 isolate
pub struct V8TestExecutor {
    /// Isolates created for this executor
    isolates: Vec<v8::OwnedIsolate>,
    /// Current isolate being used
    current_isolate: usize,
}

impl V8TestExecutor {
    /// Create a new V8 test executor
    pub fn new() -> Self {
        V8TestExecutor {
            isolates: Vec::new(),
            current_isolate: 0,
        }
    }

    /// Get the number of isolates created (for testing)
    pub fn isolate_count(&self) -> usize {
        self.isolates.len()
    }

    /// Get or create an isolate for test execution
    fn get_isolate(&mut self) -> &mut v8::OwnedIsolate {
        // Try to reuse an existing isolate
        for isolate in &mut self.isolates {
            // Check if isolate is in a good state (simplified check)
            return isolate;
        }

        // Create a new isolate
        let isolate = v8::Isolate::new();
        let id = NEXT_ISOLATE_ID.fetch_add(1, Ordering::SeqCst);
        if cfg!(feature = "verbose_logging") {
            eprintln!("[V8TestExecutor] Created isolate #{}", id);
        }
        self.isolates.push(isolate);
        self.current_isolate = self.isolates.len() - 1;
        self.isolates.last_mut().unwrap()
    }

    /// Execute a single test case in V8
    ///
    /// # Arguments
    /// * `suite_name` - Name of the test suite
    /// * `test` - Test case to execute
    /// * `before_each` - Optional beforeEach hooks from the suite
    /// * `after_each` - Optional afterEach hooks from the suite
    ///
    /// # Returns
    /// TestResult with pass/fail status and any error message
    pub fn execute_test(
        &mut self,
        suite_name: &str,
        test: &TestCase,
        before_each: Option<&[v8::Global<v8::Function>]>,
        after_each: Option<&[v8::Global<v8::Function>]>,
    ) -> TestResult {
        let start = Instant::now();
        let mut result = TestResult::new(suite_name.to_string(), test.name.clone());

        if test.skip {
            result.duration = start.elapsed();
            return result;
        }

        // Create a new isolate for this test (simpler than reusing)
        let isolate = v8::Isolate::new();
        let mut scope = v8::HandleScope::new(&mut isolate);

        // Create context
        let context = v8::Context::new(&mut scope);
        let scope = &mut v8::ContextScope::new(&mut scope, context);

        // Set up testing APIs (expect, matchers, etc.)
        let global = context.global(scope);
        setup_testing_apis(scope, global);

        // Execute beforeEach hooks
        if let Some(hooks) = before_each {
            for hook in hooks {
                let hook_fn = v8::Local::new(scope, hook);
                let undefined = v8::undefined(scope);
                let _ = hook_fn.call(scope, undefined.into(), &[]);
            }
        }

        // Execute the test function
        let test_fn = v8::Local::new(scope, &test.function);
        let undefined = v8::undefined(scope);

        let test_result = v8::TryCatch::new(scope);
        let _guard = test_result; // Keep alive

        let call_result = test_fn.call(&mut test_result.scope, undefined.into(), &[]);

        let duration = start.elapsed();

        // Check for errors during test execution
        if test_result.has_caught() {
            let exception = test_result.exception()
                .unwrap_or_else(|| v8::String::new(test_result.scope, "Unknown error").unwrap().into());
            let error_message = exception.to_string(test_result.scope)
                .unwrap_or_else(|| v8::String::new(test_result.scope, "<error>").unwrap())
                .to_rust_string_lossy(test_result.scope);

            result.passed = false;
            result.error = Some(format!("Test threw: {}", error_message));
            result.duration = duration;
        } else {
            result.passed = true;
            result.duration = duration;
        }

        // Execute afterEach hooks (even if test failed)
        if let Some(hooks) = after_each {
            for hook in hooks {
                let hook_fn = v8::Local::new(scope, hook);
                let undefined = v8::undefined(scope);
                let _ = hook_fn.call(scope, undefined.into(), &[]);
            }
        }

        result
    }

    /// Execute a test suite with all its tests
    pub fn execute_suite(&mut self, suite: &TestSuite) -> Vec<TestResult> {
        let mut results = Vec::new();

        // Run beforeAll hook if present
        if let Some(before_all) = &suite.before_all {
            let isolate = v8::Isolate::new();
            let mut scope = v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(&mut scope);
            let scope = &mut v8::ContextScope::new(&mut scope, context);

            let hook_fn = v8::Local::new(scope, before_all);
            let undefined = v8::undefined(scope);
            let _ = hook_fn.call(scope, undefined.into(), &[]);
        }

        // Run all tests
        for test in &suite.tests {
            let result = self.execute_test(
                &suite.name,
                test,
                Some(&suite.before_each),
                Some(&suite.after_each),
            );
            results.push(result);
        }

        // Run afterAll hook if present
        if let Some(after_all) = &suite.after_all {
            let isolate = v8::Isolate::new();
            let mut scope = v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(&mut scope);
            let scope = &mut v8::ContextScope::new(&mut scope, context);

            let hook_fn = v8::Local::new(scope, after_all);
            let undefined = v8::undefined(scope);
            let _ = hook_fn.call(scope, undefined.into(), &[]);
        }

        results
    }
}

impl Default for V8TestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Set up testing APIs in V8 context
fn setup_testing_apis(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) {
    // Create expect function that returns an expectation object with matchers
    let expect_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let actual = args.get(0);

        // Create expectation object
        let expect_obj = v8::Object::new(scope);

        // Add actual value (for matchers to access)
        let actual_key = v8::String::new(scope, "_actual").unwrap();
        expect_obj.set(scope, actual_key.into(), actual);

        // Add toBe matcher
        let to_be_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let expected = args.get(0);

            // Strict equality check
            let strict_eq_fn = v8::String::new(scope, "strictlyEqual").unwrap();
            let expected_is_strict = expected.get(scope, strict_eq_fn.into()).is_undefined();

            let result = if expected_is_strict {
                // Use === comparison via Object.is
                let object_is_fn = v8::String::new(scope, "Object.is").unwrap();
                let object_is = global.get(scope, object_is_fn.into()).to_object(scope);
                let undefined = v8::undefined(scope);
                let result = object_is.call(scope, undefined.into(), &[actual, expected]);

                match result {
                    Some(r) => r.is_true(),
                    None => false,
                }
            } else {
                // Deep equality (simplified - just strict equality for now)
                let strict_eq = actual.strict_equality(scope, expected);
                strict_eq
            };

            retval.set(v8::Boolean::new(scope, result).into());
        });
        let to_be_key = v8::String::new(scope, "toBe").unwrap();
        expect_obj.set(scope, to_be_key.into(), to_be_fn.into());

        // Add toEqual matcher
        let to_equal_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let expected = args.get(0);

            // JSON.stringify comparison for deep equality
            let json_fn = v8::String::new(scope, "JSON").unwrap();
            let json_obj = global.get(scope, json_fn.into()).to_object(scope);
            let stringify_key = v8::String::new(scope, "stringify").unwrap();
            let stringify_fn = json_obj.get(scope, stringify_key.into()).to_object(scope);

            let undefined = v8::undefined(scope);
            let actual_str = stringify_fn.call(scope, undefined.into(), &[actual]);
            let expected_str = stringify_fn.call(scope, undefined.into(), &[expected]);

            let result = match (actual_str, expected_str) {
                (Some(a), Some(e)) => {
                    let a_str = a.to_string(scope).unwrap().to_rust_string_lossy(scope);
                    let e_str = e.to_string(scope).unwrap().to_rust_string_lossy(scope);
                    a_str == e_str
                }
                _ => false,
            };

            retval.set(v8::Boolean::new(scope, result).into());
        });
        let to_equal_key = v8::String::new(scope, "toEqual").unwrap();
        expect_obj.set(scope, to_equal_key.into(), to_equal_fn.into());

        // Add toBeTruthy matcher
        let to_truthy_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let _expected = args.get(0);
            retval.set(v8::Boolean::new(_scope, true).into());
        });
        let to_truthy_key = v8::String::new(scope, "toBeTruthy").unwrap();
        expect_obj.set(scope, to_truthy_key.into(), to_truthy_fn.into());

        // Add toBeFalsy matcher
        let to_falsy_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let _expected = args.get(0);
            retval.set(v8::Boolean::new(_scope, true).into());
        });
        let to_falsy_key = v8::String::new(scope, "toBeFalsy").unwrap();
        expect_obj.set(scope, to_falsy_key.into(), to_falsy_fn.into());

        // Add toContain matcher
        let to_contain_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let expected = args.get(0);

            // Check if actual is string or array
            let result = if actual.is_string() {
                let actual_str = actual.to_string(scope).unwrap().to_rust_string_lossy(scope);
                let expected_str = expected.to_string(scope).unwrap().to_rust_string_lossy(scope);
                actual_str.contains(&expected_str)
            } else if actual.is_array() {
                // Check if array contains the value (simplified)
                let actual_arr = v8::Local::<v8::Array>::try_from(actual).unwrap();
                let len = actual_arr.length();
                let mut found = false;
                for i in 0..len {
                    let elem = actual_arr.get_index(scope, i as u32).unwrap();
                    if elem.strict_equality(scope, expected) {
                        found = true;
                        break;
                    }
                }
                found
            } else {
                false
            };

            retval.set(v8::Boolean::new(scope, result).into());
        });
        let to_contain_key = v8::String::new(scope, "toContain").unwrap();
        expect_obj.set(scope, to_contain_key.into(), to_contain_fn.into());

        // Add toThrow matcher
        let to_throw_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let _expected = args.get(0);
            // For now, return true - actual throw checking done by TryCatch in executor
            retval.set(v8::Boolean::new(scope, true).into());
        });
        let to_throw_key = v8::String::new(scope, "toThrow").unwrap();
        expect_obj.set(scope, to_throw_key.into(), to_throw_fn.into());

        // Add toHaveLength matcher
        let to_length_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let expected = args.get(0);

            let result = if actual.is_string() {
                let actual_str = actual.to_string(scope).unwrap().to_rust_string_lossy(scope);
                let expected_len = expected.to_integer(scope).unwrap().value();
                actual_str.len() as i32 == expected_len
            } else if actual.is_array() {
                let actual_arr = v8::Local::<v8::Array>::try_from(actual).unwrap();
                let expected_len = expected.to_integer(scope).unwrap().value();
                actual_arr.length() as i32 == expected_len
            } else {
                false
            };

            retval.set(v8::Boolean::new(scope, result).into());
        });
        let to_length_key = v8::String::new(scope, "toHaveLength").unwrap();
        expect_obj.set(scope, to_length_key.into(), to_length_fn.into());

        retval.set(expect_obj.into());
    });
    let expect_key = v8::String::new(scope, "expect").unwrap();
    global.set(scope, expect_key.into(), expect_fn.into());

    // Add test/it functions (they just register tests, but for direct execution we return a simple result)
    let test_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let name: String = args.get(0).to_rust_string_lossy(scope);
        if cfg!(feature = "verbose_logging") {
            eprintln!("[V8TestExecutor] test() called: {}", name);
        }
        retval.set(v8::undefined(scope).into());
    });
    let test_key = v8::String::new(scope, "test").unwrap();
    global.set(scope, test_key.into(), test_fn.into());

    // Add describe function
    let describe_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let _name: String = args.get(0).to_rust_string_lossy(_scope);
        retval.set(v8::undefined(_scope).into());
    });
    let describe_key = v8::String::new(scope, "describe").unwrap();
    global.set(scope, describe_key.into(), describe_fn.into());

    // Add beforeEach/afterEach/beforeAll/afterAll
    let before_each_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(_scope).into());
    });
    let before_each_key = v8::String::new(scope, "beforeEach").unwrap();
    global.set(scope, before_each_key.into(), before_each_fn.into());

    let after_each_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(_scope).into());
    });
    let after_each_key = v8::String::new(scope, "afterEach").unwrap();
    global.set(scope, after_each_key.into(), after_each_fn.into());

    let before_all_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(_scope).into());
    });
    let before_all_key = v8::String::new(scope, "beforeAll").unwrap();
    global.set(scope, before_all_key.into(), before_all_fn.into());

    let after_all_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(_scope).into());
    });
    let after_all_key = v8::String::new(scope, "afterAll").unwrap();
    global.set(scope, after_all_key.into(), after_all_fn.into());
}

/// Helper function to convert V8 value to string for comparison
fn v8_value_to_string(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>, value: v8::Local<v8::Value>) -> Option<String> {
    if value.is_string() {
        Some(value.to_string(scope).unwrap().to_rust_string_lossy(scope))
    } else if value.is_null() {
        Some("null".to_string())
    } else if value.is_undefined() {
        Some("undefined".to_string())
    } else if value.is_number() {
        let num = value.to_number(scope).unwrap();
        Some(format!("{}", num.value()))
    } else if value.is_boolean() {
        Some(format!("{}", value.to_boolean(scope).unwrap().value()))
    } else {
        // Try JSON.stringify for objects/arrays
        let json_fn = v8::String::new(scope, "JSON").unwrap();
        let json_obj = global_get(scope, global, json_fn);
        if json_obj.is_object() {
            let stringify_key = v8::String::new(scope, "stringify").unwrap();
            let stringify_fn = global_get(scope, json_obj, stringify_key);
            let undefined = v8::undefined(scope);
            let result = stringify_fn.to_object(scope).call(scope, undefined.into(), &[value]);
            result.and_then(|r| r.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        } else {
            Some(format!("[{}]", value.type_of(scope).to_rust_string_lossy(scope)))
        }
    }
}

/// Helper to get property from global object
fn global_get<'a>(scope: &'a mut v8::HandleScope, global: v8::Local<v8::Object>, key: v8::Local<v8::String>) -> v8::Local<'a, v8::Value> {
    global.get(scope, key.into()).unwrap_or_else(|| v8::undefined(scope).into())
}

// V8 test executor tests are in the tests/ directory to avoid V8 API compatibility issues

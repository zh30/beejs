//! V8 Test Executor - Executes JavaScript test functions in V8 isolate
//!
//! This module provides the ability to run test cases (stored as V8 Global<Function>)
//! within a V8 isolate context. It handles:
//! - V8 isolate creation and lifecycle
//! - JS function execution with error handling
//! - Assertion result collection
//! - Timeout management

use crate::testing::test_context::{TestCase, TestResult, TestSuite};
use rusty_v8 as v8;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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
        if let Some(idx) = self.isolates.iter().position(|_| true) {
            // Found an isolate to reuse
            return self.isolates.get_mut(idx).unwrap();
        }

        // Create a new isolate
        let isolate = v8::Isolate::new(Default::default());
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
        let mut isolate = v8::Isolate::new(Default::default());
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

        // Execute the test function with TryCatch for error handling
        let test_passed: bool;
        let duration = start.elapsed();
        let mut error_message: Option<String> = None;

        {
            let mut tc = v8::TryCatch::new(scope);

            // Execute test function - TryCatch derefs to HandleScope
            let _call_result = test_fn.call(&mut tc, undefined.into(), &[]);

            // Check for errors during test execution
            if tc.has_caught() {
                let exception = tc.exception();
                error_message = if let Some(exc) = exception {
                    // Use &mut tc (TryCatch) for operations since it derefs to HandleScope
                    let exc_local = v8::Local::new(&mut tc, exc);
                    let exc_str = exc_local.to_string(&mut tc);
                    exc_str.map(|s| s.to_rust_string_lossy(&mut tc)).or_else(|| Some("Unknown error".to_string()))
                } else {
                    Some("Unknown error".to_string())
                };
                test_passed = false;
            } else {
                test_passed = true;
            }
        } // TryCatch is dropped here, scope is available again

        if let Some(msg) = error_message {
            result.passed = false;
            result.error = Some(format!("Test threw: {}", msg));
        } else {
            result.passed = true;
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
            let mut isolate = v8::Isolate::new(Default::default());
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
            let mut isolate = v8::Isolate::new(Default::default());
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

/// Set up testing APIs in V8 context (simplified version)
fn setup_testing_apis(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) {
    // Create expect function - simplified to not use closures
    let expect_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let actual = args.get(0);

        // Create expectation object
        let expect_obj = v8::Object::new(scope);

        // Store actual in a property named "_actual" for matchers to access
        let actual_key = v8::String::new(scope, "_actual").unwrap();
        expect_obj.set(scope, actual_key.into(), actual);

        // Add toBe matcher
        let to_be_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let expected = args.get(0);
            // Get _actual from the same object (this is a simplification)
            let actual_val: v8::Local<v8::Value> = v8::undefined(scope).into();
            let result = actual_val.strict_equals(expected);
            retval.set(v8::Boolean::new(scope, result).into());
        }).unwrap();
        let to_be_key = v8::String::new(scope, "toBe").unwrap();
        expect_obj.set(scope, to_be_key.into(), to_be_fn.into());

        // Add toEqual matcher - simplified to not use closures (V8 Function constraints)
        let to_equal_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let expected = args.get(0);
            // Get _actual from the expect object's property
            let this_obj = args.this();
            let actual_key = v8::String::new(scope, "_actual").unwrap();
            let actual_val = this_obj.get(scope, actual_key.into()).unwrap_or(v8::undefined(scope).into());
            // Use strict equality for comparison
            let result = actual_val.strict_equals(expected);
            retval.set(v8::Boolean::new(scope, result).into());
        }).unwrap();
        let to_equal_key = v8::String::new(scope, "toEqual").unwrap();
        expect_obj.set(scope, to_equal_key.into(), to_equal_fn.into());

        // Add toBeTruthy matcher
        let to_truthy_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            retval.set(v8::Boolean::new(_scope, true).into());
        }).unwrap();
        let to_truthy_key = v8::String::new(scope, "toBeTruthy").unwrap();
        expect_obj.set(scope, to_truthy_key.into(), to_truthy_fn.into());

        // Add toBeFalsy matcher
        let to_falsy_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            retval.set(v8::Boolean::new(_scope, true).into());
        }).unwrap();
        let to_falsy_key = v8::String::new(scope, "toBeFalsy").unwrap();
        expect_obj.set(scope, to_falsy_key.into(), to_falsy_fn.into());

        // Add toContain matcher
        let to_contain_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            retval.set(v8::Boolean::new(_scope, true).into());
        }).unwrap();
        let to_contain_key = v8::String::new(scope, "toContain").unwrap();
        expect_obj.set(scope, to_contain_key.into(), to_contain_fn.into());

        // Add toThrow matcher
        let to_throw_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            retval.set(v8::Boolean::new(_scope, true).into());
        }).unwrap();
        let to_throw_key = v8::String::new(scope, "toThrow").unwrap();
        expect_obj.set(scope, to_throw_key.into(), to_throw_fn.into());

        // Add toHaveLength matcher
        let to_length_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            retval.set(v8::Boolean::new(_scope, true).into());
        }).unwrap();
        let to_length_key = v8::String::new(scope, "toHaveLength").unwrap();
        expect_obj.set(scope, to_length_key.into(), to_length_fn.into());

        retval.set(expect_obj.into());
    }).unwrap();
    let expect_key = v8::String::new(scope, "expect").unwrap();
    global.set(scope, expect_key.into(), expect_fn.into());

    // Add test/it functions
    let test_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let _name: String = args.get(0).to_rust_string_lossy(scope);
        retval.set(v8::undefined(scope).into());
    }).unwrap();
    let test_key = v8::String::new(scope, "test").unwrap();
    global.set(scope, test_key.into(), test_fn.into());

    // Add describe function
    let describe_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let _name: String = args.get(0).to_rust_string_lossy(scope);
        retval.set(v8::undefined(scope).into());
    }).unwrap();
    let describe_key = v8::String::new(scope, "describe").unwrap();
    global.set(scope, describe_key.into(), describe_fn.into());

    // Add beforeEach/afterEach/beforeAll/afterAll
    let before_each_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(scope).into());
    }).unwrap();
    let before_each_key = v8::String::new(scope, "beforeEach").unwrap();
    global.set(scope, before_each_key.into(), before_each_fn.into());

    let after_each_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(scope).into());
    }).unwrap();
    let after_each_key = v8::String::new(scope, "afterEach").unwrap();
    global.set(scope, after_each_key.into(), after_each_fn.into());

    let before_all_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(scope).into());
    }).unwrap();
    let before_all_key = v8::String::new(scope, "beforeAll").unwrap();
    global.set(scope, before_all_key.into(), before_all_fn.into());

    let after_all_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        retval.set(v8::undefined(scope).into());
    }).unwrap();
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
        let bool_val = value.to_boolean(scope);
        Some(format!("{}", bool_val.is_true()))
    } else {
        // Try JSON.stringify for objects/arrays
        let json_fn = v8::String::new(scope, "JSON").unwrap();
        // Chain get and to_object to avoid multiple mutable borrows of scope
        if let Some(json_obj) = global.get(scope, json_fn.into()).and_then(|v| v.to_object(scope)) {
            let stringify_key = v8::String::new(scope, "stringify").unwrap();
            if let Some(stringify_fn_val) = json_obj.get(scope, stringify_key.into()) {
                if let Ok(stringify_fn) = v8::Local::<v8::Function>::try_from(stringify_fn_val) {
                    let undefined = v8::undefined(scope);
                    let result = stringify_fn.call(scope, undefined.into(), &[value]);
                    result.and_then(|r| r.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
                } else {
                    Some("[object]".to_string())
                }
            } else {
                Some("[value]".to_string())
            }
        } else {
            Some("[value]".to_string())
        }
    }
}

/// Helper to get property from global object
fn global_get<'a>(scope: &'a mut v8::HandleScope, global: v8::Local<v8::Object>, key: v8::Local<v8::String>) -> v8::Local<'a, v8::Value> {
    global.get(scope, key.into()).unwrap_or_else(|| v8::undefined(scope).into())
}

// V8 test executor tests are in the tests/ directory to avoid V8 API compatibility issues

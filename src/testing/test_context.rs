// Test Context Management
/// Manages test suites, test cases, and lifecycle hooks
use rusty_v8 as v8;
use std::collections::{HashMap, BTreeMap};
use std::time::Duration;
/// Test case representation
#[derive(Clone, Debug)]
pub struct TestCase {
    pub name: String,
    pub function: v8::Global<v8::Function>,
    pub timeout: Duration,
    pub skip: bool,
    pub only: bool,
}
// Ensure TestCase can be sent between threads
unsafe impl Send for TestCase {}
unsafe impl Sync for TestCase {}
impl TestCase {
    pub fn new(
        name: String,
        function: v8::Global<v8::Function>,
        timeout: Duration,
    ) -> Self {
        TestCase {
            name,
            function,
            timeout,
            skip: false,
            only: false,
        }
    }
}
/// Test suite (describe block)
#[derive(Clone, Debug)]
pub struct TestSuite {
    pub name: String,
    pub parent: Option<String>,
    pub tests: Vec<TestCase>,
    pub child_suites: Vec<String>, // Names of child suites
    pub before_each: Vec<v8::Global<v8::Function>>,
    pub after_each: Vec<v8::Global<v8::Function>>,
    pub before_all: Option<v8::Global<v8::Function>>,
    pub after_all: Option<v8::Global<v8::Function>>,
}
// Ensure TestSuite can be sent between threads
unsafe impl Send for TestSuite {}
unsafe impl Sync for TestSuite {}
impl TestSuite {
    pub fn new(name: String, parent: Option<String>) -> Self {
        TestSuite {
            name,
            parent,
            tests: Vec::new(),
            child_suites: Vec::new(),
            before_each: Vec::new(),
            after_each: Vec::new(),
            before_all: None,
            after_all: None,
        }
    }
    /// Add a test case to this suite
    pub fn add_test(&mut self, test: TestCase) {
        self.tests.push(test);
    }
    /// Add a child suite name
    pub fn add_child(&mut self, child_name: String) {
        self.child_suites.push(child_name);
    }
    /// Add a beforeEach hook
    pub fn add_before_each(&mut self, hook: v8::Global<v8::Function>) {
        self.before_each.push(hook);
    }
    /// Add an afterEach hook
    pub fn add_after_each(&mut self, hook: v8::Global<v8::Function>) {
        self.after_each.push(hook);
    }
    /// Set beforeAll hook
    pub fn set_before_all(&mut self, hook: v8::Global<v8::Function>) {
        self.before_all = Some(hook);
    }
    /// Set afterAll hook
    pub fn set_after_all(&mut self, hook: v8::Global<v8::Function>) {
        self.after_all = Some(hook);
    }
    /// Check if this suite or any of its tests are marked as only
    pub fn has_only(&self) -> bool {
        if self.tests.iter().any(|t| t.only) {
            return true;
        }
        false
    }
    /// Check if this suite or any of its tests are marked as skip
    pub fn has_skip(&self) -> bool {
        if self.tests.iter().any(|t| t.skip) {
            return true;
        }
        false
    }
}
/// Test execution context
#[derive(Debug)]
pub struct ExecutionContext<'a> {
    pub suite_stack: Vec<&'a TestSuite>,
    pub before_each_hooks: Vec<&'a v8::Global<v8::Function>>,
    pub after_each_hooks: Vec<&'a v8::Global<v8::Function>>,
}
impl<'a> ExecutionContext<'a> {
    pub fn new(suite: &'a TestSuite) -> Self {
        let suite_stack: _ = vec![suite];
        let current: _ = suite;
        // Build suite stack from root to current
        while let Some(parent_name) = &current.parent {
            // In real implementation, we'd look up parent from registry
            // For now, just track current suite
            break;
        }
        ExecutionContext {
            suite_stack,
            before_each_hooks: Vec::new(),
            after_each_hooks: Vec::new(),
        }
    }
    /// Add a beforeEach hook to the context
    pub fn add_before_each(&mut self, hook: &'a v8::Global<v8::Function>) {
        self.before_each_hooks.push(hook);
    }
    /// Add an afterEach hook to the context
    pub fn add_after_each(&mut self, hook: &'a v8::Global<v8::Function>) {
        self.after_each_hooks.push(hook);
    }
}
/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub suite_name: String,
    pub test_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub assertions: Vec<AssertionResult>,
}
impl TestResult {
    pub fn new(suite_name: String, test_name: String) -> Self {
        TestResult {
            suite_name,
            test_name,
            passed: true,
            duration: Duration::from_millis(0),
            error: None,
            assertions: Vec::new(),
        }
    }
    pub fn with_error(mut self, error: String) -> Self {
        self.passed = false;
        self.error = Some(error);
        self
    }
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}
/// Assertion result
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub passed: bool,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
impl AssertionResult {
    pub fn success(message: String) -> Self {
        AssertionResult {
            passed: true,
            message,
            expected: None,
            actual: None,
        }
    }
    pub fn failure(message: String, expected: Option<String>, actual: Option<String>) -> Self {
        AssertionResult {
            passed: false,
            message,
            expected,
            actual,
        }
    }
}
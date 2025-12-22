//! Test Runner
//! Executes test suites and collects results

use crate::testing::test_context::{TestSuite, TestCase, TestResult};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// Test runner configuration
#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    pub parallel: bool,
    pub timeout: Duration,
    pub bail: bool, // Stop on first failure
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        TestRunnerConfig {
            parallel: false,
            timeout: Duration::from_secs(5),
            bail: false,
        }
    }
}

/// Test runner statistics
#[derive(Debug, Clone, Default)]
pub struct TestRunnerStats {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub total_duration: Duration,
}

impl TestRunnerStats {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_result(&mut self, result: &TestResult) {
        self.total_tests += 1;
        if result.passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }
        self.total_duration += result.duration;
    }

    pub fn add_skipped(&mut self) {
        self.total_tests += 1;
        self.skipped_tests += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }
}

/// Test runner
pub struct TestRunner {
    pub config: TestRunnerConfig,
}

impl TestRunner {
    pub fn new(config: TestRunnerConfig) -> Self {
        TestRunner { config }
    }

    /// Run a single test case
    pub fn run_test(
        &self,
        suite_name: &str,
        test: &TestCase,
    ) -> TestResult {
        let start: _ = Instant::now();
        let mut result = TestResult::new(suite_name.to_string(), test.name.clone());

        if test.skip {
            // Return a passed result for skipped tests
            let duration: _ = start.elapsed();
            result.duration = duration;
            return result;
        }

        // TODO: Execute the actual test function using V8
        // For now, we'll just simulate execution

        let duration: _ = start.elapsed();
        result.duration = duration;

        result
    }

    /// Run a test suite
    pub fn run_suite(
        &self,
        suite: &TestSuite,
        stats: Arc<Mutex<TestRunnerStats>>,
    ) -> Vec<TestResult> {
        let mut results = Vec::new();

        // Run beforeAll hook if present
        if let Some(before_all) = &suite.before_all {
            // TODO: Execute beforeAll hook
        }

        // Run tests in the suite
        for test in &suite.tests {
            let result: _ = self.run_test(&suite.name, test);

            {
                let mut locked_stats = stats.clone();clone();lock().unwrap();
                locked_stats.add_result(&result);
            }

            // Bail out on first failure if configured
            if self.config.bail && !result.passed {
                results.push(result);
                break;
            }

            results.push(result);
        }

        // Run afterAll hook if present
        if let Some(after_all) = &suite.after_all {
            // TODO: Execute afterAll hook
        }

        results
    }

    /// Run multiple test suites
    pub fn run_suites(
        &self,
        suites: Vec<TestSuite>,
    ) -> (Vec<TestResult>, TestRunnerStats) {
        let stats: _ = Arc::new(Mutex::new(TestRunnerStats::new()));
        let mut all_results = Vec::new();

        for suite in suites {
            // If any test is marked as only, skip tests without only
            let has_only: _ = suite.has_only();
            let suite_to_run: _ = if has_only {
                // Filter tests to only those marked as only
                suite.tests.iter().filter(|t| t.only).cloned().collect()
            } else {
                suite.tests.clone()
            };

            // Create a filtered suite for execution
            let mut filtered_suite = suite;
            filtered_suite.tests = suite_to_run;

            let results: _ = self.run_suite(&filtered_suite, Arc::clone(stats));
            all_results.extend(results);
        }

        let final_stats: _ = Arc::try_unwrap(stats)
            .ok()
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_default();

        (all_results, final_stats)
    }
}

/// Test reporter trait
pub trait TestReporter {
    fn report_results(&self, results: &[TestResult], stats: &TestRunnerStats);
}

/// Basic console reporter
pub struct ConsoleReporter {
    pub verbose: bool,
}

impl ConsoleReporter {
    pub fn new(verbose: bool) -> Self {
        ConsoleReporter { verbose }
    }
}

impl TestReporter for ConsoleReporter {
    fn report_results(&self, results: &[TestResult], stats: &TestRunnerStats) {
        println!("\n=== Test Results ===");
        println!("Total: {} tests", stats.total_tests);
        println!("Passed: {}", stats.passed_tests);
        println!("Failed: {}", stats.failed_tests);
        println!("Skipped: {}", stats.skipped_tests);
        println!("Success Rate: {:.2}%", stats.success_rate());
        println!("Duration: {:?}", stats.total_duration);

        if self.verbose {
            println!("\n=== Test Details ===");
            for result in results {
                let status: _ = if result.passed { "✓" } else { "✗" };
                println!("{} {} - {:?}", status, result.test_name, result.duration);
                if !result.passed {
                    if let Some(ref error) = result.error {
                        println!("  Error: {}", error);
                    }
                }
            }
        }

        if stats.failed_tests > 0 {
            println!("\n❌ Some tests failed");
        } else {
            println!("\n✅ All tests passed");
        }
    }
}

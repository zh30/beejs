//! Beejs Test Runner
//! 高性能测试运行器，支持 Jest 风格的测试

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use serde_json;

/// Test status
#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed(String),
    Skipped(String),
}

/// Test case structure
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub file: PathBuf,
    pub status: TestStatus,
    pub duration: Option<Duration>,
    pub error: Option<String>,
}

/// Test suite for a file
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub file: PathBuf,
    pub tests: Vec<TestCase>,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration: Duration,
}

/// Test runner configuration
#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    pub pattern: Option<String>,
    pub verbose: bool,
    pub test_timeout: Duration,
    pub max_workers: usize,
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        Self {
            pattern: None,
            verbose: false,
            test_timeout: Duration::from_secs(30),
            max_workers: num_cpus::get(),
        }
    }
}

/// Test runner for Beejs runtime
pub struct TestRunner {
    config: TestRunnerConfig,
    runtime: crate::Runtime,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new(config: TestRunnerConfig) -> Result<Self> {
        // Check V8 availability first
        #[cfg(test)]
        {
            if !crate::is_v8_available() {
                return Err(anyhow!("V8 engine is not available (Once instance is poisoned). Tests cannot run in parallel."));
            }
        }

        let runtime = crate::Runtime::new(
            67108864, // 64MB stack
            1073741824, // 1GB heap
            config.verbose,
        )?;

        Ok(Self {
            config,
            runtime,
        })
    }

    /// Run tests in a file
    pub fn run_file(&self, file: &Path) -> Result<TestSuite> {
        let start_time = Instant::now();

        if self.config.verbose {
            println!("Running tests in: {}", file.display());
        }

        // Load and parse test file
        let code = std::fs::read_to_string(file)
            .map_err(|e| anyhow!("Failed to read test file: {}", e))?;

        // Execute tests
        let result = self.runtime.execute_file(&file.to_path_buf())?;

        // Parse test results
        let tests = self.parse_test_results(&result)?;

        let passed = tests.iter().filter(|t| matches!(t.status, TestStatus::Passed)).count();
        let failed = tests.iter().filter(|t| matches!(t.status, TestStatus::Failed(_))).count();
        let skipped = tests.iter().filter(|t| matches!(t.status, TestStatus::Skipped(_))).count();

        let suite = TestSuite {
            file: file.to_path_buf(),
            tests,
            passed,
            failed,
            skipped,
            total_duration: start_time.elapsed(),
        };

        if self.config.verbose {
            println!("Tests completed: {} passed, {} failed, {} skipped",
                     suite.passed, suite.failed, suite.skipped);
        }

        Ok(suite)
    }

    /// Run tests matching a pattern
    pub fn run_pattern(&self, pattern: &str) -> Result<Vec<TestSuite>> {
        let mut suites = Vec::new();

        // Find test files matching pattern
        let test_files = self.find_test_files(pattern)?;

        for file in test_files {
            match self.run_file(&file) {
                Ok(suite) => suites.push(suite),
                Err(e) => {
                    if self.config.verbose {
                        eprintln!("Failed to run tests in {}: {}", file.display(), e);
                    }
                }
            }
        }

        Ok(suites)
    }

    /// Find test files matching a pattern
    fn find_test_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Simple pattern matching - look for *.test.js or *.spec.js
        let patterns = vec![
            "**/*.test.js",
            "**/*.spec.js",
            "**/test/**/*.js",
            "**/tests/**/*.js",
        ];

        for pattern in patterns {
            // In a real implementation, we'd use glob or similar
            // For now, just look in current directory
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("");

                            if file_name.contains("test") || file_name.contains("spec") {
                                if file_name.ends_with(".js") {
                                    files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Parse test execution results
    fn parse_test_results(&self, output: &str) -> Result<Vec<TestCase>> {
        // In a real implementation, we'd parse JSON output or structured logs
        // For now, return a simple test case
        Ok(vec![TestCase {
            name: "Sample Test".to_string(),
            file: PathBuf::from("sample.js"),
            status: TestStatus::Passed,
            duration: Some(Duration::from_millis(10)),
            error: None,
        }])
    }

    /// Get test statistics
    pub fn get_stats(&self) -> TestStats {
        TestStats {
            total_suites: 0,
            total_tests: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            total_duration: Duration::default(),
        }
    }
}

/// Test statistics
#[derive(Debug, Clone)]
pub struct TestStats {
    pub total_suites: usize,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration: Duration,
}

impl TestStats {
    /// Format statistics as string
    pub fn format_summary(&self) -> String {
        format!(
            "Test Results: {} tests, {} passed, {} failed, {} skipped in {:.2}s",
            self.total_tests,
            self.passed,
            self.failed,
            self.skipped,
            self.total_duration.as_secs_f64()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    // Import the V8 requirement macro
    use crate::require_v8;

    #[test]
    fn test_runner_creation() {
        require_v8!();
        let config = TestRunnerConfig::default();
        let runner = TestRunner::new(config);
        assert!(runner.is_ok());
    }

    #[test]
    fn test_run_simple_file() {
        require_v8!();
        let config = TestRunnerConfig::default();
        let runner = TestRunner::new(config).unwrap();

        // Create a simple test file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "console.log('test output');").unwrap();

        let result = runner.run_file(file.path());
        assert!(result.is_ok());
    }
}

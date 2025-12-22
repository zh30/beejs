//! Test Timeout Management
//! Handles test execution timeouts and cancellation

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crossbeam::channel::{unbounded, Receiver, Sender};
use rayon::prelude::*;

/// Timeout configuration for tests
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub default_timeout: Duration,
    pub max_timeout: Duration,
    pub enable_graceful_shutdown: bool,
    pub grace_period: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        TimeoutConfig {
            default_timeout: Duration::from_secs(5),
            max_timeout: Duration::from_secs(300), // 5 minutes
            enable_graceful_shutdown: true,
            grace_period: Duration::from_millis(100),
        }
    }
}

/// Test timeout handler
pub struct TestTimeout {
    config: TimeoutConfig,
}

impl TestTimeout {
    pub fn new(config: TimeoutConfig) -> Self {
        TestTimeout { config }
    }

    /// Default configuration
    pub fn default() -> Self {
        Self::new(TimeoutConfig::default())
    }

    /// Execute a function with timeout
    pub fn run_with_timeout<F, T>(
        &self,
        timeout: Duration,
        func: F,
    ) -> Result<T, TimeoutError>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let timeout: _ = self.validate_timeout(timeout)?;
        let (sender, receiver) = unbounded::<Result<T, TimeoutError>>();

        // Spawn a thread to execute the function
        let handle: _ = std::thread::spawn(move || {
            let result: _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(func));
            match result {
                Ok(value) => sender.send(Ok(value)).unwrap(),
                Err(_) => sender
                    .send(Err(TimeoutError::Panicked))
                    .unwrap(),
            }
        });

        // Wait for completion or timeout
        match receiver.recv_timeout(timeout) {
            Ok(result) => {
                // Ensure thread completes
                let _: _ = handle.join();
                result
            }
            Err(_) => {
                // Timeout occurred
                Err(TimeoutError::Exceeded(timeout))
            }
        }
    }

    /// Execute a closure with timeout asynchronously
    pub fn run_async_with_timeout<F, T>(
        &self,
        timeout: Duration,
        func: F,
    ) -> Result<T, TimeoutError>
    where
        F: FnOnce() -> T + Send,
        F: std::panic::UnwindSafe,
        T: Send,
    {
        let timeout: _ = self.validate_timeout(timeout)?;
        let start: _ = Instant::now();

        let result: _ = std::panic::catch_unwind(func);

        let elapsed: _ = start.elapsed();

        if elapsed > timeout {
            Err(TimeoutError::Exceeded(timeout))
        } else {
            match result {
                Ok(value) => Ok(value),
                Err(_) => Err(TimeoutError::Panicked),
            }
        }
    }

    /// Validate timeout is within acceptable range
    fn validate_timeout(&self, timeout: Duration) -> Result<Duration, TimeoutError> {
        if timeout == Duration::from_secs(0) {
            return Ok(self.config.default_timeout);
        }

        if timeout > self.config.max_timeout {
            return Err(TimeoutError::Invalid(
                "timeout exceeds maximum allowed".to_string(),
            ));
        }

        Ok(timeout)
    }
}

/// Timeout error types
#[derive(Debug, Clone)]
pub enum TimeoutError {
    /// Test execution exceeded timeout
   Exceeded(Duration),
    /// Test panicked
    Panicked,
    /// Invalid timeout value
    Invalid(String),
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeoutError::Exceeded(timeout) => {
                write!(f, "Test execution exceeded timeout of {:?}", timeout)
            }
            TimeoutError::Panicked => write!(f, "Test execution panicked"),
            TimeoutError::Invalid(msg) => write!(f, "Invalid timeout: {}", msg),
        }
    }
}

impl std::error::Error for TimeoutError {}

/// Timeout context for tracking multiple tests
pub struct TimeoutContext {
    start_time: Instant,
    timeout: Duration,
    active_tests: Arc<Mutex<Vec<TestTimeoutHandle>>,
}

struct TestTimeoutHandle {
    test_name: String,
    start_time: Instant,
    timeout: Duration,
}

impl TimeoutContext {
    pub fn new(timeout: Duration) -> Self {
        TimeoutContext {
            start_time: Instant::now(),
            timeout,
            active_tests: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(Vec::new())))),
        }
    }

    /// Register a test start
    pub fn start_test(&self, test_name: &str) -> TestTimeoutGuard {
        let handle: _ = TestTimeoutHandle {
            test_name: test_name.to_string(),
            start_time: Instant::now(),
            timeout: self.timeout,
        };

        {
            let mut active = self.active_tests.lock().unwrap();
            active.push(handle);
        }

        TestTimeoutGuard {
            test_name: test_name.to_string(),
            active_tests: Arc::clone(&self.active_tests),
        }
    }

    /// Check if total elapsed time exceeds timeout
    pub fn check_elapsed(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// RAII guard for test timeout
pub struct TestTimeoutGuard {
    test_name: String,
    active_tests: Arc<Mutex<Vec<TestTimeoutHandle>>,
}

impl Drop for TestTimeoutGuard {
    fn drop(&mut self) {
        let mut active = self.active_tests.lock().unwrap();
        active.retain(|h| h.test_name != self.test_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_timeout_creation() {
        let timeout: _ = TestTimeout::default();
        assert!(timeout.config.default_timeout > Duration::from_secs(0));
    }

    #[test]
    fn test_run_with_timeout_success() {
        let timeout: _ = TestTimeout::default();
        let result: _ = timeout.run_with_timeout(Duration::from_secs(1), || 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_run_with_timeout_failure() {
        let timeout: _ = TestTimeout::default();
        let result: _ = timeout.run_with_timeout(Duration::from_millis(10), || {
            std::thread::sleep(Duration::from_millis(100));
            42
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_context() {
        let context: _ = TimeoutContext::new(Duration::from_secs(5));
        assert!(!context.check_elapsed());

        let _guard: _ = context.start_test("test1");
        assert!(!context.check_elapsed());
    }

    #[test]
    fn test_timeout_guard_drop() {
        let context: _ = TimeoutContext::new(Duration::from_secs(5));
        {
            let _guard: _ = context.start_test("test1");
            assert_eq!(context.active_tests.lock().unwrap().len(), 1);
        }
        assert_eq!(context.active_tests.lock().unwrap().len(), 0);
    }
}

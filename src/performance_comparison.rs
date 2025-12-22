//! Stub module for performance comparison
//! This module provides types for comparing performance across different runs

use std::collections::HashMap;

/// Configuration for performance reports
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub output_format: String,
    pub include_details: bool,
}

impl ReportConfig {
    /// Create a new report configuration
    pub fn new(output_format: String, include_details: bool) -> Self {
        Self {
            output_format,
            include_details,
        }
    }
}

/// Runner for performance benchmarks
#[derive(Debug, Clone)]
pub struct BenchmarkRunner {
    // Minimal implementation
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        Self {}
    }
}

/// Collector for performance results
#[derive(Debug, Clone)]
pub struct ResultCollector {
    pub results: HashMap<String, f64>,
}

impl ResultCollector {
    /// Create a new result collector
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Add a result
    pub fn add_result(&mut self, name: String, value: f64) {
        self.results.insert(name, value);
    }
}

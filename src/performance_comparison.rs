// Stub module for performance comparison
// This module provides types for comparing performance across different runs

use std::collections::HashMap;

/// Report format enum
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Html,
    Markdown,
    Json,
}

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

/// Report generator for performance results
#[derive(Debug, Clone)]
pub struct ReportGenerator {
    pub config: ReportConfig,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Generate a report
    pub fn generate(&self, format: ReportFormat) -> String {
        match format {
            ReportFormat::Html => "HTML report".to_string(),
            ReportFormat::Markdown => "# Performance Report\n\n".to_string(),
            ReportFormat::Json => "{}".to_string(),
        }
    }
}

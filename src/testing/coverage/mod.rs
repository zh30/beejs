//! Code Coverage Module
//! Stage 93 Phase 3.3 - Code Coverage Analysis
//!
//! Provides code coverage tracking and reporting with:
//! - Line coverage
//! - Branch coverage
//! - Function coverage
//! - HTML/JSON/LCOV report generation

pub mod tracker;
pub mod report_generator;

pub use tracker::*;
pub use report_generator::*;

/// Coverage statistics
#[derive(Debug, Clone, Default)]
pub struct CoverageStats {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub line_coverage: f64,
    pub total_branches: usize,
    pub covered_branches: usize,
    pub branch_coverage: f64,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub function_coverage: f64,
    pub total_files: usize,
    pub covered_files: usize,
}

/// Single file coverage data
#[derive(Debug, Clone)]
pub struct FileCoverage {
    pub file_path: String,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub line_coverage: f64,
    pub total_branches: usize,
    pub covered_branches: usize,
    pub branch_coverage: f64,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub function_coverage: f64,
    pub uncovered_lines: Vec<usize>,
    pub uncovered_branches: Vec<(usize, usize)>, // (line, branch_index)
}

/// Coverage report
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub stats: CoverageStats,
    pub files: Vec<FileCoverage>,
    pub generated_at: String,
    pub format: CoverageFormat,
}

/// Coverage format options
#[derive(Debug, Clone)]
pub enum CoverageFormat {
    Html,
    Json,
    Lcov,
    Text,
}

/// Coverage configuration
#[derive(Debug, Clone)]
pub struct CoverageConfig {
    pub enabled: bool,
    pub track_line_coverage: bool,
    pub track_branch_coverage: bool,
    pub track_function_coverage: bool,
    pub output_directory: String,
    pub output_format: CoverageFormat,
    pub threshold_line: f64,
    pub threshold_branch: f64,
    pub threshold_function: f64,
}

impl Default for CoverageConfig {
    fn default() -> Self {
        CoverageConfig {
            enabled: true,
            track_line_coverage: true,
            track_branch_coverage: true,
            track_function_coverage: true,
            output_directory: "coverage".to_string(),
            output_format: CoverageFormat::Html,
            threshold_line: 80.0,
            threshold_branch: 70.0,
            threshold_function: 90.0,
        }
    }
}

impl CoverageConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_output_format(mut self, format: CoverageFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn with_thresholds(mut self, line: f64, branch: f64, function: f64) -> Self {
        self.threshold_line = line;
        self.threshold_branch = branch;
        self.threshold_function = function;
        self
    }

    pub fn check_thresholds(&self, stats: &CoverageStats) -> bool {
        stats.line_coverage >= self.threshold_line
            && stats.branch_coverage >= self.threshold_branch
            && stats.function_coverage >= self.threshold_function
    }
}

/// Coverage writer trait
pub trait CoverageWriter {
    fn write(&self, report: &CoverageReport) -> Result<(), CoverageError>;
}

/// HTML coverage writer
pub struct HtmlCoverageWriter {
    config: CoverageConfig,
}

impl HtmlCoverageWriter {
    pub fn new(config: CoverageConfig) -> Self {
        HtmlCoverageWriter { config }
    }
}

impl CoverageWriter for HtmlCoverageWriter {
    fn write(&self, report: &CoverageReport) -> Result<(), CoverageError> {
        use std::fs;
        use std::io::Write;

        let output_dir: _ = &self.config.output_directory;
        fs::create_dir_all(output_dir)?;

        // Write index.html
        let index_path: _ = format!("{}/index.html", output_dir);
        let mut file = fs::File::create(&index_path)?;

        writeln!(file, "<!DOCTYPE html>")?;
        writeln!(file, "<html>")?;
        writeln!(file, "<head>")?;
        writeln!(file, "    <title>Coverage Report</title>")?;
        writeln!(file, "    <style>")?;
        writeln!(file, "        body {{ font-family: Arial, sans-serif; margin: 20px; }}")?;
        writeln!(file, "        table {{ border-collapse: collapse; width: 100%; }}")?;
        writeln!(file, "        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}")?;
        writeln!(file, "        th {{ background-color: #f2f2f2; }}")?;
        writeln!(file, "        .high {{ background-color: #d4edda; }}")?;
        writeln!(file, "        .medium {{ background-color:fff3cd; }}")?;
        writeln!(file, "        .low {{ background-color: #f8d7da; }}")?;
        writeln!(file, "    </style>")?;
        writeln!(file, "</head>")?;
        writeln!(file, "<body>")?;
        writeln!(file, "    <h1>Code Coverage Report</h1>")?;
        writeln!(file, "    <h2>Summary</h2>")?;
        writeln!(file, "    <table>")?;
        writeln!(file, "        <tr><th>Metric</th><th>Coverage</th><th>Total</th><th>Covered</th></tr>")?;
        writeln!(file, "        <tr><td>Lines</td><td>{:.2}%</td><td>{}</td><td>{}</td></tr>",
            report.stats.line_coverage,
            report.stats.total_lines,
            report.stats.covered_lines
        )?;
        writeln!(file, "        <tr><td>Branches</td><td>{:.2}%</td><td>{}</td><td>{}</td></tr>",
            report.stats.branch_coverage,
            report.stats.total_branches,
            report.stats.covered_branches
        )?;
        writeln!(file, "        <tr><td>Functions</td><td>{:.2}%</td><td>{}</td><td>{}</td></tr>",
            report.stats.function_coverage,
            report.stats.total_functions,
            report.stats.covered_functions
        )?;
        writeln!(file, "    </table>")?;

        writeln!(file, "    <h2>Files</h2>")?;
        writeln!(file, "    <table>")?;
        writeln!(file, "        <tr><th>File</th><th>Line Coverage</th><th>Branches</th><th>Functions</th></tr>")?;

        for file_coverage in &report.files {
            let line_class: _ = if file_coverage.line_coverage >= 80.0 {
                "high"
            } else if file_coverage.line_coverage >= 60.0 {
                "medium"
            } else {
                "low"
            };

            writeln!(file,
                "        <tr class=\"{}\"><td>{}</td><td>{:.2}%</td><td>{:.2}%</td><td>{:.2}%</td></tr>",
                line_class,
                file_coverage.file_path,
                file_coverage.line_coverage,
                file_coverage.branch_coverage,
                file_coverage.function_coverage
            )?;
        }

        writeln!(file, "    </table>")?;
        writeln!(file, "</body>")?;
        writeln!(file, "</html>")?;

        Ok(())
    }
}

/// JSON coverage writer
pub struct JsonCoverageWriter {
    config: CoverageConfig,
}

impl JsonCoverageWriter {
    pub fn new(config: CoverageConfig) -> Self {
        JsonCoverageWriter { config }
    }
}

impl CoverageWriter for JsonCoverageWriter {
    fn write(&self, report: &CoverageReport) -> Result<(), CoverageError> {
        use std::fs;

        let output_path: _ = format!("{}/coverage.json", self.config.output_directory);
        let content: _ = serde_json::to_string_pretty(report)?;
        fs::write(output_path, content)?;
        Ok(())
    }
}

/// Text coverage writer
pub struct TextCoverageWriter {
    config: CoverageConfig,
}

impl TextCoverageWriter {
    pub fn new(config: CoverageConfig) -> Self {
        TextCoverageWriter { config }
    }
}

impl CoverageWriter for TextCoverageWriter {
    fn write(&self, report: &CoverageReport) -> Result<(), CoverageError> {
        use std::fs;
        use std::io::Write;

        let output_path: _ = format!("{}/coverage.txt", self.config.output_directory);
        let mut file = fs::File::create(&output_path)?;

        writeln!(file, "Code Coverage Report")?;
        writeln!(file, "===================")?;
        writeln!(file)?;
        writeln!(file, "Summary:")?;
        writeln!(file, "  Lines: {}/{} ({:.2}%)",
            report.stats.covered_lines,
            report.stats.total_lines,
            report.stats.line_coverage
        )?;
        writeln!(file, "  Branches: {}/{} ({:.2}%)",
            report.stats.covered_branches,
            report.stats.total_branches,
            report.stats.branch_coverage
        )?;
        writeln!(file, "  Functions: {}/{} ({:.2}%)",
            report.stats.covered_functions,
            report.stats.total_functions,
            report.stats.function_coverage
        )?;
        writeln!(file)?;
        writeln!(file, "Files:")?;

        for file_coverage in &report.files {
            writeln!(file, "  {}: {:.2}% line, {:.2}% branch, {:.2}% function",
                file_coverage.file_path,
                file_coverage.line_coverage,
                file_coverage.branch_coverage,
                file_coverage.function_coverage
            )?;
        }

        Ok(())
    }
}

/// Coverage error
#[derive(Debug)]
pub enum CoverageError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    FileNotFound(String),
}

impl std::fmt::Display for CoverageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverageError::IoError(err) => write!(f, "I/O error: {}", err),
            CoverageError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            CoverageError::FileNotFound(path) => write!(f, "File not found: {}", path),
        }
    }
}

impl std::error::Error for CoverageError {}

impl From<std::io::Error> for CoverageError {
    fn from(err: std::io::Error) -> Self {
        CoverageError::IoError(err)
    }
}

impl From<serde_json::Error> for CoverageError {
    fn from(err: serde_json::Error) -> Self {
        CoverageError::SerializationError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_coverage_stats_default() {
        let stats: _ = CoverageStats::default();
        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.covered_lines, 0);
        assert_eq!(stats.line_coverage, 0.0);
    }

    #[test]
    fn test_file_coverage() {
        let file_coverage: _ = FileCoverage {
            file_path: "test.rs".to_string(),
            total_lines: 100,
            covered_lines: 80,
            line_coverage: 80.0,
            total_branches: 20,
            covered_branches: 15,
            branch_coverage: 75.0,
            total_functions: 10,
            covered_functions: 9,
            function_coverage: 90.0,
            uncovered_lines: vec![10, 20, 30],
            uncovered_branches: vec![(15, 1), (25, 0)],
        };

        assert_eq!(file_coverage.file_path, "test.rs");
        assert_eq!(file_coverage.line_coverage, 80.0);
        assert_eq!(file_coverage.branch_coverage, 75.0);
        assert_eq!(file_coverage.function_coverage, 90.0);
    }

    #[test]
    fn test_coverage_config() {
        let config: _ = CoverageConfig::new()
            .with_enabled(false)
            .with_thresholds(90.0, 80.0, 95.0);

        assert!(!config.enabled);
        assert_eq!(config.threshold_line, 90.0);
        assert_eq!(config.threshold_branch, 80.0);
        assert_eq!(config.threshold_function, 95.0);
    }

    #[test]
    fn test_html_coverage_writer() {
        let temp_dir: _ = tempfile::tempdir().unwrap();
        let mut config = CoverageConfig::default();
        config.output_directory = temp_dir.path().to_string_lossy().to_string();

        let report: _ = CoverageReport {
            stats: CoverageStats {
                total_lines: 100,
                covered_lines: 80,
                line_coverage: 80.0,
                total_branches: 20,
                covered_branches: 15,
                branch_coverage: 75.0,
                total_functions: 10,
                covered_functions: 9,
                function_coverage: 90.0,
                total_files: 1,
                covered_files: 1,
            },
            files: vec![FileCoverage {
                file_path: "test.rs".to_string(),
                total_lines: 100,
                covered_lines: 80,
                line_coverage: 80.0,
                total_branches: 20,
                covered_branches: 15,
                branch_coverage: 75.0,
                total_functions: 10,
                covered_functions: 9,
                function_coverage: 90.0,
                uncovered_lines: vec![],
                uncovered_branches: vec![],
            }],
            generated_at: "2023-01-01".to_string(),
            format: CoverageFormat::Html,
        };

        let writer: _ = HtmlCoverageWriter::new(config);
        let result: _ = writer.write(&report);
        assert!(result.is_ok());

        let index_path: _ = temp_dir.path().join("index.html");
        assert!(index_path.exists());
    }
}

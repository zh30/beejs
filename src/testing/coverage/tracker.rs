//! Coverage Tracker
//! Tracks code coverage during test execution

use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Coverage tracking configuration
#[derive(Debug, Clone)]
pub struct CoverageTrackingConfig {
    pub track_line_coverage: bool,
    pub track_branch_coverage: bool,
    pub track_function_coverage: bool,
    pub collect_uncovered_lines: bool,
    pub collect_uncovered_branches: bool,
}

impl Default for CoverageTrackingConfig {
    fn default() -> Self {
        CoverageTrackingConfig {
            track_line_coverage: true,
            track_branch_coverage: true,
            track_function_coverage: true,
            collect_uncovered_lines: true,
            collect_uncovered_branches: true,
        }
    }
}

/// Line coverage tracking
#[derive(Debug, Clone, Default)]
pub struct LineCoverage {
    pub total_lines: usize,
    pub covered_lines: HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32, std::collections::HashMap<usize, u32, std::collections::HashMap<usize, u32, usize, u32, usize, u32, std::collections::HashMap<usize, u32, usize, u32>>>>>>>, // line_number -> hit_count
}

impl LineCoverage {
    pub fn new(total_lines: usize) -> Self {
        LineCoverage {
            total_lines,
            covered_lines: HashMap::new(),
        }
    }

    /// Mark a line as covered
    pub fn mark_covered(&mut self, line_number: usize) {
        if line_number > 0 && line_number <= self.total_lines {
            *self.covered_lines.entry(line_number).or_insert(0) += 1;
        }
    }

    /// Get coverage percentage
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            return 0.0;
        }
        (self.covered_lines.len() as f64 / self.total_lines as f64) * 100.0
    }

    /// Get uncovered lines
    pub fn uncovered_lines(&self) -> Vec<usize> {
        let mut uncovered = Vec::new();
        for line in 1..=self.total_lines {
            if !self.covered_lines.contains_key(&line) {
                uncovered.push(line);
            }
        }
        uncovered
    }
}

/// Branch coverage tracking
#[derive(Debug, Clone, Default)]
pub struct BranchCoverage {
    pub total_branches: HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize, std::collections::HashMap<usize, usize, std::collections::HashMap<usize, usize, usize, usize, usize, usize, std::collections::HashMap<usize, usize, usize, usize>>>>>>>, // line_number -> branch_count
    pub covered_branches: HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32, (usize, usize), u32, std::collections::HashMap<(usize, usize), u32, (usize, usize), u32>>>>>>>, // (line_number, branch_index) -> hit_count
}

impl BranchCoverage {
    pub fn new() -> Self {
        BranchCoverage {
            total_branches: HashMap::new(),
            covered_branches: HashMap::new(),
        }
    }

    /// Add a branch at a line
    pub fn add_branch(&mut self, line_number: usize, branch_index: usize) {
        *self.total_branches.entry(line_number).or_insert(0) = branch_index + 1;
    }

    /// Mark a branch as covered
    pub fn mark_covered(&mut self, line_number: usize, branch_index: usize) {
        *self.covered_branches.entry((line_number, branch_index)).or_insert(0) += 1;
    }

    /// Get coverage percentage
    pub fn coverage_percentage(&self) -> f64 {
        let total: usize = self.total_branches.values().sum();
        if total == 0 {
            return 0.0;
        }
        (self.covered_branches.len() as f64 / total as f64) * 100.0
    }

    /// Get uncovered branches
    pub fn uncovered_branches(&self) -> Vec<(usize, usize)> {
        let mut uncovered = Vec::new();
        for (&line, &count) in &self.total_branches {
            for branch_index in 0..count {
                if !self.covered_branches.contains_key(&(line, branch_index)) {
                    uncovered.push((line, branch_index));
                }
            }
        }
        uncovered
    }
}

/// Function coverage tracking
#[derive(Debug, Clone, Default)]
pub struct FunctionCoverage {
    pub total_functions: HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize>>>>>>>, // function_name -> line_number
    pub covered_functions: HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32, std::collections::HashMap<String, u32, std::collections::HashMap<String, u32, String, u32, String, u32, std::collections::HashMap<String, u32, String, u32>>>>>>>, // function_name -> hit_count
}

impl FunctionCoverage {
    pub fn new() -> Self {
        FunctionCoverage {
            total_functions: HashMap::new(),
            covered_functions: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register_function(&mut self, function_name: String, line_number: usize) {
        self.total_functions.entry(function_name.clone()).or_insert(line_number);
    }

    /// Mark a function as covered
    pub fn mark_covered(&mut self, function_name: &str) {
        *self.covered_functions.entry(function_name.to_string()).or_insert(0) += 1;
    }

    /// Get coverage percentage
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_functions.is_empty() {
            return 0.0;
        }
        (self.covered_functions.len() as f64 / self.total_functions.len() as f64) * 100.0
    }

    /// Get uncovered functions
    pub fn uncovered_functions(&self) -> Vec<String> {
        let mut uncovered = Vec::new();
        for (func_name, _) in &self.total_functions {
            if !self.covered_functions.contains_key(func_name) {
                uncovered.push(func_name.clone());
            }
        }
        uncovered
    }
}

/// Per-file coverage data
#[derive(Debug, Clone, Default)]
pub struct PerFileCoverage {
    pub file_path: String,
    pub line_coverage: LineCoverage,
    pub branch_coverage: BranchCoverage,
    pub function_coverage: FunctionCoverage,
}

impl PerFileCoverage {
    pub fn new(file_path: String) -> Self {
        PerFileCoverage {
            file_path,
            line_coverage: LineCoverage::default(),
            branch_coverage: BranchCoverage::new(),
            function_coverage: FunctionCoverage::new(),
        }
    }

    /// Get combined coverage statistics
    pub fn get_stats(&self) -> (usize, usize, usize, usize, usize, usize, usize, usize) {
        let total_lines: _ = self.line_coverage.total_lines;
        let covered_lines: _ = self.line_coverage.covered_lines.len();
        let total_branches: usize = self.branch_coverage.total_branches.values().sum();
        let covered_branches: _ = self.branch_coverage.covered_branches.len();
        let total_functions: _ = self.function_coverage.total_functions.len();
        let covered_functions: _ = self.function_coverage.covered_functions.len();

        (
            total_lines,
            covered_lines,
            total_branches,
            covered_branches,
            total_functions,
            covered_functions,
            self.line_coverage.uncovered_lines().len(),
            self.branch_coverage.uncovered_branches().len(),
        )
    }
}

/// Global coverage tracker
pub struct CoverageTracker {
    config: CoverageTrackingConfig,
    files: Arc<Mutex<HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage>>>>>>>,
}

impl CoverageTracker {
    pub fn new(config: CoverageTrackingConfig) -> Self {
        CoverageTracker {
            config,
            files: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(HashMap::new())))),
        }
    }

    /// Create a new coverage tracker with default config
    pub fn default() -> Self {
        Self::new(CoverageTrackingConfig::default())
    }

    /// Register a file
    pub fn register_file(&self, file_path: String) {
        let mut files = self.files.lock().unwrap();
        if !files.contains_key(&file_path) {
            files.insert(file_path, PerFileCoverage::new(file_path));
        }
    }

    /// Mark a line as covered
    pub fn mark_line_covered(&self, file_path: &str, line_number: usize) {
        if !self.config.track_line_coverage {
            return;
        }

        let mut files = self.files.lock().unwrap();
        if let Some(file_coverage) = files.get_mut(file_path) {
            file_coverage.line_coverage.mark_covered(line_number);
        }
    }

    /// Add a branch
    pub fn add_branch(&self, file_path: &str, line_number: usize, branch_index: usize) {
        if !self.config.track_branch_coverage {
            return;
        }

        let mut files = self.files.lock().unwrap();
        if let Some(file_coverage) = files.get_mut(file_path) {
            file_coverage.branch_coverage.add_branch(line_number, branch_index);
        }
    }

    /// Mark a branch as covered
    pub fn mark_branch_covered(&self, file_path: &str, line_number: usize, branch_index: usize) {
        if !self.config.track_branch_coverage {
            return;
        }

        let mut files = self.files.lock().unwrap();
        if let Some(file_coverage) = files.get_mut(file_path) {
            file_coverage.branch_coverage.mark_covered(line_number, branch_index);
        }
    }

    /// Register a function
    pub fn register_function(&self, file_path: &str, function_name: String, line_number: usize) {
        if !self.config.track_function_coverage {
            return;
        }

        let mut files = self.files.lock().unwrap();
        if let Some(file_coverage) = files.get_mut(file_path) {
            file_coverage.function_coverage.register_function(function_name, line_number);
        }
    }

    /// Mark a function as covered
    pub fn mark_function_covered(&self, file_path: &str, function_name: &str) {
        if !self.config.track_function_coverage {
            return;
        }

        let mut files = self.files.lock().unwrap();
        if let Some(file_coverage) = files.get_mut(file_path) {
            file_coverage.function_coverage.mark_covered(function_name);
        }
    }

    /// Get coverage for a specific file
    pub fn get_file_coverage(&self, file_path: &str) -> Option<PerFileCoverage> {
        let files: _ = self.files.lock().unwrap();
        files.get(file_path).cloned()
    }

    /// Get all coverage data
    pub fn get_all_coverage(&self) -> HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage, String, PerFileCoverage, std::collections::HashMap<String, PerFileCoverage, String, PerFileCoverage>>>>>>> {
        let files: _ = self.files.lock().unwrap();
        files.clone()
    }

    /// Get overall coverage statistics
    pub fn get_overall_stats(&self) -> CoverageStats {
        let files: _ = self.files.lock().unwrap();

        let mut total_lines = 0;
        let mut covered_lines = 0;
        let mut total_branches = 0;
        let mut covered_branches = 0;
        let mut total_functions = 0;
        let mut covered_functions = 0;
        let mut covered_files = 0;

        for file_coverage in files.values() {
            let (t_lines, c_lines, t_branches, c_branches, t_funcs, c_funcs, _, _) =
                file_coverage.get_stats();

            total_lines += t_lines;
            covered_lines += c_lines;
            total_branches += t_branches;
            covered_branches += c_branches;
            total_functions += t_funcs;
            covered_functions += c_funcs;

            if t_lines > 0 && c_lines == t_lines {
                covered_files += 1;
            }
        }

        CoverageStats {
            total_lines,
            covered_lines,
            line_coverage: if total_lines > 0 {
                (covered_lines as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            },
            total_branches,
            covered_branches,
            branch_coverage: if total_branches > 0 {
                (covered_branches as f64 / total_branches as f64) * 100.0
            } else {
                0.0
            },
            total_functions,
            covered_functions,
            function_coverage: if total_functions > 0 {
                (covered_functions as f64 / total_functions as f64) * 100.0
            } else {
                0.0
            },
            total_files: files.len(),
            covered_files,
        }
    }

    /// Clear all coverage data
    pub fn clear(&self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
    }

    /// Reset coverage for a specific file
    pub fn reset_file(&self, file_path: &str) {
        let mut files = self.files.lock().unwrap();
        files.remove(file_path);
    }
}

/// Global coverage tracker instance
static GLOBAL_TRACKER: once_cell::sync::OnceCell<Arc<CoverageTracker>> =
    once_cell::sync::OnceCell::new();

/// Initialize global coverage tracker
pub fn init_global_tracker(config: CoverageTrackingConfig) -> Arc<CoverageTracker> {
    let tracker: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(CoverageTracker::new(config)))));
    GLOBAL_TRACKER.set(tracker.clone()).ok();
    tracker
}

/// Get global coverage tracker
pub fn get_global_tracker() -> Option<Arc<CoverageTracker>> {
    GLOBAL_TRACKER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_line_coverage() {
        let mut line_coverage = LineCoverage::new(10);
        line_coverage.mark_covered(1);
        line_coverage.mark_covered(2);
        line_coverage.mark_covered(3);

        assert_eq!(line_coverage.coverage_percentage(), 30.0);
        assert_eq!(line_coverage.uncovered_lines(), vec![4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_branch_coverage() {
        let mut branch_coverage = BranchCoverage::new();
        branch_coverage.add_branch(5, 0);
        branch_coverage.add_branch(5, 1);
        branch_coverage.mark_covered(5, 0);

        assert_eq!(branch_coverage.coverage_percentage(), 50.0);
        assert_eq!(branch_coverage.uncovered_branches(), vec![(5, 1)]);
    }

    #[test]
    fn test_function_coverage() {
        let mut function_coverage = FunctionCoverage::new();
        function_coverage.register_function("test_func".to_string(), 10);
        function_coverage.register_function("another_func".to_string(), 20);
        function_coverage.mark_covered("test_func");

        assert_eq!(function_coverage.coverage_percentage(), 50.0);
        assert_eq!(function_coverage.uncovered_functions(), vec!["another_func".to_string()]);
    }

    #[test]
    fn test_coverage_tracker() {
        let tracker: _ = CoverageTracker::default();

        tracker.register_file("test.rs".to_string());
        tracker.mark_line_covered("test.rs", 1);
        tracker.mark_line_covered("test.rs", 2);

        let stats: _ = tracker.get_overall_stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.covered_lines, 2);
    }

    #[test]
    fn test_global_tracker() {
        let config: _ = CoverageTrackingConfig::default();
        let tracker: _ = init_global_tracker(config);

        tracker.register_file("global_test.rs".to_string());
        tracker.mark_line_covered("global_test.rs", 1);

        let global: _ = get_global_tracker();
        assert!(global.is_some());

        let stats: _ = global.unwrap().get_overall_stats();
        assert_eq!(stats.covered_lines, 1);
    }
}

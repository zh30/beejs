//! Enhanced Debugger Module
//!
//! Provides advanced debugging capabilities including:
//! - Visual debugging interface
//! - Performance profiling
//! - Memory analysis
//! - Hot reload

pub mod ui;
pub mod inspector;

pub use ui::{
    BreakpointManager, VariableInspector, CallStackView, Repl, DebuggerUI,
    Breakpoint, BreakpointCondition, Variable, Scope, StackFrame
};
pub use inspector::{
    HeapSnapshot, ObjectTracer, MemoryAnalyzer, HeapStats, SnapshotDiff, MemoryLeak
};

/// Performance profiler
pub struct PerformanceProfiler {
    active: bool,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub async fn start_profiling(&mut self) -> Result<()> {
        self.active = true;
        Ok(())
    }

    pub async fn stop_profiling(&self) -> Result<PerformanceReport> {
        Ok(PerformanceReport {
            total_duration: 100, // Mock duration in ms
            function_counts: HashMap::new(),
        })
    }
}

/// Performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub total_duration: u64,
    pub function_counts: std::collections::HashMap<String, usize>,
}

/// Hot reload manager
pub struct HotReload {
    watched_files: std::collections::HashSet<String>,
}

impl HotReload {
    pub fn new() -> Self {
        Self {
            watched_files: std::collections::HashSet::new(),
        }
    }

    pub async fn watch_file(&mut self, file: &str) -> Result<()> {
        self.watched_files.insert(file.to_string());
        Ok(())
    }

    pub async fn unwatch_file(&mut self, file: &str) -> Result<()> {
        self.watched_files.remove(file);
        Ok(())
    }

    pub async fn is_watching(&self, file: &str) -> bool {
        self.watched_files.contains(file)
    }

    pub async fn notify_change(&mut self, file: &str) -> Result<()> {
        // TODO: Notify watchers
        Ok(())
    }
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    function_timings: std::collections::HashMap<String, u64>,
    memory_peak: usize,
    gc_count: u32,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            function_timings: std::collections::HashMap::new(),
            memory_peak: 0,
            gc_count: 0,
        }
    }

    pub async fn record_function_time(&mut self, function: &str, duration: std::time::Duration) {
        self.function_timings.insert(function.to_string(), duration.as_millis() as u64);
    }

    pub async fn record_memory_usage(&mut self, bytes: usize) {
        if bytes > self.memory_peak {
            self.memory_peak = bytes;
        }
    }

    pub async fn record_gc_event(&mut self, duration: std::time::Duration, bytes_freed: usize) {
        self.gc_count += 1;
        if bytes_freed > self.memory_peak {
            self.memory_peak = bytes_freed;
        }
    }

    pub async fn get_collected_metrics(&self) -> CollectedMetrics {
        CollectedMetrics {
            function_timings: self.function_timings.clone(),
            memory_peak: self.memory_peak,
            gc_count: self.gc_count,
        }
    }
}

/// Collected performance metrics
#[derive(Debug, Clone)]
pub struct CollectedMetrics {
    pub function_timings: std::collections::HashMap<String, u64>,
    pub memory_peak: usize,
    pub gc_count: u32,
}

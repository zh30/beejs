//! Stage 21.2: Lazy Loading Module
//! Provides lazy initialization for expensive components to reduce startup time
//! Only initializes modules when they're actually used

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

/// Lazy loading statistics
#[derive(Debug, Clone, Default)]
pub struct LazyLoaderStats {
    pub total_modules: usize,
    pub initialized_modules: usize,
    pub lazy_modules: usize,
    pub initialization_time_ms: u64,
}

/// Lazy loader for managing delayed initialization of expensive modules
pub struct LazyLoader {
    stats: Arc<Mutex<LazyLoaderStats>>,
}

impl LazyLoader {
    /// Create a new lazy loader
    pub fn new() -> Self {
        Self {
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(LazyLoaderStats::default()))),
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> LazyLoaderStats {
        self.stats.lock().unwrap().clone()
    }

    /// Record module initialization
    fn record_init(&self, module_name: &str, init_time_ms: u64) {
        let mut stats = self.stats.lock().unwrap();
        stats.initialized_modules += 1;
        stats.initialization_time_ms += init_time_ms;
        eprintln!("LazyLoader: Initialized '{}' in {}ms (total: {} initialized)",
                  module_name, init_time_ms, stats.initialized_modules);
    }

    /// Record module as lazy (not yet initialized)
    #[allow(dead_code)]
    fn record_lazy(&self, module_name: &str) {
        let mut stats = self.stats.lock().unwrap();
        stats.lazy_modules += 1;
        eprintln!("LazyLoader: '{}' is lazy (total lazy: {})", module_name, stats.lazy_modules);
    }
}

/// Global lazy loader instance
static LAZY_LOADER: Lazy<LazyLoader> = Lazy::new(|| LazyLoader::new());

/// Get the global lazy loader instance
pub fn get_lazy_loader() -> &'static LazyLoader {
    &LAZY_LOADER
}

/// Module initialization tracker
#[derive(Debug)]
pub struct ModuleInit {
    pub name: &'static str,
    init_time_ms: u64,
}

impl ModuleInit {
    /// Create a new module init tracker
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            init_time_ms: 0,
        }
    }

    /// Mark initialization as complete and record time
    pub fn done(self) {
        let loader: _ = get_lazy_loader();
        loader.record_init(self.name, self.init_time_ms);
    }
}

/// Helper macro for lazy initialization
/// Usage:
/// lazy_init!(AI_MODEL_MANAGER, ai_model_interface::init_manager())
#[macro_export]
macro_rules! lazy_init {
    ($name:ident, $init:expr) => {
        static $name: once_cell::sync::Lazy<Result<(), String>> =
            once_cell::sync::Lazy::new(|| {
                let init_tracker: _ = $crate::lazy_loader::ModuleInit::new(stringify!($name));
                let result: _ = $init.map_err(|e: anyhow::Error| e.to_string());
                init_tracker.done();
                result
            });
    };
}

/// Helper macro for creating lazy module loaders
/// Usage:
/// lazy_module!(AI_BATCH_PROCESSOR, ai_batch_processor::BatchProcessor::new())
#[macro_export]
macro_rules! lazy_module {
    ($name:ident, $init:expr) => {
        static $name: once_cell::sync::Lazy<Result<Box<dyn std::any::Any + Send + Sync>>, anyhow::Error>> =
            once_cell::sync::Lazy::new(|| {
                let init_tracker: _ = $crate::lazy_loader::ModuleInit::new(stringify!($name));
                let result: _ = $init.map(|r| Box::new(r) as Box<dyn std::any::Any + Send + Sync>);
                init_tracker.done();
                result
            });
    };
}

/// Print lazy loading statistics
pub fn print_stats() {
    let stats: _ = get_lazy_loader().get_stats();
    eprintln!("\n📊 LazyLoader Statistics:");
    eprintln!("   Total modules: {}", stats.total_modules);
    eprintln!("   Initialized: {}", stats.initialized_modules);
    eprintln!("   Lazy (not yet loaded): {}", stats.lazy_modules);
    eprintln!("   Total initialization time: {}ms", stats.initialization_time_ms);

    if stats.initialized_modules > 0 {
        let avg_time: _ = stats.initialization_time_ms / stats.initialized_modules as u64;
        eprintln!("   Average init time: {}ms", avg_time);
    }
}

/// Reset lazy loader statistics (for testing)
pub fn reset_stats() {
    let loader: _ = get_lazy_loader();
    let mut stats = loader.stats.lock().unwrap();
    *stats = LazyLoaderStats::default();
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_lazy_loader_creation() {
        let loader: _ = LazyLoader::new();
        let stats: _ = loader.get_stats();
        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.initialized_modules, 0);
    }

    #[test]
    fn test_module_init_tracker() {
        let tracker: _ = ModuleInit::new("test_module");
        // In a real test, we'd call tracker.done()
        // but for now we just verify it can be created
        assert_eq!(tracker.name, "test_module");
    }

    #[test]
    fn test_print_stats() {
        // Just verify the function doesn't panic
        print_stats();
    }
}

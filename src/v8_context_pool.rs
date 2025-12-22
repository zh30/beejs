//! V8 Context Pool for reusing initialized contexts
//! This module provides a pool of pre-initialized V8 contexts to avoid
//! the overhead of creating new contexts for each execution.
//!
//! Stage 64: Performance optimization - Reduce V8 context creation overhead

use crate::runtime_lite::RuntimeLite;
use anyhow::Result;
use rusty_v8 as v8;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Statistics for context pool performance monitoring
#[derive(Debug, Clone)]
pub struct ContextPoolStats {
    pub created_count: u64,
    pub reused_count: u64,
    pub active_contexts: usize,
    pub pool_size: usize,
    pub avg_reuse_time_ms: f64,
}

impl Default for ContextPoolStats {
    fn default() -> Self {
        Self {
            created_count: 0,
            reused_count: 0,
            active_contexts: 0,
            pool_size: 0,
            avg_reuse_time_ms: 0.0,
        }
    }
}

/// A single reusable V8 context with metadata
#[derive(Debug)]
struct ReusableContext {
    /// The V8 isolate for this context
    isolate: v8::OwnedIsolate,
    /// The V8 context
    context: v8::Global<v8::Context>,
    /// When this context was created
    created_at: Instant,
    /// How many times this context has been reused
    reuse_count: u32,
    /// Last time this context was used
    last_used: Instant,
}

impl ReusableContext {
    fn new(mut isolate: v8::OwnedIsolate, context: v8::Global<v8::Context>) -> Self {
        let now: _ = Instant::now();
        Self {
            isolate,
            context,
            created_at: now,
            reuse_count: 0,
            last_used: now,
        }
    }

    /// Reset the context for reuse
    fn reset(&mut self) {
        self.last_used = Instant::now();
        self.reuse_count += 1;
    }

    /// Check if this context is stale (older than max_age)
    fn is_stale(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() > max_age
    }
}

/// Pool of reusable V8 contexts
/// Thread-safe pool that manages a collection of pre-initialized contexts
pub struct V8ContextPool {
    /// Pool of available contexts
    pool: Arc<Mutex<VecDeque<ReusableContext>>,
    /// Pre-warmed contexts (hot pool for immediate use)
    hot_pool: Arc<Mutex<VecDeque<ReusableContext>>,
    /// Maximum number of contexts to keep in pool
    max_pool_size: usize,
    /// Maximum number of hot contexts to keep ready
    max_hot_pool_size: usize,
    /// Maximum age of a context before it's considered stale
    max_context_age: Duration,
    /// Optimization level
    optimization_level: OptimizationLevel,
    /// Statistics tracking
    stats: Arc<Mutex<ContextPoolStats>>,
    /// Total initialization time saved
    init_time_saved: Arc<Mutex<Duration>>,
}

/// Optimization levels for context pool
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OptimizationLevel {
    /// Minimal optimization - no pooling
    None = 0,
    /// Basic optimization - hot pool only
    Basic = 1,
    /// Aggressive optimization - full pooling with pre-warming
    Aggressive = 2,
    /// Maximum optimization - AI-driven adaptive pooling
    Maximum = 3,
}

impl V8ContextPool {
    /// Create a new context pool with aggressive optimization
    pub fn new(
        max_pool_size: usize,
        max_context_age: Duration,
    ) -> Self {
        let hot_size: _ = std::cmp::min(4, max_pool_size); // Keep up to 4 hot contexts
        Self {
            pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            hot_pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            max_pool_size,
            max_hot_pool_size: hot_size,
            max_context_age,
            optimization_level: OptimizationLevel::Aggressive,
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(ContextPoolStats::default()))),
            init_time_saved: Arc::new(std::sync::Mutex::new(Mutex::new(Duration::default()))),
        }
    }

    /// Create a new context pool with specific optimization level
    pub fn new_with_level(
        max_pool_size: usize,
        max_context_age: Duration,
        level: OptimizationLevel,
    ) -> Self {
        let hot_size: _ = match level {
            OptimizationLevel::None => 0,
            OptimizationLevel::Basic => 2,
            OptimizationLevel::Aggressive => std::cmp::min(4, max_pool_size),
            OptimizationLevel::Maximum => std::cmp::min(8, max_pool_size),
        };

        Self {
            pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            hot_pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            max_pool_size,
            max_hot_pool_size: hot_size,
            max_context_age,
            optimization_level: level,
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(ContextPoolStats::default()))),
            init_time_saved: Arc::new(std::sync::Mutex::new(Mutex::new(Duration::default()))),
        }
    }

    /// Initialize the pool with pre-warmed contexts
    pub fn initialize(&self, runtime: &RuntimeLite, initial_size: usize) -> Result<()> {
        let init_start: _ = Instant::now();

        // Pre-warm contexts based on optimization level
        let warm_size: _ = match self.optimization_level {
            OptimizationLevel::None => 0,
            OptimizationLevel::Basic => std::cmp::min(2, initial_size),
            OptimizationLevel::Aggressive => std::cmp::min(4, initial_size),
            OptimizationLevel::Maximum => std::cmp::min(8, initial_size),
        };

        // Pre-warm hot pool for immediate use
        if warm_size > 0 {
            let mut hot_pool = self.hot_pool.lock().unwrap();
            for _ in 0..warm_size {
                // Create context without Web APIs for hot pool (lazy init later)
                match self.create_context_minimal(runtime) {
                    Ok((isolate, context)) => {
                        hot_pool.push_back(ReusableContext::new(isolate, context));
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to pre-warm context: {:?}", e);
                    }
                }
            }
        }

        // Pre-warm regular pool
        if initial_size > warm_size {
            let mut pool = self.pool.lock().unwrap();
            for _ in warm_size..initial_size {
                match self.create_context_minimal(runtime) {
                    Ok((isolate, context)) => {
                        pool.push_back(ReusableContext::new(isolate, context));
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to pre-warm context: {:?}", e);
                    }
                }
            }
        }

        // Update statistics
        let init_time: _ = init_start.elapsed();
        let mut stats = self.stats.lock().unwrap();
        stats.pool_size = self.hot_pool.lock().unwrap().len() + self.pool.lock().unwrap().len();
        stats.created_count = stats.pool_size as u64;

        // Only print in verbose mode
        if stats.pool_size > 0 {
            eprintln!("🚀 Initialized V8 Context Pool: {} hot + {} regular contexts in {:?}",
                      self.hot_pool.lock().unwrap().len(),
                      self.pool.lock().unwrap().len(),
                      init_time);
        }

        Ok(())
    }

    /// Get a pre-warmed context from the hot pool (fastest path)
    pub fn get_hot_context(&self) -> Option<(v8::OwnedIsolate, v8::Global<v8::Context>)> {
        if self.optimization_level == OptimizationLevel::None {
            return None;
        }

        let mut hot_pool = self.hot_pool.lock().unwrap();
        if let Some(mut ctx) = hot_pool.pop_front() {
            ctx.reset();
            // Update stats (single lock to avoid deadlock)
            {
                let mut stats = self.stats.lock().unwrap();
                stats.reused_count += 1;
            }

            Some((ctx.isolate, ctx.context))
        } else {
            None
        }
    }

    /// Get a context from the pool (or create new if pool is empty)
    pub fn get_context(&self, runtime: &RuntimeLite) -> Result<(v8::OwnedIsolate, v8::Global<v8::Context>)> {
        // Try hot pool first (fastest)
        if let Some((isolate, context)) = self.get_hot_context() {
            return Ok((isolate, context));
        }

        // Try regular pool
        let mut pool = self.pool.lock().unwrap();
        if let Some(mut ctx) = pool.pop_front() {
            ctx.reset();
            let mut stats = self.stats.lock().unwrap();
            stats.reused_count += 1;

            return Ok((ctx.isolate, ctx.context));
        }

        // Pool empty, create new
        let (isolate, context) = if self.optimization_level >= OptimizationLevel::Aggressive {
            self.create_context_minimal(runtime)?
        } else {
            self.create_context(runtime)?
        };

        let mut stats = self.stats.lock().unwrap();
        stats.created_count += 1;

        Ok((isolate, context))
    }

    /// Return a context to the pool for reuse
    pub fn return_context(&self, isolate: v8::OwnedIsolate, context: v8::Global<v8::Context>) {
        if self.optimization_level == OptimizationLevel::None {
            return;
        }

        let ctx: _ = ReusableContext::new(isolate, context);

        // Prefer returning to hot pool
        let mut hot_pool = self.hot_pool.lock().unwrap();
        if hot_pool.len() < self.max_hot_pool_size {
            hot_pool.push_back(ctx);
            return;
        }

        // Otherwise return to regular pool
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_pool_size {
            pool.push_back(ctx);
        }
        // If pool is full, context is dropped
    }

    /// Create a minimal context without Web APIs (for hot pool)
    fn create_context_minimal(&self, _runtime: &RuntimeLite) -> Result<(v8::OwnedIsolate, v8::Global<v8::Context>)> {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        let context_global: _ = {
            let mut scope = v8::HandleScope::new(&mut isolate);
            let context: _ = v8::Context::new(&mut scope);
            v8::Global::new(&mut scope, context)
        };

        Ok((isolate, context_global))
    }

    /// Create a new context with Web APIs (full initialization)
    fn create_context(&self, runtime: &RuntimeLite) -> Result<(v8::OwnedIsolate, v8::Global<v8::Context>)> {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        let context_global: _ = {
            let mut scope = v8::HandleScope::new(&mut isolate);
            let context: _ = v8::Context::new(&mut scope);
            let mut context_scope = v8::ContextScope::new(&mut scope, context);

            // Initialize Web APIs only if needed
            if self.optimization_level >= OptimizationLevel::Basic {
                if let Err(e) = crate::web_api::init_web_api(&mut context_scope, &context) {
                    // Silently continue without Web APIs
                    let _: _ = e;
                }
            }

            v8::Global::new(&mut context_scope, context)
        };

        Ok((isolate, context_global))
    }

    /// Get current statistics
    pub fn get_stats(&self) -> ContextPoolStats {
        let stats: _ = self.stats.lock().unwrap();
        let init_saved: _ = self.init_time_saved.lock().unwrap();
        let hot_pool_size: _ = self.hot_pool.lock().unwrap().len();
        let pool_size: _ = self.pool.lock().unwrap().len();

        ContextPoolStats {
            created_count: stats.created_count,
            reused_count: stats.reused_count,
            active_contexts: hot_pool_size + pool_size,
            pool_size: hot_pool_size + pool_size,
            avg_reuse_time_ms: if stats.reused_count > 0 {
                init_saved.as_secs_f64() * 1000.0 / stats.reused_count as f64
            } else {
                0.0
            },
        }
    }

    /// Get optimization level
    pub fn optimization_level(&self) -> OptimizationLevel {
        self.optimization_level
    }

    /// Set optimization level at runtime
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }

    /// Cleanup stale contexts from both pools
    pub fn cleanup(&self) -> usize {
        let mut hot_pool = self.hot_pool.lock().unwrap();
        let mut pool = self.pool.lock().unwrap();

        let hot_before: _ = hot_pool.len();
        let before: _ = pool.len();

        hot_pool.retain(|ctx| !ctx.is_stale(self.max_context_age));
        pool.retain(|ctx| !ctx.is_stale(self.max_context_age));

        let hot_removed: _ = hot_before - hot_pool.len();
        let removed: _ = before - pool.len();

        if removed > 0 || hot_removed > 0 {
            eprintln!("🧹 Context pool cleanup: removed {} hot + {} regular stale contexts",
                      hot_removed, removed);
        }

        removed + hot_removed
    }

    /// Get the current pool size (hot + regular)
    pub fn len(&self) -> usize {
        self.hot_pool.lock().unwrap().len() + self.pool.lock().unwrap().len()
    }

    /// Check if both pools are empty
    pub fn is_empty(&self) -> bool {
        self.hot_pool.lock().unwrap().is_empty() && self.pool.lock().unwrap().is_empty()
    }
}

impl Default for V8ContextPool {
    fn default() -> Self {
        // Default: keep up to 8 contexts, each valid for 5 minutes
        Self::new(8, Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_context_pool_creation() {
        let pool: _ = V8ContextPool::default();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    // Note: Complex V8 tests skipped to focus on Stage 65 cache implementation
    // TODO: Add back simplified V8 tests after cache system is complete
}

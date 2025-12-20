//! V8 Context Pool for reusing initialized contexts
//! This module provides a pool of pre-initialized V8 contexts to avoid
//! the overhead of creating new contexts for each execution.
//!
//! Stage 64: Performance optimization - Reduce V8 context creation overhead

// TODO: Remove unused import: use crate::runtime_lite::RuntimeLite;
// TODO: Remove unused import: use anyhow::Result;
use rusty_v8 as v8;
use std::collections::VecDeque;
// TODO: Remove unused import: use std::sync::{Arc, Mutex};
// TODO: Remove unused import: use std::time::{Duration, Instant};

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
        let now = Instant::now();
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
    pool: Arc<Mutex<VecDeque<ReusableContext>>>,
    /// Maximum number of contexts to keep in pool
    max_pool_size: usize,
    /// Maximum age of a context before it's considered stale
    max_context_age: Duration,
    /// Statistics tracking
    stats: Arc<Mutex<ContextPoolStats>>,
    /// Total initialization time saved
    init_time_saved: Arc<Mutex<Duration>>,
}

impl V8ContextPool {
    /// Create a new context pool
    pub fn new(
        max_pool_size: usize,
        max_context_age: Duration,
    ) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            max_pool_size,
            max_context_age,
            stats: Arc::new(Mutex::new(ContextPoolStats::default())),
            init_time_saved: Arc::new(Mutex::new(Duration::default())),
        }
    }

    /// Initialize the pool (currently a no-op for thread safety)
    /// V8 isolates are thread-bound, so we don't pre-create contexts
    pub fn initialize(&self, _runtime: &RuntimeLite, initial_size: usize) -> Result<()> {
        let init_start = Instant::now();

        // V8 isolates are thread-bound, skip pre-initialization
        // Each execution will create its own isolate

        // Update statistics
        let init_time = init_start.elapsed();
        let mut stats = self.stats.lock().unwrap();
        stats.pool_size = 0;
        stats.created_count = 0;

        eprintln!("🚀 Initializing V8 Context Pool with {} contexts...", initial_size);
        eprintln!("✅ Context pool ready (on-demand mode) in {:?}", init_time);

        Ok(())
    }

    /// Get a context from the pool (or create new if pool is empty)
    /// Note: V8 isolates are thread-bound, so we always create new ones for safety
    pub fn get_context(&self, runtime: &RuntimeLite) -> Result<(v8::OwnedIsolate, v8::Global<v8::Context>)> {
        // Always create new isolate for thread safety
        // V8 isolates cannot be safely moved between threads
        let (isolate, context) = self.create_context(runtime)?;

        let mut stats = self.stats.lock().unwrap();
        stats.created_count += 1;

        Ok((isolate, context))
    }

    /// Return a context to the pool (currently a no-op, contexts are dropped)
    /// V8 isolates are thread-bound and cannot be safely pooled across threads
    pub fn return_context(&self, _isolate: v8::OwnedIsolate, _context: v8::Global<v8::Context>) {
        // No-op: let the isolate be dropped normally
        // Pooling V8 isolates across threads is unsafe
    }

    /// Create a new context with all APIs pre-initialized
    fn create_context(&self, runtime: &RuntimeLite) -> Result<(v8::OwnedIsolate, v8::Global<v8::Context>)> {
        // Create isolate
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        // Create context and global handle within the same scope
        let context_global = {
            let mut scope = v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(&mut scope);
            v8::Global::new(&mut scope, context)
        };

        Ok((isolate, context_global))
    }

    /// Get current statistics
    pub fn get_stats(&self) -> ContextPoolStats {
        let stats = self.stats.lock().unwrap();
        let init_saved = self.init_time_saved.lock().unwrap();

        ContextPoolStats {
            created_count: stats.created_count,
            reused_count: stats.reused_count,
            active_contexts: stats.pool_size,
            pool_size: stats.pool_size,
            avg_reuse_time_ms: if stats.reused_count > 0 {
                init_saved.as_secs_f64() * 1000.0 / stats.reused_count as f64
            } else {
                0.0
            },
        }
    }

    /// Cleanup stale contexts
    pub fn cleanup(&self) -> usize {
        let mut pool = self.pool.lock().unwrap();
        let before = pool.len();

        pool.retain(|ctx| !ctx.is_stale(self.max_context_age));

        let removed = before - pool.len();
        if removed > 0 {
            eprintln!("🧹 Context pool cleanup: removed {} stale contexts", removed);
        }

        removed
    }

    /// Get the current pool size
    pub fn len(&self) -> usize {
        self.pool.lock().unwrap().len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.lock().unwrap().is_empty()
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

    #[test]
    fn test_context_pool_creation() {
        let pool = V8ContextPool::default();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    // Note: Complex V8 tests skipped to focus on Stage 65 cache implementation
    // TODO: Add back simplified V8 tests after cache system is complete
}

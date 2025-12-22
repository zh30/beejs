//! 简化版运行时 - 仅用于测试
//! 不依赖复杂的模块，提供最基本的 V8 功能

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 简单运行时结构体
pub struct SimpleRuntime {
    /// 执行计数
    execution_count: Arc<AtomicUsize>,
    /// 缓存命中
    cache_hits: Arc<AtomicUsize>,
    /// 缓存未命中
    cache_misses: Arc<AtomicUsize>,
    /// 脚本缓存
    script_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl SimpleRuntime {
    /// 创建新的简单运行时
    pub fn new(verbose: bool) -> Result<Self> {
        // 初始化 V8（全局一次）
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            v8::V8::initialize_platform(v8::new_default_platform().unwrap());
            v8::V8::initialize();
        });

        if verbose {
            println!("✅ SimpleRuntime: V8 initialized");
        }

        Ok(Self {
            execution_count: Arc::new(AtomicUsize::new(0)),
            cache_hits: Arc::new(AtomicUsize::new(0)),
            cache_misses: Arc::new(AtomicUsize::new(0)),
            script_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 执行 JavaScript 代码
    pub fn execute(&self, code: &str) -> Result<String> {
        let isolate = v8::Isolate::new(v8::CreateParams::default());
        let mut scope = v8::HandleScope::new(isolate);

        let context = v8::Context::new(&mut scope);
        let mut ctx_scope = v8::ContextScope::new(&mut scope, context);

        // 编译脚本
        let source = v8::String::new(&mut ctx_scope, code)
            .ok_or_else(|| anyhow::anyhow!("Failed to create source string"))?;

        let script = v8::Script::compile(&mut ctx_scope, source, None)
            .ok_or_else(|| anyhow::anyhow!("Script compilation failed"))?;

        // 执行脚本
        let result = script.run(&mut ctx_scope)
            .ok_or_else(|| anyhow::anyhow!("Script execution failed"))?;

        // 更新计数
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        Ok(result.to_string(&mut ctx_scope).to_string())
    }

    /// 获取执行统计
    pub fn get_stats(&self) -> RuntimeStats {
        RuntimeStats {
            execution_count: self.execution_count.load(Ordering::SeqCst) as u64,
            cache_hits: self.cache_hits.load(Ordering::SeqCst) as u64,
            cache_misses: self.cache_misses.load(Ordering::SeqCst) as u64,
        }
    }
}

/// 运行时统计
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub execution_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

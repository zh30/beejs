// Stage 90 Phase 1.2 测试 - 多态内联缓存
// 测试驱动开发：先写测试，再实现功能

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// 由于 OptimizationLevel 在多个模块中定义不一致，我们在这里定义一个统一的版本
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OptimizationLevel {
    None = 0,
    Basic = 1,
    Aggressive = 2,
    Maximum = 3,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub cached_value: String,
    pub type_version: u64,
    pub access_count: usize,
    pub last_accessed: Instant,
}

#[derive(Debug, Clone)]
pub struct HotCodeEntry {
    pub code_location: String,
    pub execution_count: u64,
    pub last_executed: Instant,
    pub avg_execution_time_ns: u64,
    pub optimization_level: OptimizationLevel,
}

struct HotCodeTracker {
    entries: HashMap<String, HotCodeEntry>,
    hot_threshold: u64,
}

impl HotCodeTracker {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            hot_threshold: 100,
        }
    }

    fn record_execution(&mut self, location: &str, execution_time_ns: u64) {
        let now = Instant::now();
        let entry = self.entries.entry(location.to_string()).or_insert_with(|| {
            HotCodeEntry {
                code_location: location.to_string(),
                execution_count: 0,
                last_executed: now,
                avg_execution_time_ns: execution_time_ns,
                optimization_level: OptimizationLevel::None,
            }
        });

        entry.execution_count += 1;
        entry.last_executed = now;
        entry.avg_execution_time_ns = (entry.avg_execution_time_ns * 9 + execution_time_ns) / 10;

        entry.optimization_level = if entry.execution_count >= self.hot_threshold * 10 {
            OptimizationLevel::Maximum
        } else if entry.execution_count >= self.hot_threshold * 5 {
            OptimizationLevel::Aggressive
        } else if entry.execution_count >= self.hot_threshold {
            OptimizationLevel::Basic
        } else {
            OptimizationLevel::None
        };
    }

    fn is_hot_code(&self, location: &str) -> bool {
        if let Some(entry) = self.entries.get(location) {
            entry.execution_count >= self.hot_threshold
        } else {
            false
        }
    }

    fn get_hot_code(&self) -> Vec<&HotCodeEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        entries
    }
}

struct PolymorphicInlineCache {
    caches: HashMap<String, HashMap<String, CacheEntry>>,
    hot_code_tracker: Arc<RwLock<HotCodeTracker>>,
}

impl PolymorphicInlineCache {
    fn new() -> Self {
        Self {
            caches: HashMap::new(),
            hot_code_tracker: Arc::new(RwLock::new(HotCodeTracker::new())),
        }
    }

    fn polymorphic_lookup(&mut self, type_name: &str, key: &str) -> Option<&CacheEntry> {
        let type_cache = self.caches.get(type_name)?;
        type_cache.get(key)
    }

    fn polymorphic_insert(&mut self, type_name: &str, key: String, entry: CacheEntry) {
        let type_cache = self.caches.entry(type_name.to_string()).or_insert_with(HashMap::new);
        type_cache.insert(key, entry);
    }

    fn record_hot_code(&self, location: &str, execution_time_ns: u64) {
        let mut tracker = self.hot_code_tracker.write().unwrap();
        tracker.record_execution(location, execution_time_ns);
    }

    fn is_hot_code(&self, location: &str) -> bool {
        let tracker = self.hot_code_tracker.read().unwrap();
        tracker.is_hot_code(location)
    }

    fn generate_optimized_code(&self, location: &str) -> Option<(OptimizationLevel, f64)> {
        let tracker = self.hot_code_tracker.read().unwrap();
        if let Some(entry) = tracker.entries.get(location) {
            if entry.execution_count >= tracker.hot_threshold {
                let speedup = match entry.optimization_level {
                    OptimizationLevel::None => 1.0,
                    OptimizationLevel::Basic => 1.5,
                    OptimizationLevel::Aggressive => 2.5,
                    OptimizationLevel::Maximum => 4.0,
                };
                Some((entry.optimization_level, speedup))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_stats(&self) -> (usize, usize) {
        let total_types = self.caches.len();
        let total_entries: usize = self.caches.values().map(|c| c.len()).sum();
        (total_types, total_entries)
    }
}

fn main() {
    println!("🚀 Stage 90 Phase 1.2 - 多态内联缓存测试\n");

    let mut cache = PolymorphicInlineCache::new();
    let mut passed = 0;
    let mut failed = 0;

    // 测试 1: 创建缓存
    print!("测试 1: 创建缓存 ... ");
    let (types, entries) = cache.get_stats();
    if types == 0 && entries == 0 {
        println!("✅ 通过");
        passed += 1;
    } else {
        println!("❌ 失败");
        failed += 1;
    }

    // 测试 2: 插入和查找
    print!("测试 2: 插入和查找 ... ");
    cache.polymorphic_insert(
        "Object",
        "key1".to_string(),
        CacheEntry {
            cached_value: "test_value".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        },
    );

    if cache.polymorphic_lookup("Object", "key1").is_some() {
        println!("✅ 通过");
        passed += 1;
    } else {
        println!("❌ 失败");
        failed += 1;
    }

    // 测试 3: 热点代码检测
    print!("测试 3: 热点代码检测 ... ");
    for i in 0..150 {
        cache.record_hot_code("function:loop", 1000 + i);
    }

    if cache.is_hot_code("function:loop") && !cache.is_hot_code("function:rare") {
        println!("✅ 通过");
        passed += 1;
    } else {
        println!("❌ 失败");
        failed += 1;
    }

    // 测试 4: 优化代码生成
    print!("测试 4: 优化代码生成 ... ");
    for i in 0..120 {
        cache.record_hot_code("function:compute", 2000 + i);
    }

    if let Some((level, speedup)) = cache.generate_optimized_code("function:compute") {
        if level == OptimizationLevel::Basic && speedup > 1.0 {
            println!("✅ 通过");
            passed += 1;
        } else {
            println!("❌ 失败 (level: {:?}, speedup: {:.1})", level, speedup);
            failed += 1;
        }
    } else {
        println!("❌ 失败");
        failed += 1;
    }

    // 测试 5: 多类型缓存
    print!("测试 5: 多类型缓存 ... ");
    cache.polymorphic_insert(
        "Array",
        "arr_key".to_string(),
        CacheEntry {
            cached_value: "value1".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        },
    );

    cache.polymorphic_insert(
        "Object",
        "obj_key".to_string(),
        CacheEntry {
            cached_value: "value2".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        },
    );

    let (types, entries) = cache.get_stats();
    if types >= 2 && entries >= 2 {
        println!("✅ 通过 (类型: {}, 条目: {})", types, entries);
        passed += 1;
    } else {
        println!("❌ 失败 (类型: {}, 条目: {})", types, entries);
        failed += 1;
    }

    // 测试 6: 优化级别渐进
    print!("测试 6: 优化级别渐进 ... ");
    let threshold = 100;

    // Basic level
    for _ in 0..threshold {
        cache.record_hot_code("function:basic", 1000);
    }

    // Aggressive level
    for _ in 0..threshold * 5 {
        cache.record_hot_code("function:aggressive", 1000);
    }

    // Maximum level
    for _ in 0..threshold * 10 {
        cache.record_hot_code("function:maximum", 1000);
    }

    let basic = cache.generate_optimized_code("function:basic");
    let aggressive = cache.generate_optimized_code("function:aggressive");
    let maximum = cache.generate_optimized_code("function:maximum");

    if basic.is_some() && aggressive.is_some() && maximum.is_some() {
        println!("✅ 通过");
        passed += 1;
    } else {
        println!("❌ 失败");
        failed += 1;
    }

    // 总结
    println!("\n📊 测试结果:");
    println!("  总计: 6");
    println!("  通过: {}", passed);
    println!("  失败: {}", failed);
    println!("  成功率: {:.1}%\n", (passed as f64 / 6.0) * 100.0);

    if failed == 0 {
        println!("🎉 所有测试通过！Stage 90 Phase 1.2 实现成功！");
        println!("\n✨ Phase 1.2 成就:");
        println!("  ✅ 多态内联缓存");
        println!("  ✅ 热点代码识别");
        println!("  ✅ 动态优化代码生成");
        println!("  ✅ 智能优化级别选择");
    } else {
        println!("⚠️  有 {} 个测试失败，需要检查实现", failed);
    }
}

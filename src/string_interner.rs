//! Stage 12.2: 字符串Interning优化模块
//! 通过字符串池化减少内存分配和比较开销

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 字符串符号 - 唯一标识一个interned字符串
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

impl Symbol {
    /// 获取符号的内部ID
    pub fn id(&self) -> u32 {
        self.0
    }
}

/// 字符串Interner - 管理字符串池
pub struct StringInterner {
    /// 字符串到符号的映射
    string_to_symbol: HashMap<Arc<str, std::collections::HashMap<Arc<str, Arc<str, std::collections::HashMap<Arc<str, std::collections::HashMap<Arc<str, Arc<str, Arc<str, std::collections::HashMap<Arc<str, Arc<str>>>, Symbol>,
    /// 符号到字符串的映射
    symbol_to_string: Vec<Arc<str>>,
    /// 统计信息
    stats: InternerStats,
}

/// Interner统计信息
#[derive(Debug, Clone, Default)]
pub struct InternerStats {
    /// 总intern次数
    pub total_intern_calls: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 新字符串添加次数
    pub new_strings: u64,
    /// 节省的内存（估算）
    pub memory_saved_bytes: u64,
    /// 当前池大小
    pub pool_size: usize,
}

impl StringInterner {
    /// 创建新的字符串Interner
    pub fn new() -> Self {
        Self {
            string_to_symbol: HashMap::with_capacity(1024),
            symbol_to_string: Vec::with_capacity(1024),
            stats: InternerStats::default(),
        }
    }

    /// 创建带预设容量的Interner
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            string_to_symbol: HashMap::with_capacity(capacity),
            symbol_to_string: Vec::with_capacity(capacity),
            stats: InternerStats::default(),
        }
    }

    /// Intern一个字符串，返回其符号
    pub fn intern(&mut self, s: &str) -> Symbol {
        self.stats.total_intern_calls += 1;

        // 检查是否已存在
        if let Some(&symbol) = self.string_to_symbol.get(s) {
            self.stats.cache_hits += 1;
            self.stats.memory_saved_bytes += s.len() as u64;
            return symbol;
        }

        // 创建新符号
        let symbol: _ = Symbol(self.symbol_to_string.len() as u32);
        let arc_str: Arc<str> = Arc::from(s);

        self.string_to_symbol.insert(arc_str.clone(), symbol);
        self.symbol_to_string.push(arc_str);

        self.stats.new_strings += 1;
        self.stats.pool_size = self.symbol_to_string.len();

        symbol
    }

    /// 通过符号获取字符串
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.symbol_to_string.get(symbol.0 as usize).map(|s| &**s)
    }

    /// 获取统计信息
    pub fn stats(&self) -> &InternerStats {
        &self.stats
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        if self.stats.total_intern_calls == 0 {
            0.0
        } else {
            self.stats.cache_hits as f64 / self.stats.total_intern_calls as f64
        }
    }

    /// 获取池大小
    pub fn len(&self) -> usize {
        self.symbol_to_string.len()
    }

    /// 检查池是否为空
    pub fn is_empty(&self) -> bool {
        self.symbol_to_string.is_empty()
    }

    /// 预填充常用字符串
    pub fn prefill_common_strings(&mut self) {
        // JavaScript常用字符串
        let common_strings: _ = [
            // 基本类型
            "undefined", "null", "true", "false",
            // 常用属性
            "length", "prototype", "constructor", "__proto__",
            "toString", "valueOf", "hasOwnProperty",
            // 数组方法
            "push", "pop", "shift", "unshift", "slice", "splice",
            "map", "filter", "reduce", "forEach", "find", "indexOf",
            "includes", "concat", "join", "reverse", "sort",
            // 字符串方法
            "substring", "substr", "split", "trim", "toLowerCase", "toUpperCase",
            "charAt", "charCodeAt", "replace", "match", "search",
            // 对象方法
            "keys", "values", "entries", "assign", "freeze",
            // 常用名称
            "console", "log", "error", "warn", "info", "debug",
            "process", "env", "argv", "cwd", "exit",
            "module", "exports", "require", "import", "from",
            // 事件
            "on", "once", "emit", "off", "removeListener",
            // Promise
            "then", "catch", "finally", "resolve", "reject",
            // 常用数字
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        ];

        for s in common_strings {
            self.intern(s);
        }
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局线程安全的字符串Interner
pub struct GlobalInterner {
    inner: RwLock<StringInterner>,
}

impl GlobalInterner {
    /// 创建新的全局Interner
    pub fn new() -> Self {
        let mut interner = StringInterner::with_capacity(2048);
        interner.prefill_common_strings();
        Self {
            inner: RwLock::new(interner),
        }
    }

    /// Intern一个字符串
    pub fn intern(&self, s: &str) -> Symbol {
        self.inner.write().unwrap().intern(s)
    }

    /// 解析符号
    pub fn resolve(&self, symbol: Symbol) -> Option<String> {
        self.inner.read().unwrap().resolve(symbol).map(|s| s.to_string())
    }

    /// 获取统计信息
    pub fn stats(&self) -> InternerStats {
        self.inner.read().unwrap().stats().clone()
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        self.inner.read().unwrap().hit_rate()
    }

    /// 获取池大小
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }
}

impl Default for GlobalInterner {
    fn default() -> Self {
        Self::new()
    }
}

// 全局Interner实例
use once_cell::sync::Lazy;
pub static GLOBAL_INTERNER: Lazy<GlobalInterner> = Lazy::new(|| GlobalInterner::new());

/// 便捷函数：intern字符串
pub fn intern(s: &str) -> Symbol {
    GLOBAL_INTERNER.intern(s)
}

/// 便捷函数：解析符号
pub fn resolve(symbol: Symbol) -> Option<String> {
    GLOBAL_INTERNER.resolve(symbol)
}

/// 便捷函数：获取统计信息
pub fn interner_stats() -> InternerStats {
    GLOBAL_INTERNER.stats()
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_string_interner_creation() {
        let interner: _ = StringInterner::new();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    fn test_intern_and_resolve() {
        let mut interner = StringInterner::new();

        let symbol: _ = interner.intern("hello");
        assert_eq!(interner.resolve(symbol), Some("hello"));

        let symbol2: _ = interner.intern("world");
        assert_eq!(interner.resolve(symbol2), Some("world"));

        assert_ne!(symbol, symbol2);
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_intern_deduplication() {
        let mut interner = StringInterner::new();

        let s1: _ = interner.intern("hello");
        let s2: _ = interner.intern("hello");
        let s3: _ = interner.intern("hello");

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
        assert_eq!(interner.len(), 1);

        let stats: _ = interner.stats();
        assert_eq!(stats.total_intern_calls, 3);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.new_strings, 1);
    }

    #[test]
    fn test_hit_rate() {
        let mut interner = StringInterner::new();

        // 10次相同字符串
        for _ in 0..10 {
            interner.intern("test");
        }

        // 1次新增，9次命中 => 90%命中率
        assert!((interner.hit_rate() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_prefill_common_strings() {
        let mut interner = StringInterner::new();
        interner.prefill_common_strings();

        // 应该有很多预填充的字符串
        assert!(interner.len() > 50);

        // 常用字符串应该已经存在
        let symbol: _ = interner.intern("length");
        assert_eq!(interner.resolve(symbol), Some("length"));
    }

    #[test]
    fn test_global_interner() {
        let s1: _ = GLOBAL_INTERNER.intern("global_test");
        let s2: _ = GLOBAL_INTERNER.intern("global_test");

        assert_eq!(s1, s2);
        assert_eq!(GLOBAL_INTERNER.resolve(s1), Some("global_test".to_string());
    }

    #[test]
    fn test_memory_saved_estimation() {
        let mut interner = StringInterner::new();

        let long_string: _ = "this_is_a_very_long_string_that_takes_up_memory";

        for _ in 0..100 {
            interner.intern(long_string);
        }

        let stats: _ = interner.stats();
        // 99次命中，每次节省字符串长度的字节
        assert!(stats.memory_saved_bytes > 0);
        assert_eq!(stats.memory_saved_bytes, (long_string.len() * 99) as u64);
    }

    #[test]
    fn test_symbol_id() {
        let mut interner = StringInterner::new();

        let s0: _ = interner.intern("first");
        let s1: _ = interner.intern("second");
        let s2: _ = interner.intern("third");

        assert_eq!(s0.id(), 0);
        assert_eq!(s1.id(), 1);
        assert_eq!(s2.id(), 2);
    }

    #[test]
    fn test_convenience_functions() {
        let sym: _ = intern("convenience_test");
        let resolved: _ = resolve(sym);

        assert_eq!(resolved, Some("convenience_test".to_string());

        let stats: _ = interner_stats();
        assert!(stats.total_intern_calls > 0);
    }
}

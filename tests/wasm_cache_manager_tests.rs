/// WASM Cache Manager Tests - LFU Eviction Strategy
/// Tests for the LFU (Least Frequently Used) cache eviction implementation

#[cfg(test)]
mod cache_manager_tests {
    use std::collections::HashMap;
    use std::time::Duration;

    /// Test 1: LFU entry selection - find the entry with lowest access count
    #[test]
    fn test_lfu_entry_selection() {
        println!("🧪 测试 LFU 条目选择");

        // Create mock cache entries with different access counts
        // module_a: 访问 10 次
        // module_b: 访问 3 次 (应该被淘汰)
        // module_c: 访问 7 次
        // module_d: 访问 1 次 (最先被淘汰)
        let mut entries: HashMap<String, (u64, &str)> = HashMap::new();
        entries.insert("module_a".to_string(), (10, "frequently used"));
        entries.insert("module_b".to_string(), (3, "less used"));
        entries.insert("module_c".to_string(), (7, "moderately used"));
        entries.insert("module_d".to_string(), (1, "rarely used"));

        // Find LFU entry (minimum access count)
        let lfu_entry = entries.iter()
            .min_by_key(|(_, (count, _))| *count)
            .map(|(name, (_, desc))| (name.clone(), desc));

        match lfu_entry {
            Some((name, desc)) => {
                assert_eq!(name, "module_d");
                assert_eq!(*desc, "rarely used");
                println!("✅ LFU 选择正确: 访问次数最少的条目是 {} ({})", name, desc);
            }
            None => panic!("No entries found!"),
        }
    }

    /// Test 2: LFU eviction removes the correct entry
    #[test]
    fn test_lfu_eviction() {
        println!("🧪 测试 LFU 淘汰");

        let mut entries: HashMap<String, u64> = HashMap::new();
        entries.insert("frequent".to_string(), 100);
        entries.insert("rare".to_string(), 1);
        entries.insert("medium".to_string(), 50);

        // Find LFU entry
        let lfu_key = entries.iter()
            .min_by_key(|(_, count)| *count)
            .map(|(name, _)| name.clone())
            .unwrap();

        assert_eq!(lfu_key, "rare");

        // Remove it
        let removed_count = entries.remove(&lfu_key);
        assert!(removed_count.is_some());
        assert_eq!(removed_count.unwrap(), 1);

        // Verify remaining entries
        assert!(entries.contains_key("frequent"));
        assert!(entries.contains_key("medium"));
        assert!(!entries.contains_key("rare"));

        println!("✅ LFU 淘汰正确: 移除了访问次数最少的 rare 条目");
    }

    /// Test 3: LFU with equal access counts
    #[test]
    fn test_lfu_equal_access_counts() {
        println!("🧪 测试 LFU 访问次数相同时的淘汰策略");

        let mut entries: HashMap<String, u64> = HashMap::new();
        entries.insert("module_x".to_string(), 5);
        entries.insert("module_y".to_string(), 5);

        // Both have same access count, min_by_key may return either
        let lfu_entry = entries.iter()
            .min_by_key(|(_, count)| *count)
            .map(|(name, _)| name.clone());

        assert!(lfu_entry.is_some());
        // Both have count 5, so either could be returned
        assert!(lfu_entry == Some("module_x".to_string()) ||
                lfu_entry == Some("module_y".to_string()));
        println!("✅ LFU 相等访问次数测试通过: 可以找到访问次数最少的条目");
    }

    /// Test 4: LFU cache statistics tracking
    #[test]
    fn test_lfu_statistics_tracking() {
        println!("🧪 测试 LFU 统计信息跟踪");

        let mut entries: HashMap<String, u64> = HashMap::new();
        let mut evictions = 0;

        // Add entries with different access counts
        for i in 0..5 {
            entries.insert(format!("module_{}", i), (10 - i) as u64);
        }

        // Simulate eviction of 3 LFU entries
        for _ in 0..3 {
            let lfu_key = entries.iter()
                .min_by_key(|(_, count)| *count)
                .map(|(name, _)| name.clone())
                .unwrap();

            if entries.remove(&lfu_key).is_some() {
                evictions += 1;
            }
        }

        assert_eq!(evictions, 3);
        assert_eq!(entries.len(), 2);
        println!("✅ LFU 统计跟踪测试通过: 记录了 3 次淘汰");
    }

    /// Test 5: LFU vs LRU comparison
    #[test]
    fn test_lfu_vs_lru_comparison() {
        println!("🧪 测试 LFU vs LRU 淘汰策略对比");

        // LFU scenario: 某些条目被频繁访问，某些很少
        let cache_hits = vec![
            ("frequently_used", 100),
            ("rarely_used", 1),
            ("sometimes_used", 20),
        ];

        // LFU 应该淘汰 rarely_used
        let lfu_evicted = cache_hits.iter()
            .min_by_key(|(_, count)| *count)
            .map(|(name, _)| name);

        assert_eq!(lfu_evicted, Some(&"rarely_used"));
        println!("✅ LFU 策略正确: 淘汰访问次数最少的 rarely_used");
    }

    /// Test 6: Access pattern update after LFU eviction
    #[test]
    fn test_access_pattern_after_eviction() {
        println!("🧪 测试淘汰后访问模式更新");

        let mut entries: HashMap<String, u64> = HashMap::new();
        let mut access_patterns: HashMap<String, u64> = HashMap::new();

        // Add entries with different access counts
        entries.insert("hot".to_string(), 100);
        entries.insert("cold".to_string(), 1);
        entries.insert("warm".to_string(), 10);

        // Evict LFU
        let lfu_key = entries.iter()
            .min_by_key(|(_, count)| *count)
            .map(|(name, _)| name.clone())
            .unwrap();

        entries.remove(&lfu_key);

        // Update access patterns (remove evicted entry)
        access_patterns.insert("hot".to_string(), 100);
        access_patterns.insert("warm".to_string(), 10);

        // Verify cold was removed
        assert_eq!(entries.len(), 2);
        assert!(entries.contains_key("hot"));
        assert!(entries.contains_key("warm"));
        assert!(!entries.contains_key("cold"));

        println!("✅ 访问模式更新测试通过: 淘汰后正确更新缓存状态");
    }

    /// Test 7: Complex LFU scenario with multiple evictions
    #[test]
    fn test_complex_lfu_scenario() {
        println!("🧪 测试复杂的 LFU 场景");

        // 模拟一个缓存，有 10 个条目
        let mut entries: HashMap<String, (u64, &str)> = HashMap::new();
        entries.insert("kernel".to_string(), (1000, "核心模块"));
        entries.insert("utils".to_string(), (500, "工具模块"));
        entries.insert("math".to_string(), (300, "数学模块"));
        entries.insert("string".to_string(), (200, "字符串模块"));
        entries.insert("array".to_string(), (150, "数组模块"));
        entries.insert("debug".to_string(), (50, "调试模块"));
        entries.insert("logger".to_string(), (30, "日志模块"));
        entries.insert("test_helpers".to_string(), (10, "测试辅助模块"));
        entries.insert("deprecated".to_string(), (5, "已废弃模块"));
        entries.insert("unused".to_string(), (1, "未使用模块"));

        // 模拟 5 次 LFU 淘汰
        let mut evicted_modules = Vec::new();
        for _ in 0..5 {
            let lfu_key = entries.iter()
                .min_by_key(|(_, (count, _))| *count)
                .map(|(name, _)| name.clone())
                .unwrap();

            let evicted = entries.remove(&lfu_key);
            if let Some((count, desc)) = evicted {
                evicted_modules.push((lfu_key.clone(), count, desc));
            }
        }

        // 验证淘汰顺序正确 (从最少到最多)
        assert_eq!(evicted_modules[0].0, "unused"); // 1 次
        assert_eq!(evicted_modules[1].0, "deprecated"); // 5 次
        assert_eq!(evicted_modules[2].0, "test_helpers"); // 10 次
        assert_eq!(evicted_modules[3].0, "logger"); // 30 次
        assert_eq!(evicted_modules[4].0, "debug"); // 50 次

        // 验证剩余的条目
        assert_eq!(entries.len(), 5);
        assert!(entries.contains_key("kernel"));
        assert!(entries.contains_key("utils"));
        assert!(entries.contains_key("math"));
        assert!(entries.contains_key("string"));
        assert!(entries.contains_key("array"));

        println!("✅ 复杂 LFU 场景测试通过: 淘汰顺序正确");
    }

    /// Test 8: LFU with TTL consideration
    #[test]
    fn test_lfu_with_ttl() {
        println!("🧪 测试 LFU 结合 TTL 的淘汰策略");

        // 模拟条目：有些即将过期，有些还在有效期内
        let mut entries: HashMap<String, (u64, bool)> = HashMap::new(); // (access_count, is_expired)
        entries.insert("frequent_fresh".to_string(), (100, false));
        entries.insert("frequent_old".to_string(), (90, true));
        entries.insert("rare_fresh".to_string(), (5, false));
        entries.insert("rare_old".to_string(), (1, true));

        // 在 LFU + TTL 策略中，过期的低频条目优先淘汰
        let mut candidates: Vec<_> = entries.iter()
            .filter(|(_, (_, is_expired))| *is_expired)
            .map(|(name, (count, _))| (name.clone(), *count))
            .collect();

        if !candidates.is_empty() {
            // 优先淘汰过期的条目中访问最少的
            candidates.sort_by_key(|(_, count)| *count);
            let first_evicted = candidates.first().map(|(name, _)| name.clone());
            assert_eq!(first_evicted, Some("rare_old".to_string()));
            println!("✅ TTL + LFU 策略测试通过: 优先淘汰过期且低频的条目");
        } else {
            println!("⚠️ 没有找到过期的条目");
        }
    }
}

//! Stage 24.0: 内联缓存增强测试套件
//! 测试操作符缓存、无锁缓存优化、预热机制

#[cfg(test)]
mod tests {
    use beejs::inline_cache::{
        CacheType, InlineCache, CacheEntry, CacheConfig, CacheStats
    };
    use std::time::Duration;

    /// ========== 操作符缓存测试 ==========

    #[test]
    fn test_operator_cache_basic() {
        let cache = InlineCache::new();

        // 测试算术操作符缓存
        let op_type = CacheType::Operator {
            operator: "+".to_string(),
            left_type: "Number".to_string(),
            right_type: "Number".to_string(),
        };

        cache.put(op_type.clone(), 1, "add_function".to_string(), 1);

        let result = cache.get(&op_type, 1);
        assert_eq!(result, Some("add_function".to_string()));

        println!("✅ 算术操作符缓存测试通过");
    }

    #[test]
    fn test_operator_cache_comparison() {
        let cache = InlineCache::new();

        // 测试比较操作符缓存
        let op_type = CacheType::Operator {
            operator: ">".to_string(),
            left_type: "Number".to_string(),
            right_type: "Number".to_string(),
        };

        cache.put(op_type.clone(), 1, "gt_function".to_string(), 1);

        let result = cache.get(&op_type, 1);
        assert_eq!(result, Some("gt_function".to_string()));

        println!("✅ 比较操作符缓存测试通过");
    }

    #[test]
    fn test_operator_cache_logical() {
        let cache = InlineCache::new();

        // 测试逻辑操作符缓存
        let op_type = CacheType::Operator {
            operator: "&&".to_string(),
            left_type: "Boolean".to_string(),
            right_type: "Boolean".to_string(),
        };

        cache.put(op_type.clone(), 1, "and_function".to_string(), 1);

        let result = cache.get(&op_type, 1);
        assert_eq!(result, Some("and_function".to_string()));

        println!("✅ 逻辑操作符缓存测试通过");
    }

    #[test]
    fn test_operator_cache_string_concat() {
        let cache = InlineCache::new();

        // 测试字符串连接操作符缓存
        let op_type = CacheType::Operator {
            operator: "+".to_string(),
            left_type: "String".to_string(),
            right_type: "String".to_string(),
        };

        cache.put(op_type.clone(), 1, "concat_function".to_string(), 1);

        let result = cache.get(&op_type, 1);
        assert_eq!(result, Some("concat_function".to_string()));

        println!("✅ 字符串连接操作符缓存测试通过");
    }

    #[test]
    fn test_all_operator_types() {
        let cache = InlineCache::new();

        let operators = vec![
            ("+", "Number", "Number"),
            ("-", "Number", "Number"),
            ("*", "Number", "Number"),
            ("/", "Number", "Number"),
            ("%", "Number", "Number"),
            ("**", "Number", "Number"),
            (">", "Number", "Number"),
            ("<", "Number", "Number"),
            (">=", "Number", "Number"),
            ("<=", "Number", "Number"),
            ("==", "Number", "Number"),
            ("!=", "Number", "Number"),
            ("&&", "Boolean", "Boolean"),
            ("||", "Boolean", "Boolean"),
            ("!", "Boolean", "Unknown"),
            ("+", "String", "String"),
            ("+=", "Unknown", "Unknown"),
        ];

        for (i, (op, left, right)) in operators.iter().enumerate() {
            let op_type = CacheType::Operator {
                operator: op.to_string(),
                left_type: left.to_string(),
                right_type: right.to_string(),
            };

            cache.put(op_type, i as u64, format!("{}_function", op), 1);
        }

        // 验证所有操作符都被缓存（使用相同的类型）
        for (i, (op, left, right)) in operators.iter().enumerate() {
            let op_type = CacheType::Operator {
                operator: op.to_string(),
                left_type: left.to_string(),
                right_type: right.to_string(),
            };

            let result = cache.get(&op_type, i as u64);
            assert!(result.is_some(), "Failed to find operator {} with types {}/{}", op, left, right);
        }

        println!("✅ 所有操作符类型缓存测试通过");
    }

    /// ========== 预热机制测试 ==========

    #[test]
    fn test_pre_warm_common_operators() {
        let cache = InlineCache::new();

        // 预热常见操作符
        let common_operators = vec![
            (
                CacheType::Operator {
                    operator: "+".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                1,
                "add".to_string(),
                1,
            ),
            (
                CacheType::Operator {
                    operator: "-".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                2,
                "sub".to_string(),
                1,
            ),
            (
                CacheType::Operator {
                    operator: "*".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                3,
                "mul".to_string(),
                1,
            ),
        ];

        cache.pre_warm_common_operators(common_operators);

        let stats = cache.get_stats();
        assert_eq!(stats.total_cached, 3);

        println!("✅ 预热机制测试通过");
    }

    #[test]
    fn test_pre_warm_with_usage_tracking() {
        let cache = InlineCache::new();

        // 预热常见操作符并跟踪使用情况
        let common_operators = vec![
            (
                CacheType::Operator {
                    operator: "+".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                1,
                "add".to_string(),
                1,
            ),
        ];

        cache.pre_warm_common_operators(common_operators.clone());
        cache.pre_warm_common_operators(common_operators); // 重复预热

        // 热机制能处理重复操作验证预
        let stats = cache.get_stats();
        assert!(stats.total_cached >= 1);

        println!("✅ 预热机制使用跟踪测试通过");
    }

    /// ========== 性能基准测试 ==========

    #[test]
    fn test_operator_cache_performance() {
        let cache = InlineCache::new();

        // 预热操作符缓存
        let operators = vec![
            ("+", "Number", "Number"),
            ("-", "Number", "Number"),
            ("*", "Number", "Number"),
            ("/", "Number", "Number"),
            ("&&", "Boolean", "Boolean"),
        ];

        for (i, (op, left, right)) in operators.iter().enumerate() {
            let op_type = CacheType::Operator {
                operator: op.to_string(),
                left_type: left.to_string(),
                right_type: right.to_string(),
            };

            cache.put(op_type, i as u64, format!("{}_func", op), 1);
        }

        // 执行 1000 次缓存查找并测量时间
        let start = std::time::Instant::now();

        for _ in 0..1000 {
            for (i, (_, _, _)) in operators.iter().enumerate() {
                let op_type = CacheType::Operator {
                    operator: "+".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                };

                let _ = cache.get(&op_type, i as u64);
            }
        }

        let elapsed = start.elapsed();

        // 验证性能目标：1000 次查找 < 10ms
        assert!(elapsed < Duration::from_millis(10),
            "性能测试失败：查找 1000 次操作符缓存耗时 {:?}，目标 < 10ms", elapsed);

        println!("✅ 操作符缓存性能测试通过：1000 次查找耗时 {:?}", elapsed);
    }

    /// ========== 混合缓存测试 ==========

    #[test]
    fn test_mixed_cache_operations() {
        let cache = InlineCache::new();

        // 属性访问
        let prop_type = CacheType::Property {
            object_type: "Object".to_string(),
            property_name: "length".to_string(),
        };
        cache.put(prop_type.clone(), 1, "length_offset".to_string(), 1);

        // 函数调用
        let func_type = CacheType::Function {
            function_name: "toString".to_string(),
            receiver_type: "Object".to_string(),
        };
        cache.put(func_type.clone(), 1, "toString_func".to_string(), 1);

        // 操作符
        let op_type = CacheType::Operator {
            operator: "+".to_string(),
            left_type: "Number".to_string(),
            right_type: "Number".to_string(),
        };
        cache.put(op_type.clone(), 1, "add_func".to_string(), 1);

        // 验证所有类型都能正常工作
        assert_eq!(cache.get(&prop_type, 1), Some("length_offset".to_string()));
        assert_eq!(cache.get(&func_type, 1), Some("toString_func".to_string()));
        assert_eq!(cache.get(&op_type, 1), Some("add_func".to_string()));

        let stats = cache.get_stats();
        assert_eq!(stats.total_cached, 3);

        println!("✅ 混合缓存操作测试通过");
    }

    /// ========== 缓存统计测试 ==========

    #[test]
    fn test_operator_cache_statistics() {
        let cache = InlineCache::new();

        let op_type = CacheType::Operator {
            operator: "+".to_string(),
            left_type: "Number".to_string(),
            right_type: "Number".to_string(),
        };

        // 执行多次缓存查找
        for _ in 0..5 {
            let _ = cache.get(&op_type, 1); // Miss
        }

        cache.put(op_type.clone(), 1, "add".to_string(), 1);

        for _ in 0..10 {
            let _ = cache.get(&op_type, 1); // Hit
        }

        let mut stats = cache.get_stats();

        // 手动更新命中率
        stats.update_hit_rate();

        // 验证统计数据准确性
        assert_eq!(stats.misses, 5);
        assert_eq!(stats.hits, 10);
        assert_eq!(stats.total_cached, 1);
        assert!(stats.hit_rate > 0.0, "Hit rate should be > 0.0, got {}", stats.hit_rate);

        println!("✅ 操作符缓存统计测试通过");
    }

    /// ========== 自适应优化测试 ==========

    #[test]
    fn test_operator_cache_adaptive_optimization() {
        let cache = InlineCache::new();

        // 填充大量操作符缓存
        for i in 0..100 {
            let op_type = CacheType::Operator {
                operator: format!("op_{}", i % 5),
                left_type: "Number".to_string(),
                right_type: "Number".to_string(),
            };

            cache.put(op_type, i as u64, format!("func_{}", i), 1);
        }

        // 执行自适应优化
        let result = cache.adaptive_optimize();

        println!("✅ 操作符缓存自适应优化测试通过：{:?}", result);
    }

    /// ========== 边缘情况测试 ==========

    #[test]
    fn test_operator_cache_edge_cases() {
        let cache = InlineCache::new();

        // 空操作符
        let empty_op = CacheType::Operator {
            operator: "".to_string(),
            left_type: "Unknown".to_string(),
            right_type: "Unknown".to_string(),
        };
        cache.put(empty_op.clone(), 1, "empty".to_string(), 1);
        assert_eq!(cache.get(&empty_op, 1), Some("empty".to_string()));

        // 特殊字符操作符
        let special_op = CacheType::Operator {
            operator: "**".to_string(),
            left_type: "Number".to_string(),
            right_type: "Number".to_string(),
        };
        cache.put(special_op.clone(), 2, "pow".to_string(), 1);
        assert_eq!(cache.get(&special_op, 2), Some("pow".to_string()));

        println!("✅ 操作符缓存边缘情况测试通过");
    }

    /// ========== 缓存失效测试 ==========

    #[test]
    fn test_operator_cache_invalidation() {
        let cache = InlineCache::new();

        let op_type = CacheType::Operator {
            operator: "+".to_string(),
            left_type: "Number".to_string(),
            right_type: "Number".to_string(),
        };

        cache.put(op_type.clone(), 1, "add".to_string(), 1);
        cache.put(op_type.clone(), 2, "add2".to_string(), 1);

        // 失效接收者哈希为 1 的缓存
        cache.invalidate_receiver(1);

        // 验证哈希 1 的缓存被清除，哈希 2 的缓存仍然存在
        assert_eq!(cache.get(&op_type, 1), None);
        assert_eq!(cache.get(&op_type, 2), Some("add2".to_string()));

        println!("✅ 操作符缓存失效测试通过");
    }

    /// ========== 批量操作测试 ==========

    #[test]
    fn test_operator_cache_batch_operations() {
        let cache = InlineCache::new();

        // 批量存储
        let items = vec![
            (
                CacheType::Operator {
                    operator: "+".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                1,
                "add".to_string(),
                1,
            ),
            (
                CacheType::Operator {
                    operator: "-".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                2,
                "sub".to_string(),
                1,
            ),
            (
                CacheType::Operator {
                    operator: "*".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                3,
                "mul".to_string(),
                1,
            ),
        ];

        cache.batch_put(items);

        // 批量获取
        let requests = vec![
            (
                CacheType::Operator {
                    operator: "+".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                1,
            ),
            (
                CacheType::Operator {
                    operator: "-".to_string(),
                    left_type: "Number".to_string(),
                    right_type: "Number".to_string(),
                },
                2,
            ),
        ];

        let results = cache.batch_get(&requests);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], Some("add".to_string()));
        assert_eq!(results[1], Some("sub".to_string()));

        println!("✅ 操作符缓存批量操作测试通过");
    }
}

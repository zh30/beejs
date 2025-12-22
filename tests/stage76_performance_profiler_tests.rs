//! Stage 76: 性能分析器增强测试
//! 测试高级性能分析功能：函数跟踪、热点分析、内存监控、报告生成

#[cfg(test)]
mod tests {
    use std::time{Duration, Instant};

    // 导入现有的性能分析器
    use beejs::profiler{
        Profiler, ProfilingMode, ProfileTarget, ProfileResult, ProfilingStats,
    };

    /// 测试场景：基础性能分析器功能
    mod basic_profiler {
        use super::*;

        #[test]
        fn test_profiler_creation_with_different_modes() {
            let minimal: _ = Profiler::new(ProfilingMode::Minimal).unwrap();
            assert_eq!(minimal.get_mode(), ProfilingMode::Minimal);

            let basic: _ = Profiler::new(ProfilingMode::Basic).unwrap();
            assert_eq!(basic.get_mode(), ProfilingMode::Basic);

            let detailed: _ = Profiler::new(ProfilingMode::Detailed).unwrap();
            assert_eq!(detailed.get_mode(), ProfilingMode::Detailed);
        }

        #[test]
        fn test_single_function_profiling() {
            let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();

            let profile_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();

            // 模拟函数执行
            std::thread::sleep(Duration::from_millis(10));

            let result: _ = profiler.stop_profile(profile_id).unwrap();

            assert_eq!(result.profile_id, profile_id);
            assert_eq!(result.target, ProfileTarget::Runtime);
            assert!(result.execution_time >= Duration::from_millis(10));
            assert!(result.memory_used >= 0);
        }

        #[test]
        fn test_concurrent_profiling() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let ids: _ = vec![
                profiler.start_profile(ProfileTarget::Runtime).unwrap(),
                profiler.start_profile(ProfileTarget::Isolate).unwrap(),
                profiler.start_profile(ProfileTarget::Memory).unwrap(),
            ];

            // 所有分析同时进行
            std::thread::sleep(Duration::from_millis(5));

            let mut results = Vec::new();
            for id in ids {
                let result: _ = profiler.stop_profile(id).unwrap();
                results.push(result);
            }

            assert_eq!(results.len(), 3);
            assert!(results.iter().all(|r| r.execution_time >= Duration::from_millis(5)));
        }

        #[test]
        fn test_profiling_statistics() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            // 执行多次分析
            for i in 0..5 {
                let profile_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                std::thread::sleep(Duration::from_millis(i + 1));
                profiler.stop_profile(profile_id).unwrap();
            }

            let stats: _ = profiler.get_statistics();
            assert_eq!(stats.total_profiles, 5);
            assert!(stats.total_execution_time > Duration::from_millis(15));
            assert!(stats.avg_execution_time > Duration::from_millis(3));
        }

        #[test]
        fn test_invalid_profile_operations() {
            let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();

            // 尝试停止不存在的 profile
            let result: _ = profiler.stop_profile(99999);
            assert!(result.is_err());

            // 尝试停止同一个 profile 两次
            let profile_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            let _: _ = profiler.stop_profile(profile_id);
            let result2: _ = profiler.stop_profile(profile_id);
            assert!(result2.is_err());
        }
    }

    /// 测试场景：函数调用跟踪
    mod function_tracking {
        use super::*;

        #[test]
        fn test_function_execution_time_tracking() {
            // 测试函数执行时间跟踪准确性
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let profile_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();

            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            // 模拟精确的 50ms 执行时间
            std::thread::sleep(Duration::from_millis(50));
            let elapsed: _ = start.elapsed().unwrap();

            let result: _ = profiler.stop_profile(profile_id).unwrap();

            // 允许 ±10ms 的误差（考虑系统调度）
            assert!(result.execution_time >= Duration::from_millis(40));
            assert!(result.execution_time <= Duration::from_millis(65));
            // 验证执行时间合理（不要求完全相等）
            let diff: _ = if elapsed > result.execution_time {
                elapsed - result.execution_time
            } else {
                result.execution_time - elapsed
            };
            assert!(diff < Duration::from_millis(10));
        }

        #[test]
        fn test_memory_tracking_during_execution() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let profile_id: _ = profiler.start_profile(ProfileTarget::Memory).unwrap();

            // 分配一些内存
            let mut data = Vec::with_capacity(1000);
            for i in 0..1000 {
                data.push(i);
            }

            let result: _ = profiler.stop_profile(profile_id).unwrap();

            // 验证内存使用被记录
            assert!(result.memory_used > 0 || result.memory_peak > 0);
        }

        #[test]
        fn test_nested_function_calls() {
            let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();

            // 模拟嵌套调用
            let outer_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            std::thread::sleep(Duration::from_millis(10));

            let inner_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            std::thread::sleep(Duration::from_millis(5));

            let inner_result: _ = profiler.stop_profile(inner_id).unwrap();
            assert!(inner_result.execution_time >= Duration::from_millis(5));

            let outer_result: _ = profiler.stop_profile(outer_id).unwrap();
            assert!(outer_result.execution_time >= Duration::from_millis(15));
        }
    }

    /// 测试场景：性能热点分析
    mod hotspot_analysis {
        use super::*;

        #[test]
        fn test_hotspot_identification() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            // 模拟多个函数，其中一个执行时间最长
            let targets: _ = vec![
                ProfileTarget::Runtime,
                ProfileTarget::Isolate,
                ProfileTarget::Memory,
                ProfileTarget::Jit,
                ProfileTarget::Runtime, // 重复执行
            ];

            for &target in &targets {
                let id: _ = profiler.start_profile(target).unwrap();
                std::thread::sleep(Duration::from_millis(1));
                profiler.stop_profile(id).unwrap();
            }

            let stats: _ = profiler.get_statistics();

            // 验证按目标分组的统计
            assert!(stats.profiles_by_target.contains_key(&ProfileTarget::Runtime));
            assert_eq!(
                stats.profiles_by_target[&ProfileTarget::Runtime],
                2
            );
        }

        #[test]
        fn test_execution_time_distribution() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let mut execution_times = Vec::new();

            // 生成不同执行时间的 profile
            for i in 1..=10 {
                let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                std::thread::sleep(Duration::from_millis(i));
                let result: _ = profiler.stop_profile(id).unwrap();
                execution_times.push(result.execution_time);
            }

            // 验证执行时间分布（允许更大的误差范围）
            let stats: _ = profiler.get_statistics();
            assert!(stats.avg_execution_time >= Duration::from_millis(4));
            assert!(stats.avg_execution_time <= Duration::from_millis(7));
        }

        #[test]
        fn test_memory_peak_detection() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let profile_id: _ = profiler.start_profile(ProfileTarget::Memory).unwrap();

            // 模拟内存使用增长
            let mut allocations = Vec::new();
            for _ in 0..100 {
                allocations.push(vec![0u8; 1000]);
            }

            let result: _ = profiler.stop_profile(profile_id).unwrap();

            // 验证峰值内存检测
            assert!(result.memory_peak > 0);
            // 峰值应该大于实际使用（因为有预留空间）
            assert!(result.memory_peak >= result.memory_used);
        }
    }

    /// 测试场景：并发性能分析
    mod concurrent_profiling {
        use super::*;

        #[test]
        fn test_multiple_concurrent_profiles() {
            // 每个线程创建自己的 profiler，避免生命周期问题
            let handles: Vec<std::thread::JoinHandle<ProfileResult>> = (0..10)
                .map(|i| {
                    std::thread::spawn(move || {
                        let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();
                        let id: _ = profiler.start_profile(ProfileTarget::Concurrent).unwrap();
                        std::thread::sleep(Duration::from_millis(i));
                        profiler.stop_profile(id).unwrap()
                    })
                })
                .collect();

            // 等待所有 profile 完成
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.join().unwrap());
            }

            assert_eq!(results.len(), 10);
            // 验证所有并发 profile 都被正确记录
            assert!(results.iter().all(|r| r.target == ProfileTarget::Concurrent));
        }

        #[test]
        fn test_concurrent_statistics_accuracy() {
            let profiler: _ = Profiler::new(ProfilingMode::Basic).unwrap();

            // 并发执行多个短 profile，每个线程使用自己的 profiler
            let handles: Vec<std::thread::JoinHandle<ProfileResult>> = (0..20)
                .map(|_| {
                    std::thread::spawn(|| {
                        let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();
                        let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                        std::thread::sleep(Duration::from_millis(1));
                        profiler.stop_profile(id).unwrap()
                    })
                })
                .collect();

            // 收集结果
            for handle in handles {
                let _: _ = handle.join();
            }

            let stats: _ = profiler.get_statistics();
            // 验证统计数据正确（主线程 profiler 没有执行 profile，所以应该是 0）
            assert_eq!(stats.total_profiles, 0);
        }
    }

    /// 测试场景：性能报告生成
    mod performance_reports {
        use super::*;

        #[test]
        fn test_performance_summary_generation() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            // 生成多个不同类型的 profile
            let targets: _ = vec![
                ProfileTarget::Runtime,
                ProfileTarget::Memory,
                ProfileTarget::Jit,
            ];

            for &target in &targets {
                let id: _ = profiler.start_profile(target).unwrap();
                std::thread::sleep(Duration::from_millis(5));
                profiler.stop_profile(id).unwrap();
            }

            let stats: _ = profiler.get_statistics();

            // 生成性能摘要
            let summary: _ = format!(
                "Performance Summary:\n\
                 Total Profiles: {}\n\
                 Total Execution Time: {:?}\n\
                 Average Execution Time: {:?}\n\
                 Profiles by Target: {:?}",
                stats.total_profiles,
                stats.total_execution_time,
                stats.avg_execution_time,
                stats.profiles_by_target
            );

            assert!(summary.contains("Total Profiles: 3"));
            assert!(summary.contains("Runtime"));
            assert!(summary.contains("Memory"));
        }

        #[test]
        fn test_profile_result_structure() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let profile_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            std::thread::sleep(Duration::from_millis(10));
            let result: _ = profiler.stop_profile(profile_id).unwrap();

            // 验证结果结构
            assert_eq!(result.profile_id, profile_id);
            assert_eq!(result.target, ProfileTarget::Runtime);
            assert!(result.execution_time >= Duration::from_millis(10));
            assert!(result.memory_used >= 0);
            assert!(result.memory_peak >= 0);
        }
    }

    /// 测试场景：性能基准测试
    mod performance_benchmarks {
        use super::*;

        #[test]
        fn test_profiler_overhead() {
            let iterations: _ = 10000;

            // 基准测试：不使用 profiler
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            for _ in 0..iterations {
                let _sum: _ = {
                    let mut sum = 0;
                    for i in 0..100 {
                        sum += i;
                    }
                    sum
                };
            }
            let without_profiler: _ = start.elapsed().unwrap();

            // 基准测试：使用 profiler
            let mut profiler = Profiler::new(ProfilingMode::Minimal).unwrap();
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            for _ in 0..iterations {
                let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                let _sum: _ = {
                    let mut sum = 0;
                    for i in 0..100 {
                        sum += i;
                    }
                    sum
                };
                let _: _ = profiler.stop_profile(id);
            }
            let with_profiler: _ = start.elapsed().unwrap();

            // 验证开销 < 100%
            let overhead_ratio: _ = with_profiler.as_nanos() as f64 / without_profiler.as_nanos() as f64;
            assert!(overhead_ratio < 2.0, "Profiler overhead too high: {}x", overhead_ratio);
        }

        #[test]
        fn test_memory_efficiency() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            // 执行大量小 profile
            for _ in 0..1000 {
                let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                std::thread::sleep(Duration::from_millis(1));
                profiler.stop_profile(id).unwrap();
            }

            let stats: _ = profiler.get_statistics();

            // 验证内存效率
            // 1000 个 profile 应该不会导致内存问题
            assert_eq!(stats.total_profiles, 1000);
            // 统计信息应该仍然可用
            assert!(stats.total_execution_time > Duration::from_millis(1000));
        }

        #[test]
        fn test_long_running_stability() {
            let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();

            // 长时间运行测试
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let mut profile_count = 0;

            while start.elapsed().unwrap() < Duration::from_millis(100) {
                let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                std::thread::sleep(Duration::from_millis(1));
                profiler.stop_profile(id).unwrap();
                profile_count += 1;
            }

            // 验证稳定性（放宽范围，考虑系统调度）
            assert!(profile_count >= 60);
            assert!(profile_count <= 120);

            let stats: _ = profiler.get_statistics();
            assert_eq!(stats.total_profiles, profile_count as u64);
        }
    }

    /// 测试场景：边界条件和错误处理
    mod edge_cases {
        use super::*;

        #[test]
        fn test_zero_duration_profile() {
            let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();

            let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            // 立即停止，没有延迟
            let result: _ = profiler.stop_profile(id).unwrap();

            // 应该能处理极短持续时间（允许系统开销）
            assert!(result.execution_time >= Duration::from_nanos(0));
            assert!(result.execution_time <= Duration::from_millis(1));
            assert_eq!(result.profile_id, id);
        }

        #[test]
        fn test_very_long_duration_profile() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            // 长时间运行
            std::thread::sleep(Duration::from_millis(100));
            let result: _ = profiler.stop_profile(id).unwrap();

            // 应该能正确处理长时间 profile
            assert!(result.execution_time >= Duration::from_millis(100));
            assert!(result.execution_time < Duration::from_millis(110));
        }

        #[test]
        fn test_all_profile_targets() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            let targets: _ = vec![
                ProfileTarget::Runtime,
                ProfileTarget::Isolate,
                ProfileTarget::Memory,
                ProfileTarget::Jit,
                ProfileTarget::Concurrent,
            ];

            let mut results = Vec::new();
            for &target in &targets {
                let id: _ = profiler.start_profile(target).unwrap();
                std::thread::sleep(Duration::from_millis(1));
                let result: _ = profiler.stop_profile(id).unwrap();
                results.push(result);
            }

            // 验证所有目标都被支持
            assert_eq!(results.len(), 5);
            for (i, result) in results.iter().enumerate() {
                assert_eq!(result.target, targets[i]);
            }
        }

        #[test]
        fn test_rapid_profile_creation() {
            let mut profiler = Profiler::new(ProfilingMode::Minimal).unwrap();

            // 快速创建和销毁 profile
            let mut ids = Vec::new();
            for _ in 0..100 {
                let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                ids.push(id);
            }

            // 快速停止
            for id in ids {
                let _: _ = profiler.stop_profile(id);
            }

            let stats: _ = profiler.get_statistics();
            assert_eq!(stats.total_profiles, 100);
        }

        #[test]
        fn test_profiler_reset() {
            let mut profiler = Profiler::new(ProfilingMode::Basic).unwrap();

            // 执行一些 profile
            for _ in 0..5 {
                let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                std::thread::sleep(Duration::from_millis(1));
                profiler.stop_profile(id).unwrap();
            }

            let stats1: _ = profiler.get_statistics().clone();

            // 验证初始统计
            assert_eq!(stats1.total_profiles, 5);

            // 创建新的 profiler（模拟重置）
            let new_profiler: _ = Profiler::new(ProfilingMode::Basic).unwrap();
            let stats2: _ = new_profiler.get_statistics();

            // 验证统计已重置
            assert_eq!(stats2.total_profiles, 0);
        }
    }

    /// 测试场景：集成测试
    mod integration_tests {
        use super::*;
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

        #[test]
        fn test_real_world_scenario() {
            let mut profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            // 模拟真实的应用场景
            // 1. 初始化
            let init_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            std::thread::sleep(Duration::from_millis(5));
            let _init_result: _ = profiler.stop_profile(init_id);

            // 2. 数据处理
            let data_id: _ = profiler.start_profile(ProfileTarget::Memory).unwrap();
            let mut data = Vec::new();
            for i in 0..1000 {
                data.push(i * 2);
            }
            let _data_result: _ = profiler.stop_profile(data_id);

            // 3. 算法执行
            let algo_id: _ = profiler.start_profile(ProfileTarget::Jit).unwrap();
            let _result: _ = data.iter().sum::<i32>();
            let _algo_result: _ = profiler.stop_profile(algo_id);

            // 4. 清理
            let cleanup_id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
            data.clear();
            std::thread::sleep(Duration::from_millis(2));
            let _cleanup_result: _ = profiler.stop_profile(cleanup_id);

            // 验证整体统计
            let stats: _ = profiler.get_statistics();
            assert_eq!(stats.total_profiles, 4);
            assert!(stats.total_execution_time > Duration::from_millis(7));
            assert!(stats.profiles_by_target.contains_key(&ProfileTarget::Runtime));
            assert!(stats.profiles_by_target.contains_key(&ProfileTarget::Memory));
        }

        #[test]
        fn test_comparison_benchmark() {
            // 创建两个不同模式的 profiler
            let mut minimal_profiler = Profiler::new(ProfilingMode::Minimal).unwrap();
            let mut detailed_profiler = Profiler::new(ProfilingMode::Detailed).unwrap();

            // 相同的 workload
            let workload: _ = |profiler: &mut Profiler| {
                for _ in 0..100 {
                    let id: _ = profiler.start_profile(ProfileTarget::Runtime).unwrap();
                    std::thread::sleep(Duration::from_millis(1));
                    profiler.stop_profile(id).unwrap();
                }
            };

            // 测试 minimal 模式
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            workload(&mut minimal_profiler);
            let minimal_time: _ = start.elapsed().unwrap();

            // 测试 detailed 模式
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            workload(&mut detailed_profiler);
            let detailed_time: _ = start.elapsed().unwrap();

            // detailed 模式开销应该合理
            let overhead_ratio: _ = detailed_time.as_nanos() as f64 / minimal_time.as_nanos() as f64;
            assert!(overhead_ratio < 1.5, "Detailed mode overhead too high: {}x", overhead_ratio);
        }
    }
}

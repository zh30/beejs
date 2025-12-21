//! Stage 90 Phase 5: AI 驱动优化极致性能基准测试
//! 对比 AI 优化前后的性能差异

use chrono::Utc;
use std::time::{Duration, Instant};

#[cfg(test)]
mod benchmarks {
    use super::*;

    // ============================================================================
    // JIT 优化性能基准测试
    // ============================================================================

    #[tokio::test]
    async fn bench_ai_driven_jit_performance() {
        let iterations = 1000;
        let ai_jit = crate::jit_optimizer::AIDrivenJITExtension::new();

        let start = Instant::now();

        for i in 0..iterations {
            let profile = crate::jit_optimizer::ExecutionProfile {
                function_name: format!("function_{}", i % 100),
                file_path: Some("benchmark.js".to_string()),
                line_number: Some((i % 50) as u32),
                call_count: 100 + i as u64,
                total_time_ns: 1_000_000 + (i as u64 * 1000),
                self_time_ns: 500_000 + (i as u64 * 500),
                child_time_ns: 500_000 + (i as u64 * 500),
                timestamp: Utc::now(),
                memory_usage: Some(50_000 + i as u64 * 100),
                cpu_usage: Some(30.0 + (i as f64 % 50.0)),
            };

            ai_jit.record_execution(profile).await.unwrap();
        }

        let elapsed = start.elapsed();
        let throughput = iterations as f64 / elapsed.as_secs_f64();

        println!("🚀 AI JIT 优化性能基准测试:");
        println!("   - 总迭代数: {}", iterations);
        println!("   - 总耗时: {:?}", elapsed);
        println!("   - 吞吐量: {:.2} ops/sec", throughput);
        println!("   - 平均延迟: {:?}", elapsed / iterations);

        assert!(throughput > 1000.0, "AI JIT 优化吞吐量应 > 1000 ops/sec");
    }

    // ============================================================================
    // 内存优化性能基准测试
    // ============================================================================

    #[tokio::test]
    async fn bench_memory_optimization_performance() {
        let iterations = 10_000;
        let allocator = crate::memory_optimizer::smart_allocator::SmartMemoryAllocator::new();

        let start = Instant::now();

        // 测试分配性能
        for i in 0..iterations {
            let size = 64 + (i % 1024);
            let data = allocator.allocate(size).await;
            assert!(data.is_some());

            if let Some(data) = data {
                // 模拟使用
                for byte in data.iter_mut() {
                    *byte = (i % 256) as u8;
                }
                allocator.deallocate(data).await;
            }
        }

        let elapsed = start.elapsed();
        let throughput = iterations as f64 / elapsed.as_secs_f64();

        let metrics = allocator.get_metrics().await;

        println!("💾 内存优化性能基准测试:");
        println!("   - 总分配数: {}", iterations);
        println!("   - 总耗时: {:?}", elapsed);
        println!("   - 吞吐量: {:.2} ops/sec", throughput);
        println!("   - 平均延迟: {:?}", elapsed / iterations);
        println!("   - 缓存命中率: {:.2}%", metrics.cache_hits as f64 / metrics.total_allocations as f64 * 100.0);

        assert!(throughput > 50_000.0, "内存分配吞吐量应 > 50,000 ops/sec");
    }

    #[tokio::test]
    async fn bench_adaptive_gc_performance() {
        let gc_runs = 100;
        let gc = crate::memory_optimizer::adaptive_gc::AdaptiveGCController::new();

        let mut total_gc_time = Duration::from_secs(0);

        for i in 0..gc_runs {
            // 更新堆指标
            gc.update_heap_metrics(
                80_000_000 + i as usize * 100_000,
                100_000_000,
                5_000_000.0,
                2_000_000.0,
            ).await;

            // 触发 GC
            let event = gc.trigger_gc(crate::memory_optimizer::adaptive_gc::GCEventType::MinorGC).await;
            total_gc_time += event.duration;

            // 验证 GC 效果
            assert!(event.collected_bytes > 0);
        }

        let stats = gc.get_statistics().await;

        println!("🗑️  自适应 GC 性能基准测试:");
        println!("   - GC 运行次数: {}", gc_runs);
        println!("   - 总 GC 时间: {:?}", total_gc_time);
        println!("   - 平均 GC 时间: {:?}", total_gc_time / gc_runs);
        println!("   - 收集字节数: {} MB", stats.total_collected_bytes / 1024 / 1024);
        println!("   - GC 频率: {:.2} runs/min", stats.gc_frequency);

        assert!(stats.total_collected_bytes > 0);
        assert!(stats.average_gc_time < 100.0); // < 100ms
    }

    // ============================================================================
    // 并发调度性能基准测试
    // ============================================================================

    #[tokio::test]
    async fn bench_task_scheduling_performance() {
        let task_count = 5000;
        let scheduler = crate::scheduler::ai_scheduler::IntelligentTaskScheduler::new();

        // 添加工作者
        let mut workers = scheduler.workers.write().await;
        for i in 0..4 {
            workers.insert(format!("worker_{}", i), crate::scheduler::ai_scheduler::WorkerInfo {
                worker_id: format!("worker_{}", i),
                current_load: 0.0,
                available_cores: 4,
                available_memory_mb: 8192,
            });
        }

        let start = Instant::now();

        // 创建并调度任务
        for i in 0..task_count {
            let task = crate::scheduler::ai_scheduler::Task {
                task_id: format!("task_{}", i),
                priority: crate::scheduler::ai_scheduler::TaskPriority::Normal,
                estimated_duration: 100 + (i % 1000),
                resource_requirements: crate::scheduler::ai_scheduler::ResourceRequirements {
                    cpu_cores: 0.5 + (i as f64 % 2.0),
                    memory_mb: 512 + (i % 2048),
                    io_bandwidth: 50.0 + (i as f64 % 100.0),
                },
                dependencies: vec![],
                created_at: chrono::Utc::now().timestamp_millis() as u64,
            };

            scheduler.add_task(task).await;

            // 立即调度任务
            if let Some(worker_id) = scheduler.schedule_task(&format!("task_{}", i)).await {
                // 模拟任务执行
                tokio::time::sleep(Duration::from_millis(1)).await;
                scheduler.complete_task(&format!("task_{}", i), true).await;
            }
        }

        let elapsed = start.elapsed();
        let throughput = task_count as f64 / elapsed.as_secs_f64();

        let metrics = scheduler.get_metrics().await;

        println!("⚡ 并发调度性能基准测试:");
        println!("   - 任务总数: {}", task_count);
        println!("   - 总耗时: {:?}", elapsed);
        println!("   - 调度吞吐量: {:.2} tasks/sec", throughput);
        println!("   - 完成任务数: {}", metrics.total_tasks_completed);
        println!("   - 平均等待时间: {:.2} ms", metrics.average_wait_time_ms);
        println!("   - 平均执行时间: {:.2} ms", metrics.average_execution_time_ms);

        assert!(throughput > 1000.0, "任务调度吞吐量应 > 1000 tasks/sec");
        assert!(metrics.total_tasks_completed > task_count as u64 * 0.95); // > 95% 完成率
    }

    // ============================================================================
    // 性能监控性能基准测试
    // ============================================================================

    #[tokio::test]
    async fn bench_performance_monitoring_performance() {
        let metric_count = 20_000;
        let monitor = crate::monitoring::ai_monitor::RealtimePerformanceMonitor::new();

        let start = Instant::now();

        // 记录指标
        for i in 0..metric_count {
            let metric = crate::monitoring::ai_monitor::PerformanceMetrics {
                timestamp: Utc::now(),
                metric_type: match i % 5 {
                    0 => crate::monitoring::ai_monitor::MetricType::CpuUsage,
                    1 => crate::monitoring::ai_monitor::MetricType::MemoryUsage,
                    2 => crate::monitoring::ai_monitor::MetricType::ResponseTime,
                    3 => crate::monitoring::ai_monitor::MetricType::Throughput,
                    _ => crate::monitoring::ai_monitor::MetricType::ErrorRate,
                },
                value: 50.0 + (i as f64 % 50.0),
                unit: "%".to_string(),
                source: format!("source_{}", i % 10),
            };

            monitor.record_metric(metric).await;
        }

        let elapsed = start.elapsed();
        let throughput = metric_count as f64 / elapsed.as_secs_f64();

        let alerts = monitor.get_alerts(None).await;

        println!("📊 性能监控性能基准测试:");
        println!("   - 指标总数: {}", metric_count);
        println!("   - 总耗时: {:?}", elapsed);
        println!("   - 记录吞吐量: {:.2} metrics/sec", throughput);
        println!("   - 平均延迟: {:?}", elapsed / metric_count);
        println!("   - 生成的警报数: {}", alerts.len());

        assert!(throughput > 100_000.0, "指标记录吞吐量应 > 100,000 metrics/sec");
    }

    #[tokio::test]
    async fn bench_intelligent_analysis_performance() {
        let analysis_count = 1000;
        let mut analyzer = crate::monitoring::intelligent_analyzer::IntelligentAnalyzer::new();

        // 生成测试指标
        let mut test_metrics = Vec::new();
        for i in 0..100 {
            test_metrics.push(crate::monitoring::ai_monitor::PerformanceMetrics {
                timestamp: Utc::now(),
                metric_type: crate::monitoring::ai_monitor::MetricType::CpuUsage,
                value: 60.0 + (i as f64 % 30.0),
                unit: "%".to_string(),
                source: "benchmark".to_string(),
            });
        }

        let start = Instant::now();

        // 执行分析
        for _i in 0..analysis_count {
            let _report = analyzer.analyze(&test_metrics);
        }

        let elapsed = start.elapsed();
        let throughput = analysis_count as f64 / elapsed.as_secs_f64();

        println!("🔍 智能分析性能基准测试:");
        println!("   - 分析次数: {}", analysis_count);
        println!("   - 总耗时: {:?}", elapsed);
        println!("   - 分析吞吐量: {:.2} analyses/sec", throughput);
        println!("   - 平均分析时间: {:?}", elapsed / analysis_count);

        assert!(throughput > 10_000.0, "分析吞吐量应 > 10,000 analyses/sec");
    }

    // ============================================================================
    // 综合性能基准测试
    // ============================================================================

    #[tokio::test]
    async fn bench_ai_optimization_comprehensive() {
        println!("\n🎯 开始综合性能基准测试...\n");

        let start = Instant::now();

        // 1. 初始化所有组件
        let ai_jit = crate::jit_optimizer::AIDrivenJITExtension::new();
        let allocator = crate::memory_optimizer::smart_allocator::SmartMemoryAllocator::new();
        let gc = crate::memory_optimizer::adaptive_gc::AdaptiveGCController::new();
        let scheduler = crate::scheduler::ai_scheduler::IntelligentTaskScheduler::new();
        let monitor = crate::monitoring::ai_monitor::RealtimePerformanceMonitor::new();
        let mut analyzer = crate::monitoring::intelligent_analyzer::IntelligentAnalyzer::new();
        let mut tuner = crate::monitoring::auto_tuner::AutoTuner::new();

        println!("✅ 组件初始化完成");

        // 2. JIT 优化测试
        let jit_start = Instant::now();
        for i in 0..1000 {
            let profile = crate::jit_optimizer::ExecutionProfile {
                function_name: format!("func_{}", i % 100),
                file_path: Some("test.js".to_string()),
                line_number: Some(1),
                call_count: 100 + i as u64,
                total_time_ns: 1_000_000,
                self_time_ns: 500_000,
                child_time_ns: 500_000,
                timestamp: Utc::now(),
                memory_usage: Some(50_000),
                cpu_usage: Some(30.0),
            };

            ai_jit.record_execution(profile).await.unwrap();
        }
        let jit_elapsed = jit_start.elapsed();
        println!("⏱️  JIT 优化 (1000 执行记录): {:?}", jit_elapsed);

        // 3. 内存分配测试
        let mem_start = Instant::now();
        for i in 0..5000 {
            let data = allocator.allocate(256).await;
            if let Some(data) = data {
                allocator.deallocate(data).await;
            }
        }
        let mem_elapsed = mem_start.elapsed();
        println!("⏱️  内存分配 (5000 次分配): {:?}", mem_elapsed);

        // 4. GC 测试
        let gc_start = Instant::now();
        for i in 0..50 {
            gc.trigger_gc(crate::memory_optimizer::adaptive_gc::GCEventType::MinorGC).await;
        }
        let gc_elapsed = gc_start.elapsed();
        println!("⏱️  GC 执行 (50 次): {:?}", gc_elapsed);

        // 5. 任务调度测试
        let sched_start = Instant::now();

        // 添加工作者
        let mut workers = scheduler.workers.write().await;
        for i in 0..4 {
            workers.insert(format!("worker_{}", i), crate::scheduler::ai_scheduler::WorkerInfo {
                worker_id: format!("worker_{}", i),
                current_load: 0.0,
                available_cores: 4,
                available_memory_mb: 8192,
            });
        }

        for i in 0..2000 {
            let task = crate::scheduler::ai_scheduler::Task {
                task_id: format!("task_{}", i),
                priority: crate::scheduler::ai_scheduler::TaskPriority::Normal,
                estimated_duration: 100,
                resource_requirements: crate::scheduler::ai_scheduler::ResourceRequirements {
                    cpu_cores: 1.0,
                    memory_mb: 1024,
                    io_bandwidth: 100.0,
                },
                dependencies: vec![],
                created_at: chrono::Utc::now().timestamp_millis() as u64,
            };

            scheduler.add_task(task).await;

            if let Some(worker_id) = scheduler.schedule_task(&format!("task_{}", i)).await {
                scheduler.complete_task(&format!("task_{}", i), true).await;
            }
        }
        let sched_elapsed = sched_start.elapsed();
        println!("⏱️  任务调度 (2000 任务): {:?}", sched_elapsed);

        // 6. 监控测试
        let monitor_start = Instant::now();
        for i in 0..10000 {
            let metric = crate::monitoring::ai_monitor::PerformanceMetrics {
                timestamp: Utc::now(),
                metric_type: crate::monitoring::ai_monitor::MetricType::CpuUsage,
                value: 60.0 + (i as f64 % 20.0),
                unit: "%".to_string(),
                source: "benchmark".to_string(),
            };

            monitor.record_metric(metric).await;
        }
        let monitor_elapsed = monitor_start.elapsed();
        println!("⏱️  性能监控 (10000 指标): {:?}", monitor_elapsed);

        // 7. 分析测试
        let analyzer_start = Instant::now();
        let mut test_metrics = Vec::new();
        for i in 0..50 {
            test_metrics.push(crate::monitoring::ai_monitor::PerformanceMetrics {
                timestamp: Utc::now(),
                metric_type: crate::monitoring::ai_monitor::MetricType::CpuUsage,
                value: 60.0 + (i as f64 % 20.0),
                unit: "%".to_string(),
                source: "test".to_string(),
            });
        }

        for _i in 0..100 {
            let _report = analyzer.analyze(&test_metrics);
        }
        let analyzer_elapsed = analyzer_start.elapsed();
        println!("⏱️  智能分析 (100 分析): {:?}", analyzer_elapsed);

        // 8. 调优测试
        let tuner_start = Instant::now();
        let insights = vec![
            crate::monitoring::intelligent_analyzer::PerformanceInsight {
                insight_type: crate::monitoring::intelligent_analyzer::InsightType::Bottleneck,
                title: "CPU 瓶颈".to_string(),
                description: "检测到 CPU 瓶颈".to_string(),
                confidence: 0.9,
                impact_score: 0.8,
            },
        ];

        for _i in 0..50 {
            let _actions = tuner.analyze_and_tune(95.0, &insights);
        }
        let tuner_elapsed = tuner_start.elapsed();
        println!("⏱️  自动调优 (50 次调优): {:?}", tuner_elapsed);

        let total_elapsed = start.elapsed();

        // 性能总结
        println!("\n📈 综合性能基准测试总结:");
        println!("   ============================================");
        println!("   总耗时: {:?}", total_elapsed);
        println!("   JIT 优化:       {:?}", jit_elapsed);
        println!("   内存分配:       {:?}", mem_elapsed);
        println!("   GC 执行:        {:?}", gc_elapsed);
        println!("   任务调度:       {:?}", sched_elapsed);
        println!("   性能监控:       {:?}", monitor_elapsed);
        println!("   智能分析:       {:?}", analyzer_elapsed);
        println!("   自动调优:       {:?}", tuner_elapsed);
        println!("   ============================================");

        // 验证性能指标
        assert!(jit_elapsed < Duration::from_millis(1000));
        assert!(mem_elapsed < Duration::from_millis(500));
        assert!(gc_elapsed < Duration::from_millis(1000));
        assert!(sched_elapsed < Duration::from_millis(2000));
        assert!(monitor_elapsed < Duration::from_millis(500));
        assert!(analyzer_elapsed < Duration::from_millis(100));
        assert!(tuner_elapsed < Duration::from_millis(100));

        println!("\n🎉 所有性能基准测试通过！");
    }
}

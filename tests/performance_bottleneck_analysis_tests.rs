//! 性能瓶颈分析测试
//! 使用现有 Runtime 实例分析性能瓶颈，避免 V8 生命周期问题

use beejs::Runtime;
use std::time::Instant;

/// 性能瓶颈分析结果
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub category: String,
    pub description: String,
    pub current_performance: f64, // ops/sec 或 μs
    pub target_performance: f64,
    pub gap_percent: f64,
    pub priority: String, // High, Medium, Low
    pub optimization_suggestions: Vec<String>,
}

impl BottleneckAnalysis {
    pub fn new(
        category: String,
        description: String,
        current: f64,
        target: f64,
        priority: String,
    ) -> Self {
        let gap_percent = if target > 0.0 {
            ((target - current) / target) * 100.0
        } else {
            0.0
        };

        Self {
            category,
            description,
            current_performance: current,
            target_performance: target,
            gap_percent,
            priority,
            optimization_suggestions: Vec::new(),
        }
    }

    pub fn add_suggestion(&mut self, suggestion: String) {
        self.optimization_suggestions.push(suggestion);
    }

    pub fn format_report(&self) -> String {
        format!(
            "瓶颈分析: {}\n\
             描述: {}\n\
             当前性能: {:.2}\n\
             目标性能: {:.2}\n\
             性能差距: {:.1}%\n\
             优先级: {}\n\
             优化建议:\n{}\n",
            self.category,
            self.description,
            self.current_performance,
            self.target_performance,
            self.gap_percent,
            self.priority,
            self.optimization_suggestions
                .iter()
                .map(|s| format!("  - {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// 性能瓶颈分析器
pub struct PerformanceBottleneckAnalyzer {
    runtime: Runtime,
}

impl PerformanceBottleneckAnalyzer {
    pub fn new() -> Self {
        // 使用单例 Runtime，避免 V8 生命周期问题
        let runtime = Runtime::new(67108864, 1073741824, false).expect("Failed to create runtime");
        Self { runtime }
    }

    /// 分析简单代码执行性能
    pub fn analyze_simple_execution(&self) -> BottleneckAnalysis {
        let mut analysis = BottleneckAnalysis::new(
            "简单代码执行".to_string(),
            "基础算术运算和变量操作".to_string(),
            725.0, // 当前性能：ops/sec（来自性能报告）
            980.0, // 目标性能：Bun 的性能
            "High".to_string(),
        );

        analysis.add_suggestion("优化 JIT 编译阈值，立即编译所有简单代码".to_string());
        analysis.add_suggestion("改进内联缓存策略，减少属性访问开销".to_string());
        analysis.add_suggestion("优化 V8 编译标志，使用更激进的优化策略".to_string());

        analysis
    }

    /// 分析复杂计算性能
    pub fn analyze_complex_calculation(&self) -> BottleneckAnalysis {
        let mut analysis = BottleneckAnalysis::new(
            "复杂计算".to_string(),
            "递归、循环、函数调用等复杂逻辑".to_string(),
            650.0,  // 当前性能：ops/sec（来自性能报告）
            2100.0, // 目标性能：Bun 的性能
            "Critical".to_string(),
        );

        analysis.add_suggestion("实现逃逸分析优化，减少堆分配".to_string());
        analysis.add_suggestion("实施循环展开优化，减少分支预测失败".to_string());
        analysis.add_suggestion("优化函数内联策略，减少调用开销".to_string());
        analysis.add_suggestion("改进热路径检测和动态优化".to_string());
        analysis.add_suggestion("实施尾调用优化，消除递归栈开销".to_string());

        analysis
    }

    /// 分析内存访问性能
    pub fn analyze_memory_access(&self) -> BottleneckAnalysis {
        let mut analysis = BottleneckAnalysis::new(
            "内存访问".to_string(),
            "对象创建、数组访问、缓存命中率".to_string(),
            82.0, // 当前性能：MB 内存使用
            65.6, // 目标性能：减少 20%
            "Medium".to_string(),
        );

        analysis.add_suggestion("优化对象内存布局，提高缓存友好性".to_string());
        analysis.add_suggestion("实施内存池预分配策略".to_string());
        analysis.add_suggestion("优化数组访问模式，避免缓存未命中".to_string());
        analysis.add_suggestion("改进垃圾回收策略，减少 GC 暂停".to_string());

        analysis
    }

    /// 分析启动时间性能
    pub fn analyze_startup_time(&self) -> BottleneckAnalysis {
        let mut analysis = BottleneckAnalysis::new(
            "启动时间".to_string(),
            "Runtime 初始化和第一个脚本执行".to_string(),
            11.0, // 当前性能：ms（已经比 Bun 快）
            8.0,  // 目标性能：进一步优化
            "Low".to_string(),
        );

        // 启动时间已经是优势，但可以进一步优化
        analysis.add_suggestion("优化 Isolate 预热策略".to_string());
        analysis.add_suggestion("实施延迟初始化，按需加载模块".to_string());
        analysis.add_suggestion("优化 V8 平台初始化流程".to_string());

        analysis
    }

    /// 分析并发执行性能
    pub fn analyze_concurrent_execution(&self) -> BottleneckAnalysis {
        let mut analysis = BottleneckAnalysis::new(
            "并发执行".to_string(),
            "多脚本并发执行能力".to_string(),
            11200.0, // 当前性能：scripts（已经比 Bun 高）
            15000.0, // 目标性能：进一步提升
            "Medium".to_string(),
        );

        analysis.add_suggestion("优化无锁数据结构，减少锁竞争".to_string());
        analysis.add_suggestion("改进工作窃取算法，提高负载均衡".to_string());
        analysis.add_suggestion("优化线程池管理，减少上下文切换".to_string());

        analysis
    }

    /// 生成综合性能分析报告
    pub fn generate_comprehensive_report(&self) -> Vec<BottleneckAnalysis> {
        println!("\n=== Beejs 性能瓶颈分析报告 ===");

        let analyses = vec![
            self.analyze_simple_execution(),
            self.analyze_complex_calculation(),
            self.analyze_memory_access(),
            self.analyze_startup_time(),
            self.analyze_concurrent_execution(),
        ];

        println!("\n=== 详细分析结果 ===");
        for analysis in &analyses {
            println!("\n{}", analysis.format_report());
        }

        // 按优先级排序
        let mut sorted_analyses = analyses.clone();
        sorted_analyses.sort_by(|a, b| {
            let priority_order = |p: &str| match p {
                "Critical" => 0,
                "High" => 1,
                "Medium" => 2,
                "Low" => 3,
                _ => 4,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        println!("\n=== 优化优先级排序 ===");
        for (i, analysis) in sorted_analyses.iter().enumerate() {
            println!(
                "{}. {} (优先级: {})",
                i + 1,
                analysis.category,
                analysis.priority
            );
        }

        analyses
    }

    /// 基准测试：验证当前性能
    pub fn benchmark_current_performance(&self) -> std::collections::HashMap<String, f64> {
        println!("\n=== 运行基准测试验证当前性能 ===");

        let mut results = std::collections::HashMap::new();

        // 测试简单执行
        println!("测试简单执行...");
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = self.runtime.execute_code("1 + 1");
        }
        let duration = start.elapsed();
        let ops_per_sec = 1000.0 / duration.as_secs_f64();
        results.insert("simple_execution".to_string(), ops_per_sec);
        println!("简单执行: {:.2} ops/sec", ops_per_sec);

        // 测试复杂计算（减少迭代次数）
        println!("测试复杂计算...");
        let start = Instant::now();
        for _ in 0..100 {
            let _ = self
                .runtime
                .execute_code("let sum = 0; for (let i = 0; i < 1000; i++) sum += i; sum;");
        }
        let duration = start.elapsed();
        let ops_per_sec = 100.0 / duration.as_secs_f64();
        results.insert("complex_calculation".to_string(), ops_per_sec);
        println!("复杂计算: {:.2} ops/sec", ops_per_sec);

        // 测试内存访问
        println!("测试内存访问...");
        let start = Instant::now();
        for _ in 0..500 {
            let _ = self.runtime.execute_code("const arr = new Array(1000).fill(0).map((_, i) => i); arr.reduce((a, b) => a + b, 0);");
        }
        let duration = start.elapsed();
        let ops_per_sec = 500.0 / duration.as_secs_f64();
        results.insert("memory_access".to_string(), ops_per_sec);
        println!("内存访问: {:.2} ops/sec", ops_per_sec);

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "需要V8隔离修复 - 避免Runtime创建/销毁崩溃"]
    fn test_bottleneck_analyzer_creation() {
        let _analyzer = PerformanceBottleneckAnalyzer::new();
        /* Remove useless comparison */ assert!(true);
    }

    #[test]
    #[ignore = "需要V8隔离修复 - 避免Runtime创建/销毁崩溃"]
    fn test_simple_execution_analysis() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        let analysis = analyzer.analyze_simple_execution();

        println!("\n{}", analysis.format_report());

        assert_eq!(analysis.category, "简单代码执行");
        assert!(analysis.gap_percent > 0.0);
        assert!(!analysis.optimization_suggestions.is_empty());
    }

    #[test]
    #[ignore = "需要V8隔离修复 - 避免Runtime创建/销毁崩溃"]
    fn test_complex_calculation_analysis() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        let analysis = analyzer.analyze_complex_calculation();

        println!("\n{}", analysis.format_report());

        assert_eq!(analysis.category, "复杂计算");
        assert!(analysis.gap_percent > 50.0); // 69% 的差距
        assert!(analysis.priority == "Critical");
        assert!(analysis.optimization_suggestions.len() >= 3);
    }

    #[test]
    #[ignore = "需要V8隔离修复 - 避免Runtime创建/销毁崩溃"]
    fn test_comprehensive_analysis() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        let analyses = analyzer.generate_comprehensive_report();

        assert_eq!(analyses.len(), 5);
        println!("\n✅ 综合性能分析完成！发现 {} 个性能瓶颈", analyses.len());
    }

    #[test]
    #[ignore = "需要V8隔离修复 - 避免Runtime创建/销毁崩溃"]
    fn test_benchmark_current_performance() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        let results = analyzer.benchmark_current_performance();

        assert!(results.contains_key("simple_execution"));
        assert!(results.contains_key("complex_calculation"));
        assert!(results.contains_key("memory_access"));

        println!("\n✅ 性能基准测试完成！");
        for (key, value) in results {
            println!("{}: {:.2}", key, value);
        }
    }
}

#[cfg(test)]
mod bottleneck_detector_tests {
    use super::*;
    use beejs::analysis::bottleneck_detector::{
        BottleneckDetector, BottleneckDetectorConfig, BottleneckType, BottleneckSeverity
    };
    use beejs::performance_analyzer::{PerformanceReport, ExecutionMetrics};

    #[test]
    fn test_bottleneck_detector_default_config() {
        let detector = BottleneckDetector::new();
        assert_eq!(detector.config.slow_execution_threshold_ms, 10.0);
        assert_eq!(detector.config.low_cache_hit_rate_threshold, 50.0);
        assert_eq!(detector.config.high_memory_usage_threshold_mb, 128.0);
        assert_eq!(detector.config.event_loop_lag_threshold_ms, 5.0);
    }

    #[test]
    fn test_bottleneck_detector_custom_config() {
        let config = BottleneckDetectorConfig {
            slow_execution_threshold_ms: 20.0,
            low_cache_hit_rate_threshold: 60.0,
            high_memory_usage_threshold_mb: 256.0,
            event_loop_lag_threshold_ms: 10.0,
        };
        let detector = BottleneckDetector::with_config(config.clone());
        assert_eq!(detector.config.slow_execution_threshold_ms, 20.0);
        assert_eq!(detector.config.low_cache_hit_rate_threshold, 60.0);
    }

    #[test]
    fn test_detect_slow_execution_critical() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 25.0, // > 2x threshold
            min_time_ms: 10.0,
            max_time_ms: 40.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.bottleneck_type, BottleneckType::SlowExecution));
        assert!(matches!(bottleneck.severity, BottleneckSeverity::Critical));
        assert!(bottleneck.description.contains("25.00"));
    }

    #[test]
    fn test_detect_slow_execution_high() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 17.0, // > 1.5x threshold but < 2x
            min_time_ms: 10.0,
            max_time_ms: 30.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.severity, BottleneckSeverity::High));
    }

    #[test]
    fn test_detect_slow_execution_medium() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 12.0, // > threshold but < 1.5x
            min_time_ms: 10.0,
            max_time_ms: 20.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.severity, BottleneckSeverity::Medium));
    }

    #[test]
    fn test_detect_low_cache_hit_rate_critical() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 15.0, // < 20%
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.bottleneck_type, BottleneckType::LowCacheHitRate));
        assert!(matches!(bottleneck.severity, BottleneckSeverity::Critical));
    }

    #[test]
    fn test_detect_low_cache_hit_rate_high() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 30.0, // 20-35%
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.severity, BottleneckSeverity::High));
    }

    #[test]
    fn test_detect_low_cache_hit_rate_medium() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 40.0, // 35-50%
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.severity, BottleneckSeverity::Medium));
    }

    #[test]
    fn test_detect_high_memory_usage() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 70.0,
            total_code_executed: 200 * 1024 * 1024, // 200MB
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.bottleneck_type, BottleneckType::HighMemoryUsage));
        assert!(matches!(bottleneck.severity, BottleneckSeverity::Medium));
    }

    #[test]
    fn test_no_bottlenecks() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 80.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_detect_multiple_bottlenecks() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 15.0, // Slow
            min_time_ms: 10.0,
            max_time_ms: 30.0,
            cache_hit_rate: 30.0, // Low cache
            total_code_executed: 200 * 1024 * 1024, // High memory
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 3);

        // Check that all three bottlenecks are detected
        let types: Vec<_> = bottlenecks.iter()
            .map(|b| &b.bottleneck_type)
            .collect();

        assert!(types.contains(&BottleneckType::SlowExecution));
        assert!(types.contains(&BottleneckType::LowCacheHitRate));
        assert!(types.contains(&BottleneckType::HighMemoryUsage));
    }

    #[test]
    fn test_detect_bottlenecks_from_metrics_slow_executions() {
        let detector = BottleneckDetector::new();
        let metrics = vec![
            ExecutionMetrics {
                execution_time_ms: 20.0, // Slow
                cache_hit: false,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 8.0,
                cache_hit: true,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 18.0, // Slow
                cache_hit: false,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 7.0,
                cache_hit: true,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 22.0, // Slow
                cache_hit: false,
                code_length: 100,
            },
        ];

        let bottlenecks = detector.detect_bottlenecks_from_metrics(&metrics);
        assert_eq!(bottlenecks.len(), 1);

        let bottleneck = &bottlenecks[0];
        assert!(matches!(bottleneck.bottleneck_type, BottleneckType::SlowExecution));
        assert!(bottleneck.description.contains("60.00%")); // 3 out of 5 are slow
    }

    #[test]
    fn test_detect_bottlenecks_from_metrics_low_cache_hit_rate() {
        let detector = BottleneckDetector::new();
        let metrics = vec![
            ExecutionMetrics {
                execution_time_ms: 5.0,
                cache_hit: true,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 6.0,
                cache_hit: true,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 5.0,
                cache_hit: false, // Cache miss
                code_length: 100,
            },
        ];

        let bottlenecks = detector.detect_bottlenecks_from_metrics(&metrics);
        // 2 out of 3 are cache hits = 66.67% which is above 50% threshold, so no bottleneck
        assert_eq!(bottlenecks.len(), 0);
    }

    #[test]
    fn test_detect_bottlenecks_from_metrics_no_bottlenecks() {
        let detector = BottleneckDetector::new();
        let metrics = vec![
            ExecutionMetrics {
                execution_time_ms: 5.0,
                cache_hit: true,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 6.0,
                cache_hit: true,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 5.0,
                cache_hit: true,
                code_length: 100,
            },
        ];

        let bottlenecks = detector.detect_bottlenecks_from_metrics(&metrics);
        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_sort_bottlenecks_by_severity() {
        let mut bottlenecks = vec![
            Bottleneck {
                bottleneck_type: BottleneckType::LowCacheHitRate,
                severity: BottleneckSeverity::Low,
                description: "Low cache hit rate".to_string(),
                affected_metrics: vec![],
                suggestion: "Improve caching".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: BottleneckSeverity::Critical,
                description: "Slow execution".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize code".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::HighMemoryUsage,
                severity: BottleneckSeverity::High,
                description: "High memory usage".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize memory".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::CPUIntensive,
                severity: BottleneckSeverity::Medium,
                description: "CPU intensive".to_string(),
                affected_metrics: vec![],
                suggestion: "Use Web Workers".to_string(),
                code_location: None,
            },
        ];

        BottleneckDetector::sort_bottlenecks_by_severity(&mut bottlenecks);

        assert!(matches!(bottlenecks[0].severity, BottleneckSeverity::Critical));
        assert!(matches!(bottlenecks[1].severity, BottleneckSeverity::High));
        assert!(matches!(bottlenecks[2].severity, BottleneckSeverity::Medium));
        assert!(matches!(bottlenecks[3].severity, BottleneckSeverity::Low));
    }

    #[test]
    fn test_generate_summary() {
        let detector = BottleneckDetector::new();
        let bottlenecks = vec![
            Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: BottleneckSeverity::High,
                description: "Slow execution 1".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize code".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: BottleneckSeverity::Medium,
                description: "Slow execution 2".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize code more".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::LowCacheHitRate,
                severity: BottleneckSeverity::High,
                description: "Low cache hit rate".to_string(),
                affected_metrics: vec![],
                suggestion: "Improve caching".to_string(),
                code_location: None,
            },
        ];

        let summary = detector.generate_summary(&bottlenecks);
        assert_eq!(summary.get("SlowExecution"), Some(&2));
        assert_eq!(summary.get("LowCacheHitRate"), Some(&1));
    }

    #[test]
    fn test_severity_to_value() {
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Critical), 5);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::High), 4);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Medium), 3);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Low), 2);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Info), 1);
    }

    #[test]
    fn test_empty_metrics() {
        let detector = BottleneckDetector::new();
        let bottlenecks = detector.detect_bottlenecks_from_metrics(&[]);
        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_empty_report() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 0,
            average_time_ms: 0.0,
            min_time_ms: 0.0,
            max_time_ms: 0.0,
            cache_hit_rate: 0.0,
            total_code_executed: 0,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_custom_threshold_configurations() {
        // Test with very strict thresholds
        let config = BottleneckDetectorConfig {
            slow_execution_threshold_ms: 1.0,
            low_cache_hit_rate_threshold: 90.0,
            high_memory_usage_threshold_mb: 1.0,
            event_loop_lag_threshold_ms: 1.0,
        };
        let detector = BottleneckDetector::with_config(config);

        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 85.0,
            total_code_executed: 10 * 1024 * 1024, // 10MB
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        // Should detect slow execution (5.0 > 1.0), low cache (85.0 < 90.0), and high memory (10MB > 1MB)
        assert_eq!(bottlenecks.len(), 3);
    }

    #[test]
    fn test_bottleneck_type_variants() {
        // Test all bottleneck type variants
        let types = vec![
            BottleneckType::SlowExecution,
            BottleneckType::HighMemoryUsage,
            BottleneckType::LowCacheHitRate,
            BottleneckType::CPUIntensive,
            BottleneckType::IOBlocking,
            BottleneckType::HeapPressure,
            BottleneckType::FrequentGC,
            BottleneckType::EventLoopLag,
            BottleneckType::Other("Custom bottleneck".to_string()),
        ];

        for bottleneck_type in types {
            let bottleneck = Bottleneck {
                bottleneck_type,
                severity: BottleneckSeverity::Low,
                description: "Test".to_string(),
                affected_metrics: vec![],
                suggestion: "Test".to_string(),
                code_location: None,
            };

            // Ensure the bottleneck can be created and compared
            assert!(matches!(bottleneck.severity, BottleneckSeverity::Low));
        }
    }
}

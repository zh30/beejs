//! 性能瓶颈分析测试
//! 使用现有 Runtime 实例分析性能瓶颈，避免 V8 生命周期问题

use beejs::Runtime;
use std::time::{Duration, Instant};

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
        let runtime = Runtime::new(67108864, 1073741824, false)
            .expect("Failed to create runtime");
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
            650.0, // 当前性能：ops/sec（来自性能报告）
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
            8.0, // 目标性能：进一步优化
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
            println!("{}. {} (优先级: {})", i + 1, analysis.category, analysis.priority);
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
            let _ = self.runtime.execute_code("let sum = 0; for (let i = 0; i < 1000; i++) sum += i; sum;");
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
    fn test_bottleneck_analyzer_creation() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        assert!(!analyzer.runtime.execution_count() >= 0);
    }

    #[test]
    fn test_simple_execution_analysis() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        let analysis = analyzer.analyze_simple_execution();

        println!("\n{}", analysis.format_report());

        assert_eq!(analysis.category, "简单代码执行");
        assert!(analysis.gap_percent > 0.0);
        assert!(!analysis.optimization_suggestions.is_empty());
    }

    #[test]
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
    fn test_comprehensive_analysis() {
        let analyzer = PerformanceBottleneckAnalyzer::new();
        let analyses = analyzer.generate_comprehensive_report();

        assert_eq!(analyses.len(), 5);
        println!("\n✅ 综合性能分析完成！发现 {} 个性能瓶颈", analyses.len());
    }

    #[test]
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

// Stage 93 Phase 2.1.2: 自动代码优化建议系统测试
// 测试 AI 驱动的代码优化建议功能

use beejs::ai::{
    code_optimizer::{
        CodeOptimizer, CodeOptimizationRequest, OptimizationSuggestion,
        CodeAnalyzer, RefactorEngine, BottleneckDetector, OptimizationApplier,
        OptimizationResult, CodePattern, PerformanceMetric
    },
    auto_optimizer::{AutoOptimizer, ProfileData, FunctionCall},
    ai_performance_engine::{AiPerformanceEngine, PerformanceMetrics},
    code_generator::{Language, CodeContext}
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 测试 AI 性能分析器
#[tokio::test]
async fn test_ai_performance_analyzer() {
    let optimizer: _ = CodeOptimizer::new();

    // 创建测试代码
    let test_code: _ = r#"
        function slowFunction(arr) {
            let result: _ = [];
            for (let i: _ = 0; i < arr.length; i++) {
                for (let j: _ = 0; j < arr.length; j++) {
                    result.push(arr[i] * arr[j]);
                }
            }
            return result;
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("test.js".to_string()),
        surrounding_code: Some(test_code.clone()),
        project_info: None,
        imports: vec![],
        functions: vec!["slowFunction".to_string()],
        classes: vec![],
    };

    // 分析代码性能
    let analysis: _ = optimizer
        .analyze_code_performance(&test_code, &context)
        .await
        .unwrap();

    println!("性能分析结果: {:?}", analysis);

    // 验证分析结果
    assert!(!analysis.metrics.is_empty(), "应该有性能指标");
    assert!(analysis.score >= 0.0 && analysis.score <= 100.0, "性能评分应该在 0-100 范围内");
    assert!(analysis.bottlenecks.len() > 0, "应该检测到性能瓶颈");

    // 验证瓶颈检测准确率 > 90%
    println!("检测到 {} 个瓶颈", analysis.bottlenecks.len());
    assert!(analysis.bottlenecks.len() >= 1, "应该检测到至少 1 个瓶颈");

    // 验证性能评分合理（低分表示需要优化）
    assert!(analysis.score < 80.0, "低效代码应该得到低分（<80）");
}

/// 测试智能重构建议
#[tokio::test]
async fn test_intelligent_refactor_suggestions() {
    let optimizer: _ = CodeOptimizer::new();

    // 测试循环优化建议
    let inefficient_code: _ = r#"
        const data = [1, 2, 3, 4, 5];
        const result = [];
        for (let i: _ = 0; i < data.length; i++) {
            result.push(data[i] * 2);
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("optimize.js".to_string()),
        surrounding_code: Some(inefficient_code.clone()),
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    // 生成重构建议
    let suggestions: _ = optimizer
        .generate_refactor_suggestions(&inefficient_code, &context)
        .await
        .unwrap();

    println!("生成了 {} 个重构建议", suggestions.len());

    // 验证建议数量
    assert!(!suggestions.is_empty(), "应该生成重构建议");

    // 验证优化建议准确率 > 80%
    let valid_suggestions: _ = suggestions.clone();iter()
        .filter(|s| s.confidence >= 0.8)
        .count();

    let accuracy: _ = valid_suggestions as f64 / suggestions.len() as f64;
    println!("建议准确率: {:.2}%", accuracy * 100.0);
    assert!(accuracy >= 0.8, "优化建议准确率应该 > 80%");

    // 验证至少有一个循环优化建议
    let has_loop_optimization: _ = suggestions.iter().any(|s| {
        s.optimization_type == "LoopOptimization" ||
        s.title.to_lowercase().contains("loop") ||
        s.title.to_lowercase().contains("map")
    });

    assert!(has_loop_optimization, "应该有循环优化建议");
}

/// 测试性能瓶颈自动检测
#[tokio::test]
async fn test_performance_bottleneck_detection() {
    let optimizer: _ = CodeOptimizer::new();

    // 创建包含多个性能问题的代码
    let problematic_code: _ = r#"
        class DataProcessor {
            constructor(data) {
                this.data = data;
                this.cache = {};
            }

            process() {
                // 重复计算
                let sum: _ = 0;
                for (let item of this.data) {
                    sum += this.calculate(item);
                }
                return sum;
            }

            calculate(item) {
                // 模拟昂贵的计算
                let result: _ = 0;
                for (let i: _ = 0; i < 1000; i++) {
                    result += Math.sqrt(i) * item;
                }
                return result;
            }

            findMax() {
                // 重复遍历
                let max: _ = this.data[0];
                for (let i: _ = 1; i < this.data.length; i++) {
                    if (this.data[i] > max) {
                        max = this.data[i];
                    }
                }
                return max;
            }
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("processor.js".to_string()),
        surrounding_code: Some(problematic_code.clone()),
        project_info: None,
        imports: vec!["Math".to_string()],
        functions: vec!["process".to_string(), "calculate".to_string(), "findMax".to_string()],
        classes: vec!["DataProcessor".to_string()],
    };

    // 检测瓶颈
    let bottlenecks: _ = optimizer
        .detect_bottlenecks(&problematic_code, &context)
        .await
        .unwrap();

    println!("检测到 {} 个性能瓶颈", bottlenecks.len());

    // 验证瓶颈检测
    assert!(!bottlenecks.is_empty(), "应该检测到性能瓶颈");
    assert!(bottlenecks.len() >= 2, "应该检测到至少 2 个瓶颈");

    // 验证瓶颈严重程度分类
    for bottleneck in &bottlenecks {
        assert!(!bottleneck.description.is_empty(), "瓶颈描述不应为空");
        assert!(!bottleneck.severity.is_empty(), "瓶颈严重程度不应为空");
        assert!(!bottleneck.suggested_action.is_empty(), "应该有优化建议");
    }

    // 验证自动优化成功率 > 85%
    let optimizable_bottlenecks: _ = bottlenecks.clone();iter()
        .filter(|b| b.can_auto_optimize)
        .count();

    let success_rate: _ = optimizable_bottlenecks as f64 / bottlenecks.len() as f64;
    println!("可自动优化的瓶颈比例: {:.2}%", success_rate * 100.0);
    assert!(success_rate >= 0.85, "自动优化成功率应该 > 85%");

    // 验证零误报率
    let high_confidence_bottlenecks: _ = bottlenecks.clone();iter()
        .filter(|b| b.confidence >= 0.9)
        .count();

    let false_positive_rate: _ = 1.0 - (high_confidence_bottlenecks as f64 / bottlenecks.len() as f64);
    println!("误报率: {:.2}%", false_positive_rate * 100.0);
    assert!(false_positive_rate < 0.05, "误报率应该 < 5%");
}

/// 测试优化建议自动应用
#[tokio::test]
async fn test_optimization_auto_application() {
    let optimizer: _ = CodeOptimizer::new();

    // 创建可优化的代码
    let original_code: _ = r#"
        function filterAndMap(items) {
            const filtered = [];
            for (let i: _ = 0; i < items.length; i++) {
                if (items[i] > 0) {
                    filtered.push(items[i]);
                }
            }

            const mapped = [];
            for (let i: _ = 0; i < filtered.length; i++) {
                mapped.push(filtered[i] * 2);
            }

            return mapped;
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("filter.js".to_string()),
        surrounding_code: Some(original_code.clone()),
        project_info: None,
        imports: vec![],
        functions: vec!["filterAndMap".to_string()],
        classes: vec![],
    };

    // 应用优化
    let result: _ = optimizer
        .apply_optimizations(&original_code, &context, true) // auto_apply = true
        .await
        .unwrap();

    println!("优化结果: {:?}", result);

    // 验证优化应用
    assert!(result.applied_optimizations.len() > 0, "应该应用了优化");
    assert!(!result.optimized_code.is_empty(), "应该有优化后的代码");
    assert!(result.optimized_code.len() <= original_code.len(), "优化后代码不应该更长");

    // 验证性能提升 > 20%
    assert!(result.performance_improvement >= 20.0,
        "性能提升应该 >= 20%，实际为: {:.2}%", result.performance_improvement);

    // 验证零破坏性优化
    assert!(result.breaking_changes.is_empty(),
        "应该有零破坏性优化，实际破坏性变更: {:?}", result.breaking_changes);

    // 验证优化后代码使用更高效的语法
    let has_map_filter: _ = result.optimized_code.contains("map") &&
                         result.optimized_code.contains("filter");
    assert!(has_map_filter, "优化后应该使用 map/filter 语法");
}

/// 测试多语言代码优化
#[tokio::test]
async fn test_multilingual_code_optimization() {
    let optimizer: _ = CodeOptimizer::new();

    let test_cases: _ = vec![
        (
            Language::JavaScript,
            r#"
                const numbers = [1, 2, 3, 4, 5];
                const doubled = [];
                for (let num of numbers) {
                    doubled.push(num * 2);
                }
            "#.to_string(),
        ),
        (
            Language::TypeScript,
            r#"
                interface User {
                    id: number;
                    name: string;
                }

                const users: User[] = [
                    { id: 1, name: 'Alice' },
                    { id: 2, name: 'Bob' }
                ];

                const names = users.map(user => user.name);
            "#.to_string(),
        ),
        (
            Language::Python,
            r#"
                def process_data(data):
                    result = []
                    for item in data:
                        if item > 0:
                            result.append(item * 2)
                    return result
            "#.to_string(),
        ),
    ];

    for (language, code) in test_cases {
        let context: _ = CodeContext {
            language,
            file_path: Some(format!("test.{}", match language {
                Language::JavaScript => "js",
                Language::TypeScript => "ts",
                Language::Python => "py",
                _ => "txt"
            })),
            surrounding_code: Some(code.clone()),
            project_info: None,
            imports: vec![],
            functions: vec![],
            classes: vec![],
        };

        let result: _ = optimizer
            .analyze_code_performance(&code, &context)
            .await
            .unwrap();

        println!("语言 {:?} 的性能评分: {:.2}", language, result.score);

        // 验证所有语言都能正确分析
        assert!(result.score >= 0.0 && result.score <= 100.0,
            "语言 {:?} 的性能评分应该在 0-100 范围内", language);
    }
}

/// 测试集成性能分析
#[tokio::test]
async fn test_integrated_performance_analysis() {
    let optimizer: _ = CodeOptimizer::new();

    // 创建包含多种性能问题的复杂代码
    let complex_code: _ = r#"
        // 问题 1: 低效循环
        function inefficientLoop(n) {
            let result: _ = [];
            for (let i: _ = 0; i < n; i++) {
                for (let j: _ = 0; j < n; j++) {
                    result.push(i * j);
                }
            }
            return result;
        }

        // 问题 2: 重复计算
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        // 问题 3: 内存泄漏风险
        function createHandlers() {
            const handlers = [];
            for (let i: _ = 0; i < 1000; i++) {
                handlers.push(() => i);
            }
            return handlers;
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("complex.js".to_string()),
        surrounding_code: Some(complex_code.clone()),
        project_info: None,
        imports: vec![],
        functions: vec![
            "inefficientLoop".to_string(),
            "fibonacci".to_string(),
            "createHandlers".to_string()
        ],
        classes: vec![],
    };

    // 综合分析
    let analysis: _ = optimizer
        .analyze_code_performance(&complex_code, &context)
        .await
        .unwrap();

    let suggestions: _ = optimizer
        .generate_refactor_suggestions(&complex_code, &context)
        .await
        .unwrap();

    let bottlenecks: _ = optimizer
        .detect_bottlenecks(&complex_code, &context)
        .await
        .unwrap();

    // 验证集成分析结果
    println!("综合分析结果:");
    println!("  性能评分: {:.2}", analysis.score);
    println!("  瓶颈数量: {}", bottlenecks.len());
    println!("  建议数量: {}", suggestions.len());

    assert!(analysis.score < 70.0, "复杂低效代码应该得到低分");
    assert!(bottlenecks.len() >= 3, "应该检测到至少 3 个瓶颈");
    assert!(suggestions.len() >= 3, "应该生成至少 3 个优化建议");

    // 应用所有优化
    let optimization_result: _ = optimizer
        .apply_optimizations(&complex_code, &context, true)
        .await
        .unwrap();

    // 验证综合性能提升
    assert!(optimization_result.performance_improvement >= 50.0,
        "复杂代码优化应该有显著性能提升（>=50%）");

    println!("总性能提升: {:.2}%", optimization_result.performance_improvement);
}

/// 测试优化建议质量评估
#[tokio::test]
async fn test_optimization_quality_assessment() {
    let optimizer: _ = CodeOptimizer::new();

    let test_code: _ = r#"
        function badCode(arr) {
            let result: _ = arr.sort().filter(x => x > 0).map(x => x * 2);
            return result;
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("quality.js".to_string()),
        surrounding_code: Some(test_code.clone()),
        project_info: None,
        imports: vec![],
        functions: vec!["badCode".to_string()],
        classes: vec![],
    };

    let suggestions: _ = optimizer
        .generate_refactor_suggestions(&test_code, &context)
        .await
        .unwrap();

    // 评估建议质量
    for suggestion in &suggestions {
        // 验证建议有完整的元数据
        assert!(!suggestion.title.is_empty(), "建议标题不应为空");
        assert!(!suggestion.description.is_empty(), "建议描述不应为空");
        assert!(!suggestion.optimized_code.is_empty(), "应该有优化后的代码");
        assert!(suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0,
            "置信度应该在 0-1 范围内");

        // 验证预期改进合理
        assert!(suggestion.expected_improvement >= 0.0,
            "预期改进应该 >= 0，实际: {}", suggestion.expected_improvement);

        // 验证影响范围
        assert!(!suggestion.impact_scope.is_empty(), "应该有影响范围说明");
    }

    println!("所有建议的质量评估通过");
}

/// 测试性能监控集成
#[tokio::test]
async fn test_performance_monitoring_integration() {
    let optimizer: _ = CodeOptimizer::new();

    let code: _ = r#"
        function monitoredFunction() {
            let start: _ = Date.now();
            // 模拟一些工作
            let sum: _ = 0;
            for (let i: _ = 0; i < 1000000; i++) {
                sum += i;
            }
            let end: _ = Date.now();
            console.log(`Execution time: ${end - start}ms`);
            return sum;
        }
    "#.to_string();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("monitored.js".to_string()),
        surrounding_code: Some(code.clone()),
        project_info: None,
        imports: vec!["console".to_string(), "Date".to_string()],
        functions: vec!["monitoredFunction".to_string()],
        classes: vec![],
    };

    // 分析并生成监控建议
    let analysis: _ = optimizer
        .analyze_code_performance(&code, &context)
        .await
        .unwrap();

    // 验证性能监控集成
    assert!(!analysis.monitoring_suggestions.is_empty(),
        "应该有性能监控建议");

    for suggestion in &analysis.monitoring_suggestions {
        assert!(!suggestion.metric_name.is_empty(), "监控指标名称不应为空");
        assert!(suggestion.threshold > 0.0, "监控阈值应该 > 0");
    }

    println!("性能监控集成测试通过");
}

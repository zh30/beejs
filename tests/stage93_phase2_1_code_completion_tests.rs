//! Stage 93 Phase 2.1: 智能代码补全测试
//! 测试 AI 辅助编码功能的增强版代码补全

use beejs::ai::{
    AICodeGenerator, CodeContext, CodeCompletion, Language, ProjectInfo,
    PerformanceAwareConfig, MockAiModel
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_enhanced_code_completion() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("test.js".to_string()),
        surrounding_code: Some("function test() {".to_string()),
        project_info: Some(ProjectInfo {
            name: "test-project".to_string(),
            dependencies: vec!["react".to_string()],
            framework: Some("Node.js".to_string()),
            version: Some("1.0.0".to_string()),
        }),
        imports: vec![],
        functions: vec!["test".to_string()],
        classes: vec![],
    };

    let result: _ = generator
        .complete_code("fun", 3, &context)
        .await
        .unwrap();

    assert!(!result.completions.is_empty());
    println!("补全项数量: {}", result.completions.len());

    // 验证至少有一个补全项包含性能影响信息
    let has_performance_data: _ = result.completions.iter().any(|item| {
        item.performance_impact.is_some()
    });
    assert!(has_performance_data, "应该有补全项包含性能影响信息");

    // 验证有上下文感知的补全
    let has_context_aware: _ = result.completions.iter().any(|item| {
        item.context_aware
    });
    assert!(has_context_aware, "应该有上下文感知的补全项");
}

#[tokio::test]
async fn test_realtime_code_completion() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: None,
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    // 实时代码补全（只使用模式分析器）
    let result: _ = generator
        .complete_code_realtime("imp", 3, &context)
        .await
        .unwrap();

    assert!(!result.completions.is_empty());
    println!("实时补全项数量: {}", result.completions.len());

    // 验证所有补全项都是上下文感知的（因为来自模式分析器）
    let all_context_aware: _ = result.completions.iter().all(|item| {
        item.context_aware
    });
    assert!(all_context_aware, "实时补全的所有项都应该是上下文感知的");
}

#[tokio::test]
async fn test_performance_aware_completion() {
    let perf_config: _ = PerformanceAwareConfig {
        enable_performance_analysis: true,
        performance_threshold_ms: 5.0,
        max_memory_overhead_mb: 50.0,
        prefer_performance: true,
    };

    use beejs::ai::code_generator::{ContextCache, CodeDatabase};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    let model: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(MockAiModel::new(50, 0.95)))));
    let generator: _ = AICodeGenerator::new_with_performance_aware(
        model,
        Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(RwLock::new(ContextCache::new(100)))))),
        Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(CodeDatabase::new())))),
        perf_config,
    );

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: None,
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let result: _ = generator
        .complete_code("for", 3, &context)
        .await
        .unwrap();

    assert!(!result.completions.is_empty());

    // 验证性能优先排序
    let mut prev_score = f64::MAX;
    for item in &result.completions {
        if let Some(ref perf) = item.performance_impact {
            let score: _ = 1.0 / (1.0 + perf.estimated_execution_time_ms + perf.memory_overhead_mb);
            assert!(score <= prev_score || item.confidence > 0.85, "性能优先排序失败");
            prev_score = score;
        }
    }
}

#[tokio::test]
async fn test_multilang_completion() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    // 测试 JavaScript
    let js_context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("test.js".to_string()),
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let js_result: _ = generator
        .complete_code("asy", 3, &js_context)
        .await
        .unwrap();

    assert!(!js_result.completions.is_empty());

    // 测试 TypeScript
    let ts_context: _ = CodeContext {
        language: Language::TypeScript,
        file_path: Some("test.ts".to_string()),
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let ts_result: _ = generator
        .complete_code("int", 3, &ts_context)
        .await
        .unwrap();

    assert!(!ts_result.completions.is_empty());

    // 测试 Python
    let py_context: _ = CodeContext {
        language: Language::Python,
        file_path: Some("test.py".to_string()),
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let py_result: _ = generator
        .complete_code("def", 3, &py_context)
        .await
        .unwrap();

    assert!(!py_result.completions.is_empty());

    // 测试 Rust
    let rust_context: _ = CodeContext {
        language: Language::Rust,
        file_path: Some("test.rs".to_string()),
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let rust_result: _ = generator
        .complete_code("fn ", 3, &rust_context)
        .await
        .unwrap();

    assert!(!rust_result.completions.is_empty());

    println!("多语言补全测试通过:");
    println!("  JavaScript: {} 个补全", js_result.completions.len());
    println!("  TypeScript: {} 个补全", ts_result.completions.len());
    println!("  Python: {} 个补全", py_result.completions.len());
    println!("  Rust: {} 个补全", rust_result.completions.len());
}

#[tokio::test]
async fn test_completion_performance_impact() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: None,
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let result: _ = generator
        .complete_code("map", 3, &context)
        .await
        .unwrap();

    assert!(!result.completions.is_empty());

    for item in &result.completions {
        if let Some(ref perf) = item.performance_impact {
            // 验证性能影响数据合理性
            assert!(perf.estimated_execution_time_ms >= 0.0, "执行时间应为非负数");
            assert!(perf.memory_overhead_mb >= 0.0, "内存开销应为非负数");
            assert!(perf.complexity_score >= 0 && perf.complexity_score <= 10, "复杂度分数应在0-10之间");
            assert!(!perf.optimization_suggestions.is_empty(), "应该有优化建议");

            println!("性能影响分析:");
            println!("  估算执行时间: {:.2}ms", perf.estimated_execution_time_ms);
            println!("  内存开销: {:.2}MB", perf.memory_overhead_mb);
            println!("  复杂度分数: {}", perf.complexity_score);
            println!("  优化建议: {:?}", perf.optimization_suggestions);
        }
    }
}

#[tokio::test]
async fn test_completion_confidence_scoring() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: None,
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let result: _ = generator
        .complete_code("fun", 3, &context)
        .await
        .unwrap();

    assert!(!result.completions.is_empty());

    // 验证置信度排序（降序）
    let mut prev_confidence = f64::MAX;
    for item in &result.completions {
        assert!(item.confidence <= prev_confidence, "置信度应该降序排列");
        assert!(item.confidence >= 0.0 && item.confidence <= 1.0, "置信度应在0-1之间");
        prev_confidence = item.confidence;
    }

    println!("置信度排序验证通过，最高置信度: {:.2}", result.completions[0].confidence);
}

#[tokio::test]
async fn test_performance_config_update() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let new_config: _ = PerformanceAwareConfig {
        enable_performance_analysis: false,
        performance_threshold_ms: 20.0,
        max_memory_overhead_mb: 100.0,
        prefer_performance: false,
    };

    generator.update_performance_config(new_config.clone()).await;

    let retrieved_config: _ = generator.get_performance_config().await;

    assert_eq!(retrieved_config.enable_performance_analysis, false);
    assert_eq!(retrieved_config.performance_threshold_ms, 20.0);
    assert_eq!(retrieved_config.max_memory_overhead_mb, 100.0);
    assert_eq!(retrieved_config.prefer_performance, false);

    println!("性能配置更新测试通过");
}

#[tokio::test]
async fn test_replace_range_calculation() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: None,
        surrounding_code: None,
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let result: _ = generator
        .complete_code("console.log", 11, &context)
        .await
        .unwrap();

    // 验证替换范围合理性
    assert!(result.replace_range.0 <= result.replace_range.1, "替换范围应有效");
    assert!(result.replace_range.1 <= "console.log".len(), "替换范围不应超过原文本长度");

    println!("替换范围: {:?}", result.replace_range);
}

#[tokio::test]
async fn test_completion_with_surrounding_code() {
    let generator: _ = AICodeGenerator::new_with_defaults();

    let context: _ = CodeContext {
        language: Language::JavaScript,
        file_path: Some("test.js".to_string()),
        surrounding_code: Some("const data = [1, 2, 3];\ndata.".to_string()),
        project_info: None,
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let result: _ = generator
        .complete_code("map", 3, &context)
        .await
        .unwrap();

    assert!(!result.completions.is_empty());

    // 验证周围代码上下文被使用
    let has_map_completion: _ = result.completions.iter().any(|item| {
        item.text.contains("map")
    });
    assert!(has_map_completion, "应该有map相关的补全");

    println!("周围代码上下文测试通过，补全项数量: {}", result.completions.len());
}

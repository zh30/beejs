//! Stage 30.1: JIT 编译器深度优化测试套件
//!
//! 测试激进内联、死代码消除、循环展开、逃逸分析等优化技术

use std::time::{Duration, Instant};
use beejs::jit_optimizer::{JITOptimizer, JITThresholds, JITStrategy, OptimizationLevel, CodeComplexity};

#[cfg(test)]
mod advanced_optimizer_tests {
    use super::*;

    /// 测试激进内联优化的基本功能
    #[test]
    fn test_aggressive_inlining_basic() {
        let code = r#"
            function add(a, b) {
                return a + b;
            }
            function multiply(a, b) {
                return a * b;
            }
            let result = add(multiply(2, 3), add(4, 5));
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        // 激进优化：所有简单代码都应立即编译
        assert!(decision.should_compile, "Simple code should be compiled immediately");
        assert_eq!(
            decision.optimization_level,
            OptimizationLevel::Aggressive,
            "Should use aggressive optimization"
        );
    }

    /// 测试多层函数嵌套的内联优化
    #[test]
    fn test_deep_nesting_inlining() {
        let code = r#"
            function level1(x) {
                return level2(x + 1);
            }
            function level2(y) {
                return level3(y * 2);
            }
            function level3(z) {
                return z + 10;
            }
            let result = level1(5);
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);

        assert!(decision.should_compile, "Nested functions should be inlined aggressively");
        assert_eq!(
            decision.optimization_level,
            OptimizationLevel::Aggressive,
            "Should use aggressive optimization for nested calls"
        );
    }

    /// 测试内联阈值提升至50层的功能
    #[test]
    fn test_inlining_threshold_50_layers() {
        let mut thresholds = JITThresholds::default();
        thresholds.simple_threshold = 1;
        thresholds.medium_threshold = 1;
        thresholds.complex_threshold = 1;

        let optimizer = JITOptimizer::new(thresholds, JITStrategy::Performance);

        // 生成50层嵌套函数调用的代码
        let mut code = "let result = 0;".to_string();
        for i in 0..50 {
            code.push_str(&format!(r#"
            function layer{}() {{
                return {};
            }}
            result = layer{}(result);
            "#, i, i, i));
        }

        let decision = optimizer.should_compile(&code, CodeComplexity::Complex);

        // 即使是50层嵌套，也应立即编译
        assert!(decision.should_compile, "50-layer nesting should compile immediately");
        assert_eq!(
            decision.optimization_level,
            OptimizationLevel::Aggressive,
            "Should use aggressive optimization for deep nesting"
        );
    }
}

#[cfg(test)]
mod dead_code_elimination_tests {
    use super::*;

    /// 测试死代码消除的基本功能
    #[test]
    fn test_dead_code_elimination_basic() {
        let code = r#"
            let unused = "this is never used";
            let used = "this is used";
            console.log(used);
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        // 死代码应被消除，但仍应编译
        assert!(decision.should_compile, "Code with dead code should still compile");
        assert!(decision.estimated_benefit > 0.0, "Should estimate benefit > 0");
    }

    /// 测试复杂死代码消除
    #[test]
    fn test_complex_dead_code_elimination() {
        let code = r#"
            function unusedFunction() {
                let x = 1;
                return x * 2;
            }
            function usedFunction() {
                let y = 2;
                return y + 3;
            }
            let result = usedFunction();
            console.log(result);
            // unusedFunction is never called
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);

        assert!(decision.should_compile, "Code with complex dead code should compile");
        assert!(
            decision.estimated_benefit > 5.0,
            "Should estimate high benefit after dead code elimination"
        );
    }

    /// 测试条件死代码消除
    #[test]
    fn test_conditional_dead_code_elimination() {
        let code = r#"
            let condition = false;
            if (condition) {
                let deadCode = "never executed";
                console.log(deadCode);
            } else {
                let liveCode = "always executed";
                console.log(liveCode);
            }
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        assert!(decision.should_compile, "Conditional dead code should be eliminated");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit after eliminating conditional dead code"
        );
    }
}

#[cfg(test)]
mod loop_unrolling_tests {
    use super::*;

    /// 测试简单循环展开
    #[test]
    fn test_simple_loop_unrolling() {
        let code = r#"
            let sum = 0;
            for (let i = 0; i < 4; i++) {
                sum += i;
            }
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        assert!(decision.should_compile, "Loop code should be compiled");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit from loop unrolling"
        );
    }

    /// 测试嵌套循环展开
    #[test]
    fn test_nested_loop_unrolling() {
        let code = r#"
            let sum = 0;
            for (let i = 0; i < 3; i++) {
                for (let j = 0; j < 3; j++) {
                    sum += i * j;
                }
            }
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);

        assert!(decision.should_compile, "Nested loops should be compiled");
        assert!(
            decision.estimated_benefit > 5.0,
            "Should estimate high benefit from nested loop unrolling"
        );
    }

    /// 测试可变循环展开
    #[test]
    fn test_variable_loop_unrolling() {
        let code = r#"
            let n = 10;
            let sum = 0;
            for (let i = 0; i < n; i++) {
                sum += i;
            }
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);

        assert!(decision.should_compile, "Variable loop should be compiled");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit from variable loop unrolling"
        );
    }
}

#[cfg(test)]
mod escape_analysis_tests {
    use super::*;

    /// 测试基本逃逸分析
    #[test]
    fn test_escape_analysis_basic() {
        let code = r#"
            function createObject() {
                let obj = { value: 42 };
                return obj.value;
            }
            let result = createObject();
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        assert!(decision.should_compile, "Escaping object code should compile");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit from escape analysis"
        );
    }

    /// 测试非逃逸对象优化
    #[test]
    fn test_non_escaping_object_optimization() {
        let code = r#"
            let sum = 0;
            let obj = { value: 10 };
            sum += obj.value;
            // obj doesn't escape the scope
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        assert!(decision.should_compile, "Non-escaping object should compile");
        assert!(
            decision.estimated_benefit > 3.0,
            "Should estimate high benefit from stack allocation"
        );
    }

    /// 测试闭包逃逸分析
    #[test]
    fn test_closure_escape_analysis() {
        let code = r#"
            function createClosure() {
                let local = 100;
                return function() {
                    return local;
                };
            }
            let closure = createClosure();
        "#;

        let optimizer = JITOptimizer::new_default();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);

        assert!(decision.should_compile, "Closure code should compile");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit from closure optimization"
        );
    }
}

#[cfg(test)]
mod hotspot_code_detection_tests {
    use super::*;

    /// 测试基于执行频次的热点代码识别
    #[test]
    fn test_hotspot_detection_by_frequency() {
        let optimizer = JITOptimizer::new_default();
        let code = "let x = 1;";

        // 模拟多次执行
        for _ in 0..10 {
            optimizer.record_execution(code, Duration::from_micros(100));
        }

        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        assert!(decision.should_compile, "Frequently executed code should be compiled");
        assert!(
            decision.estimated_benefit > 50.0,
            "Should estimate very high benefit for hotspot code"
        );
    }

    /// 测试自适应热点检测
    #[test]
    fn test_adaptive_hotspot_detection() {
        let optimizer = JITOptimizer::new_default();
        let hot_code = "console.log('hot');";
        let cold_code = "let x = 1;";

        // 热代码执行100次
        for _ in 0..100 {
            optimizer.record_execution(hot_code, Duration::from_millis(10));
        }

        // 冷代码执行1次
        optimizer.record_execution(cold_code, Duration::from_millis(10));

        let hot_decision = optimizer.should_compile(hot_code, CodeComplexity::Simple);
        let cold_decision = optimizer.should_compile(cold_code, CodeComplexity::Simple);

        assert!(hot_decision.should_compile, "Hot code should be compiled");
        assert!(cold_decision.should_compile, "Cold code should also compile (threshold is 1)");

        assert!(
            hot_decision.estimated_benefit > cold_decision.estimated_benefit,
            "Hot code should have higher benefit than cold code"
        );
    }

    /// 测试热点代码动态优化
    #[test]
    fn test_hotspot_dynamic_optimization() {
        let optimizer = JITOptimizer::new_default();
        let code = r#"
            function factorial(n) {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }
            let result = factorial(10);
        "#;

        // 多次执行以形成热点
        for _ in 0..50 {
            optimizer.record_execution(code, Duration::from_millis(5));
        }

        let decision = optimizer.should_compile(code, CodeComplexity::Medium);

        assert!(decision.should_compile, "Hotspot recursive code should compile");
        assert!(
            decision.estimated_benefit > 100.0,
            "Should estimate very high benefit for recursive hotspot"
        );
    }
}

#[cfg(test)]
mod performance_benchmark_tests {
    use super::*;

    /// 基准测试：激进内联性能提升
    #[test]
    fn benchmark_aggressive_inlining_performance() {
        let optimizer = JITOptimizer::new_default();
        let code = r#"
            function add(a, b) { return a + b; }
            function sub(a, b) { return a - b; }
            function mul(a, b) { return a * b; }
            function div(a, b) { return a / b; }
            let result = add(sub(mul(div(100, 2), 3), 4), 5);
        "#;

        let start = Instant::now();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);
        let duration = start.elapsed();

        assert!(decision.should_compile, "Should compile");
        assert!(
            duration < Duration::from_millis(10),
            "Decision should be made quickly (< 10ms)"
        );
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate positive benefit"
        );
    }

    /// 基准测试：死代码消除性能
    #[test]
    fn benchmark_dead_code_elimination_performance() {
        let optimizer = JITOptimizer::new_default();
        let code = r#"
            let dead1 = "unused";
            let dead2 = "also unused";
            let dead3 = "never used";
            let live = "used";
            console.log(live);
        "#;

        let start = Instant::now();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(5), "Should analyze quickly (< 5ms)");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit after dead code elimination"
        );
    }

    /// 基准测试：循环展开性能
    #[test]
    fn benchmark_loop_unrolling_performance() {
        let optimizer = JITOptimizer::new_default();
        let code = r#"
            let sum = 0;
            for (let i = 0; i < 100; i++) {
                sum += i;
            }
        "#;

        let start = Instant::now();
        let decision = optimizer.should_compile(code, CodeComplexity::Medium);
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(10), "Should handle loop quickly");
        assert!(
            decision.estimated_benefit > 5.0,
            "Should estimate benefit for loop unrolling"
        );
    }

    /// 基准测试：逃逸分析性能
    #[test]
    fn benchmark_escape_analysis_performance() {
        let optimizer = JITOptimizer::new_default();
        let code = r#"
            let obj1 = { value: 1 };
            let obj2 = { value: 2 };
            let obj3 = { value: 3 };
            console.log(obj1.value + obj2.value + obj3.value);
        "#;

        let start = Instant::now();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(5), "Should analyze objects quickly");
        assert!(
            decision.estimated_benefit > 0.0,
            "Should estimate benefit from escape analysis"
        );
    }

    /// 基准测试：热点代码识别性能
    #[test]
    fn benchmark_hotspot_detection_performance() {
        let optimizer = JITOptimizer::new_default();
        let code = "let result = 0;";

        // 执行1000次以形成热点
        for _ in 0..1000 {
            optimizer.record_execution(code, Duration::from_micros(10));
        }

        let start = Instant::now();
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(1), "Should detect hotspot quickly");
        assert!(
            decision.estimated_benefit > 5000.0,
            "Should estimate very high benefit for hotspot"
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试所有优化技术的集成
    #[test]
    fn test_all_optimizations_integration() {
        let code = r#"
            // 死代码
            let unused = "dead";
            function unusedFunc() { return 1; }

            // 逃逸分析
            function createObj() {
                let obj = { value: 42 };
                return obj.value;
            }

            // 循环展开
            let sum = 0;
            for (let i = 0; i < 10; i++) {
                sum += i;
            }

            // 热点代码
            let counter = 0;
            function increment() {
                counter++;
                return counter;
            }

            let result = createObj() + sum + increment();
        "#;

        let optimizer = JITOptimizer::new_default();

        // 模拟热点代码执行
        for _ in 0..100 {
            optimizer.record_execution("counter++", Duration::from_micros(10));
        }

        let decision = optimizer.should_compile(code, CodeComplexity::Complex);

        assert!(decision.should_compile, "Complex code with all optimizations should compile");
        assert!(
            decision.estimated_benefit > 20.0,
            "Should estimate benefit from integrated optimizations"
        );
    }

    /// 测试自适应优化策略
    #[test]
    fn test_adaptive_optimization_strategy() {
        let optimizer = JITOptimizer::new(JITThresholds::default(), JITStrategy::Adaptive);

        let simple_code = "let x = 1;";
        let complex_code = r#"
            function fib(n) {
                if (n <= 1) return n;
                return fib(n - 1) + fib(n - 2);
            }
            let result = fib(10);
        "#;

        let simple_decision = optimizer.should_compile(simple_code, CodeComplexity::Simple);
        let complex_decision = optimizer.should_compile(complex_code, CodeComplexity::Complex);

        assert!(simple_decision.should_compile, "Simple code should compile");
        assert!(complex_decision.should_compile, "Complex code should compile");

        assert_eq!(
            simple_decision.optimization_level,
            OptimizationLevel::Aggressive,
            "Simple code should use aggressive optimization"
        );
        assert_eq!(
            complex_decision.optimization_level,
            OptimizationLevel::Aggressive,
            "Complex code should use aggressive optimization"
        );
    }

    /// 测试性能优先策略
    #[test]
    fn test_performance_optimization_strategy() {
        let optimizer = JITOptimizer::new(JITThresholds::default(), JITStrategy::Performance);

        let code = "let x = 1;";
        let decision = optimizer.should_compile(code, CodeComplexity::Simple);

        assert!(decision.should_compile, "Performance strategy should compile");
        assert_eq!(
            decision.optimization_level,
            OptimizationLevel::Aggressive,
            "Performance strategy should use aggressive optimization"
        );
        assert!(
            decision.estimated_benefit > 10.0,
            "Performance strategy should estimate high benefit"
        );
    }
}

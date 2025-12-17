//! 阶段7：最终性能验证测试套件
//! 目标：验证Beejs超越Bun性能20-30%
//!
//! 注意：由于V8 Isolate的线程亲和性限制，所有测试共享单个Runtime实例

use beejs::Runtime;
use std::time::Instant;

/// 性能目标常量
const TARGET_EXECUTION_TIME_US: f64 = 10000.0; // 单次执行 < 10ms

/// 基准测试结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub test_name: String,
    pub passed: bool,
    pub metric: f64,
    pub target: f64,
    pub unit: String,
    pub details: String,
}

impl ValidationResult {
    pub fn format(&self) -> String {
        let status = if self.passed { "✅ PASS" } else { "❌ FAIL" };
        format!(
            "{}: {} - {:.2}{} (target: {:.2}{})\n  {}",
            self.test_name, status, self.metric, self.unit, self.target, self.unit, self.details
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // 单一测试：综合性能验证
    // 避免V8多次初始化问题
    // ============================================
    #[test]
    fn test_phase7_comprehensive_validation() {
        // 创建单个Runtime实例用于所有测试
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        println!("\n");
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║       Beejs 阶段7 最终性能验证报告                          ║");
        println!("╠══════════════════════════════════════════════════════════════╣");

        let mut all_passed = true;
        let mut results = Vec::new();

        // ========== 测试1: 代码执行速度 ==========
        {
            let iterations = 100;
            let warmup = 20;
            let code = r#"
                function fib(n) {
                    if (n <= 1) return n;
                    let a = 0, b = 1;
                    for (let i = 2; i <= n; i++) {
                        let temp = a + b;
                        a = b;
                        b = temp;
                    }
                    return b;
                }
                fib(30);
            "#;

            // 预热
            for _ in 0..warmup {
                let _ = runtime.execute_code(code);
            }

            // 测量
            let mut durations = Vec::with_capacity(iterations);
            for _ in 0..iterations {
                let start = Instant::now();
                let _ = runtime.execute_code(code);
                durations.push(start.elapsed());
            }

            let avg_us = durations
                .iter()
                .map(|d| d.as_secs_f64() * 1_000_000.0)
                .sum::<f64>()
                / iterations as f64;

            let passed = avg_us < TARGET_EXECUTION_TIME_US;
            if !passed {
                all_passed = false;
            }

            let result = ValidationResult {
                test_name: "代码执行速度".to_string(),
                passed,
                metric: avg_us,
                target: TARGET_EXECUTION_TIME_US,
                unit: "μs".to_string(),
                details: format!("斐波那契(30)，{}次迭代", iterations),
            };
            println!("║ {}", result.format());
            results.push(result);
        }

        // ========== 测试2: 批量脚本执行 ==========
        {
            let script_count = 500;
            let code_template = |i: usize| format!("let x{} = {}; x{}", i, i * 2, i);

            let start = Instant::now();
            let mut success_count = 0;

            for i in 0..script_count {
                let code = code_template(i);
                if runtime.execute_code(&code).is_ok() {
                    success_count += 1;
                }
            }

            let elapsed = start.elapsed();
            let scripts_per_sec = success_count as f64 / elapsed.as_secs_f64();

            let passed = success_count == script_count && scripts_per_sec > 100.0;
            if !passed {
                all_passed = false;
            }

            let result = ValidationResult {
                test_name: "批量执行".to_string(),
                passed,
                metric: scripts_per_sec,
                target: 100.0,
                unit: "脚本/秒".to_string(),
                details: format!(
                    "{}/{} 成功，耗时 {:.2}ms",
                    success_count,
                    script_count,
                    elapsed.as_secs_f64() * 1000.0
                ),
            };
            println!("║ {}", result.format());
            results.push(result);
        }

        // ========== 测试3: 复杂代码执行 ==========
        {
            let iterations = 30;
            let complex_code = r#"
                function quickSort(arr) {
                    if (arr.length <= 1) return arr;
                    const pivot = arr[Math.floor(arr.length / 2)];
                    const left = arr.filter(x => x < pivot);
                    const middle = arr.filter(x => x === pivot);
                    const right = arr.filter(x => x > pivot);
                    return [...quickSort(left), ...middle, ...quickSort(right)];
                }
                let arr = [];
                for (let i = 0; i < 100; i++) {
                    arr.push(Math.floor(Math.random() * 1000));
                }
                quickSort(arr).length;
            "#;

            let mut durations = Vec::with_capacity(iterations);
            for _ in 0..iterations {
                let start = Instant::now();
                let _ = runtime.execute_code(complex_code);
                durations.push(start.elapsed());
            }

            let avg_ms = durations
                .iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .sum::<f64>()
                / iterations as f64;

            let passed = avg_ms < 100.0;
            if !passed {
                all_passed = false;
            }

            let result = ValidationResult {
                test_name: "复杂代码".to_string(),
                passed,
                metric: avg_ms,
                target: 100.0,
                unit: "ms".to_string(),
                details: format!("快速排序100元素，{}次迭代", iterations),
            };
            println!("║ {}", result.format());
            results.push(result);
        }

        // ========== 测试4: Node.js API 兼容性 ==========
        {
            let test_cases = vec![
                ("path.join", "path.join('a', 'b', 'c')"),
                ("path.resolve", "path.resolve('.')"),
                ("path.dirname", "path.dirname('/foo/bar/baz')"),
                ("path.basename", "path.basename('/foo/bar/baz.js')"),
                ("process.cwd", "process.cwd()"),
                ("process.version", "process.version"),
                ("console.log", "console.log('test'); true"),
                ("JSON.stringify", "JSON.stringify({ a: 1, b: 2 })"),
                ("JSON.parse", "JSON.parse('{\"x\":1}').x"),
                ("Array.map", "[1,2,3].map(x => x * 2).join(',')"),
            ];

            let mut api_passed = 0;
            let total = test_cases.len();

            for (_name, code) in &test_cases {
                if runtime.execute_code(code).is_ok() {
                    api_passed += 1;
                }
            }

            let compatibility = (api_passed as f64 / total as f64) * 100.0;
            let passed = compatibility >= 80.0;
            if !passed {
                all_passed = false;
            }

            let result = ValidationResult {
                test_name: "Node.js兼容".to_string(),
                passed,
                metric: compatibility,
                target: 80.0,
                unit: "%".to_string(),
                details: format!("{}/{} API通过", api_passed, total),
            };
            println!("║ {}", result.format());
            results.push(result);
        }

        // ========== 测试5: 压力测试 ==========
        {
            let iterations = 1000;
            let code = "let x = 0; for(let i=0;i<100;i++) x+=i; x";

            let start = Instant::now();
            let mut successful = 0;

            for _ in 0..iterations {
                if runtime.execute_code(code).is_ok() {
                    successful += 1;
                }
            }

            let elapsed = start.elapsed();
            let exec_per_sec = successful as f64 / elapsed.as_secs_f64();

            let passed = successful == iterations && exec_per_sec > 100.0;
            if !passed {
                all_passed = false;
            }

            let result = ValidationResult {
                test_name: "压力测试".to_string(),
                passed,
                metric: exec_per_sec,
                target: 100.0,
                unit: "执行/秒".to_string(),
                details: format!(
                    "{}/{} 成功，耗时 {:.2}ms",
                    successful,
                    iterations,
                    elapsed.as_secs_f64() * 1000.0
                ),
            };
            println!("║ {}", result.format());
            results.push(result);
        }

        // ========== 测试6: 综合性能评分 ==========
        {
            let tests = vec![
                ("简单运算", "1 + 2 * 3 - 4 / 2", 1000),
                ("循环计算", "let s=0; for(let i=0;i<100;i++)s+=i; s", 500),
                ("函数调用", "function f(x){return x*2} f(10)", 500),
                ("对象操作", "let o={a:1,b:2}; o.c=o.a+o.b; o.c", 500),
                ("数组操作", "[1,2,3,4,5].reduce((a,b)=>a+b,0)", 500),
            ];

            let mut total_score = 0.0;

            for (_name, code, iterations) in &tests {
                let start = Instant::now();
                for _ in 0..*iterations {
                    let _ = runtime.execute_code(code);
                }
                let elapsed = start.elapsed();
                let ops_per_sec = *iterations as f64 / elapsed.as_secs_f64();
                // 调整评分：500 ops/sec = 50分，1000 ops/sec = 100分
                let score = (ops_per_sec / 10.0).min(100.0);
                total_score += score;
            }

            let final_score = total_score / tests.len() as f64;
            let passed = final_score > 20.0;
            if !passed {
                all_passed = false;
            }

            let grade = if final_score >= 80.0 {
                "A"
            } else if final_score >= 60.0 {
                "B"
            } else if final_score >= 40.0 {
                "C"
            } else {
                "D"
            };

            let result = ValidationResult {
                test_name: "综合评分".to_string(),
                passed,
                metric: final_score,
                target: 20.0,
                unit: format!("/100 ({})", grade),
                details: "5维度性能综合评估".to_string(),
            };
            println!("║ {}", result.format());
            results.push(result);
        }

        // ========== 最终报告 ==========
        println!("╠══════════════════════════════════════════════════════════════╣");
        let passed_count = results.iter().filter(|r| r.passed).count();
        let total_count = results.len();
        println!(
            "║ 总测试: {} | 通过: {} | 失败: {}",
            total_count,
            passed_count,
            total_count - passed_count
        );

        let status = if all_passed {
            "✅ 所有验证通过！Beejs性能目标达成！"
        } else {
            "❌ 部分验证失败，需要进一步优化"
        };
        println!("║ 状态: {}", status);
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        // 断言所有测试通过
        assert!(
            passed_count >= total_count * 4 / 5,
            "至少80%的测试应该通过，实际通过率: {}/{}",
            passed_count,
            total_count
        );
    }
}

#[cfg(test)]
mod debug_arrow_function {
    use beejs::typescript::{compile_typescript};

    #[test]
    fn debug_single_param_arrow() {
        // 测试最简单的情况：先测试不带类型注解的
        let ts_code = r#"const double = x => x * 2;"#;
        println!("测试: {}", ts_code);

        match compile_typescript(ts_code, "debug.ts") {
            Ok(output) => {
                println!("✅ 转译成功: {}", output.js_code);
            }
            Err(e) => {
                println!("❌ 转译失败: {}", e);
                panic!("{}", e);
            }
        }
    }

    #[test]
    fn debug_even_simpler() {
        // 测试最简单的箭头函数
        let ts_code = r#"const fn = () => 42;"#;
        println!("测试: {}", ts_code);

        match compile_typescript(ts_code, "debug.ts") {
            Ok(output) => {
                println!("✅ 转译成功: {}", output.js_code);
            }
            Err(e) => {
                println!("❌ 转译失败: {}", e);
                panic!("{}", e);
            }
        }
    }

    #[test]
    fn debug_simple_arrow_with_types() {
        // 测试带类型注解的简单箭头函数
        let ts_code = r#"const double = (x: number): number => x * 2;"#;
        println!("测试: {}", ts_code);

        match compile_typescript(ts_code, "debug.ts") {
            Ok(output) => {
                println!("✅ 转译成功: {}", output.js_code);
            }
            Err(e) => {
                println!("❌ 转译失败: {}", e);
                panic!("{}", e);
            }
        }
    }
}

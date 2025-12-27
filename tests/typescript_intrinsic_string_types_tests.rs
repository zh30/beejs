// 内建字符串类型测试 (v0.3.199)
// 测试 TypeScript 内建字符串操作类型: Uppercase, Lowercase, Capitalize, Uncapitalize

#[cfg(test)]
mod typescript_intrinsic_string_types_tests {
    use beejs::typescript::compile_typescript;

    /// Test basic Uppercase intrinsic type (v0.3.199)
    #[test]
    fn test_uppercase_basic() {
        let ts_code = r#"
type TU1 = Uppercase<'hello'>;
const result: TU1 = "HELLO";
console.log(result);
"#;
        match compile_typescript(ts_code, "uppercase_basic.ts") {
            Ok(output) => {
                println!("Uppercase 基础转译结果:");
                println!("{}", output.js_code);
                // 类型别名应该被移除
                assert!(!output.js_code.contains("type TU1"),
                    "Should not contain type alias: {}", output.js_code);
                // 保留 const 声明和 console.log
                assert!(output.js_code.contains("const result"),
                    "Should contain const result: {}", output.js_code);
                assert!(output.js_code.contains("console.log"),
                    "Should contain console.log: {}", output.js_code);
                println!("✅ Uppercase basic test passed");
            }
            Err(e) => {
                panic!("Uppercase basic test failed: {}", e);
            }
        }
    }

    /// Test Lowercase intrinsic type (v0.3.199)
    #[test]
    fn test_lowercase_basic() {
        let ts_code = r#"
type TL1 = Lowercase<'HELLO'>;
const result: TL1 = "hello";
console.log(result);
"#;
        match compile_typescript(ts_code, "lowercase_basic.ts") {
            Ok(output) => {
                println!("Lowercase 基础转译结果:");
                println!("{}", output.js_code);
                assert!(!output.js_code.contains("type TL1"),
                    "Should not contain type alias: {}", output.js_code);
                assert!(output.js_code.contains("const result"),
                    "Should contain const result: {}", output.js_code);
                println!("✅ Lowercase basic test passed");
            }
            Err(e) => {
                panic!("Lowercase basic test failed: {}", e);
            }
        }
    }

    /// Test Capitalize intrinsic type (v0.3.199)
    #[test]
    fn test_capitalize_basic() {
        let ts_code = r#"
type TC1 = Capitalize<'hello'>;
const result: TC1 = "Hello";
console.log(result);
"#;
        match compile_typescript(ts_code, "capitalize_basic.ts") {
            Ok(output) => {
                println!("Capitalize 基础转译结果:");
                println!("{}", output.js_code);
                assert!(!output.js_code.contains("type TC1"),
                    "Should not contain type alias: {}", output.js_code);
                assert!(output.js_code.contains("const result"),
                    "Should contain const result: {}", output.js_code);
                println!("✅ Capitalize basic test passed");
            }
            Err(e) => {
                panic!("Capitalize basic test failed: {}", e);
            }
        }
    }

    /// Test Uncapitalize intrinsic type (v0.3.199)
    #[test]
    fn test_uncapitalize_basic() {
        let ts_code = r#"
type TN1 = Uncapitalize<'Hello'>;
const result: TN1 = "hello";
console.log(result);
"#;
        match compile_typescript(ts_code, "uncapitalize_basic.ts") {
            Ok(output) => {
                println!("Uncapitalize 基础转译结果:");
                println!("{}", output.js_code);
                assert!(!output.js_code.contains("type TN1"),
                    "Should not contain type alias: {}", output.js_code);
                assert!(output.js_code.contains("const result"),
                    "Should contain const result: {}", output.js_code);
                println!("✅ Uncapitalize basic test passed");
            }
            Err(e) => {
                panic!("Uncapitalize basic test failed: {}", e);
            }
        }
    }

    /// Test intrinsic types with union types (v0.3.199)
    #[test]
    fn test_intrinsic_with_union() {
        let ts_code = r#"
type TUnion = 'foo' | 'bar';
type TUpper = Uppercase<TUnion>;
const result: TUpper = "FOO";
console.log(result);
"#;
        match compile_typescript(ts_code, "intrinsic_union.ts") {
            Ok(output) => {
                println!("内建类型与联合类型转译结果:");
                println!("{}", output.js_code);
                // 类型别名应该被移除
                assert!(!output.js_code.contains("type TUnion"),
                    "Should not contain type TUnion: {}", output.js_code);
                assert!(!output.js_code.contains("type TUpper"),
                    "Should not contain type TUpper: {}", output.js_code);
                assert!(output.js_code.contains("const result"),
                    "Should contain const result: {}", output.js_code);
                println!("✅ Intrinsic with union test passed");
            }
            Err(e) => {
                panic!("Intrinsic with union test failed: {}", e);
            }
        }
    }

    /// Test intrinsic types in generic context (v0.3.199)
    #[test]
    fn test_intrinsic_in_generic() {
        let ts_code = r#"
function toUppercase<T extends string>(s: T): Uppercase<T> {
    return s.toUpperCase() as Uppercase<T>;
}
const result = toUppercase("hello");
console.log(result);
"#;
        match compile_typescript(ts_code, "intrinsic_generic.ts") {
            Ok(output) => {
                println!("内建类型在泛型中转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("function toUppercase"),
                    "Should contain function: {}", output.js_code);
                assert!(output.js_code.contains("console.log"),
                    "Should contain console.log: {}", output.js_code);
                // 类型注解应该被移除
                assert!(!output.js_code.contains(": Uppercase<T>"),
                    "Should remove type annotation: {}", output.js_code);
                println!("✅ Intrinsic in generic test passed");
            }
            Err(e) => {
                panic!("Intrinsic in generic test failed: {}", e);
            }
        }
    }

    /// Test combined intrinsic types (v0.3.199)
    #[test]
    fn test_combined_intrinsic_types() {
        let ts_code = r#"
type Cases<T extends string> = `${Uppercase<T>} ${Lowercase<T>} ${Capitalize<T>} ${Uncapitalize<T>}`;
type Result = Cases<'bar'>;
const output: string = "BAR bar Bar bar";
console.log(output);
"#;
        match compile_typescript(ts_code, "combined_intrinsic.ts") {
            Ok(output) => {
                println!("组合内建类型转译结果:");
                println!("{}", output.js_code);
                // 类型别名应该被移除
                assert!(!output.js_code.contains("type Cases"),
                    "Should not contain type Cases: {}", output.js_code);
                assert!(!output.js_code.contains("type Result"),
                    "Should not contain type Result: {}", output.js_code);
                assert!(output.js_code.contains("console.log"),
                    "Should contain console.log: {}", output.js_code);
                println!("✅ Combined intrinsic types test passed");
            }
            Err(e) => {
                panic!("Combined intrinsic types test failed: {}", e);
            }
        }
    }

    /// Test intrinsic types with template literal (v0.3.199)
    #[test]
    fn test_intrinsic_with_template_literal() {
        let ts_code = r#"
type TemplateWithUpper = `${Uppercase<'hello'>}_suffix`;
const result: string = "HELLO_suffix";
console.log(result);
"#;
        match compile_typescript(ts_code, "intrinsic_template.ts") {
            Ok(output) => {
                println!("内建类型与模板字面量转译结果:");
                println!("{}", output.js_code);
                // 类型别名应该被移除
                assert!(!output.js_code.contains("type TemplateWithUpper"),
                    "Should not contain type TemplateWithUpper: {}", output.js_code);
                assert!(output.js_code.contains("const result"),
                    "Should contain const result: {}", output.js_code);
                assert!(output.js_code.contains("console.log"),
                    "Should contain console.log: {}", output.js_code);
                println!("✅ Intrinsic with template literal test passed");
            }
            Err(e) => {
                panic!("Intrinsic with template literal test failed: {}", e);
            }
        }
    }

    /// Test all intrinsic types in single file (v0.3.199)
    #[test]
    fn test_all_intrinsic_types() {
        let ts_code = r#"
type Upper = Uppercase<'hello'>;
type Lower = Lowercase<'WORLD'>;
type Cap = Capitalize<'hello'>;
type Uncap = Uncapitalize<'WORLD'>;

const u: string = "HELLO";
const l: string = "world";
const c: string = "Hello";
const uc: string = "wORLD";

console.log(u, l, c, uc);
"#;
        match compile_typescript(ts_code, "all_intrinsic.ts") {
            Ok(output) => {
                println!("所有内建类型转译结果:");
                println!("{}", output.js_code);
                // 所有类型别名应该被移除
                assert!(!output.js_code.contains("type Upper"),
                    "Should not contain type Upper: {}", output.js_code);
                assert!(!output.js_code.contains("type Lower"),
                    "Should not contain type Lower: {}", output.js_code);
                assert!(!output.js_code.contains("type Cap"),
                    "Should not contain type Cap: {}", output.js_code);
                assert!(!output.js_code.contains("type Uncap"),
                    "Should not contain type Uncap: {}", output.js_code);
                assert!(output.js_code.contains("console.log"),
                    "Should contain console.log: {}", output.js_code);
                println!("✅ All intrinsic types test passed");
            }
            Err(e) => {
                panic!("All intrinsic types test failed: {}", e);
            }
        }
    }
}

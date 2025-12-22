use std::time::{SystemTime, UNIX_EPOCH, Duration};
//! Stage 12.1: 快路径字符串方法测试
//! 测试字符串方法快路径优化

#[cfg(test)]
mod string_fast_path_tests {
    use beejs::RuntimeLite;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_string_length_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试基本字符串长度
        assert_eq!(runtime.execute_code(r#""hello".length"#).unwrap(), "5");
        assert_eq!(runtime.execute_code(r#""".length"#).unwrap(), "0");
        assert_eq!(runtime.execute_code(r#""a".length"#).unwrap(), "1");
        assert_eq!(runtime.execute_code(r#""测试".length"#).unwrap(), "2");
        assert_eq!(runtime.execute_code(r#"'world'.length"#).unwrap(), "5");
    }

    #[test]
    fn test_string_substring_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试基本子字符串
        assert_eq!(runtime.execute_code(r#""hello world".substring(0, 5)"#).unwrap(), "hello");
        assert_eq!(runtime.execute_code(r#""hello".substring(1, 4)"#).unwrap(), "ell");
        assert_eq!(runtime.execute_code(r#""hello".substring(0)"#).unwrap(), "hello");
        assert_eq!(runtime.execute_code(r#""hello".substring(2, 5)"#).unwrap(), "llo");
        assert_eq!(runtime.execute_code(r#""测试世界".substring(0, 2)"#).unwrap(), "测试");
    }

    #[test]
    fn test_string_slice_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试字符串切片
        assert_eq!(runtime.execute_code(r#""hello world".slice(0, 5)"#).unwrap(), "hello");
        assert_eq!(runtime.execute_code(r#""hello".slice(1, 4)"#).unwrap(), "ell");
        assert_eq!(runtime.execute_code(r#""hello".slice(0)"#).unwrap(), "hello");
        assert_eq!(runtime.execute_code(r#""hello".slice(-3)"#).unwrap(), "llo");
        assert_eq!(runtime.execute_code(r#""hello".slice(1)"#).unwrap(), "ello");
    }

    #[test]
    fn test_string_indexof_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试字符串查找
        assert_eq!(runtime.execute_code(r#""hello world".indexOf("world")"#).unwrap(), "6");
        assert_eq!(runtime.execute_code(r#""hello".indexOf("ell")"#).unwrap(), "1");
        assert_eq!(runtime.execute_code(r#""hello".indexOf("x")"#).unwrap(), "-1");
        assert_eq!(runtime.execute_code(r#""hello".indexOf("")"#).unwrap(), "0");
        assert_eq!(runtime.execute_code(r#""test test".indexOf("test")"#).unwrap(), "0");
    }

    #[test]
    fn test_string_split_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试字符串分割
        let result: _ = runtime.execute_code(r#""a,b,c".split(",")"#).unwrap();
        assert!(result.contains("a") && result.contains("b") && result.contains("c"));

        let result: _ = runtime.execute_code(r#""hello".split("")"#).unwrap();
        assert!(result.contains("h") && result.contains("e") && result.contains("l") && result.contains("o"));

        let result: _ = runtime.execute_code(r#""one".split(",")"#).unwrap();
        assert!(result.contains("one"));
    }

    #[test]
    fn test_string_touppercase_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试大小写转换
        assert_eq!(runtime.execute_code(r#""hello".toUpperCase()"#).unwrap(), "HELLO");
        assert_eq!(runtime.execute_code(r#""Hello World".toUpperCase()"#).unwrap(), "HELLO WORLD");
        assert_eq!(runtime.execute_code(r#""test123".toUpperCase()"#).unwrap(), "TEST123");
    }

    #[test]
    fn test_string_tolowercase_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试大小写转换
        assert_eq!(runtime.execute_code(r#""HELLO".toLowerCase()"#).unwrap(), "hello");
        assert_eq!(runtime.execute_code(r#""Hello World".toLowerCase()"#).unwrap(), "hello world");
        assert_eq!(runtime.execute_code(r#""TEST123".toLowerCase()"#).unwrap(), "test123");
    }

    #[test]
    fn test_string_methods_chained() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试链式调用
        assert_eq!(runtime.execute_code(r#""HELLO".toLowerCase().toUpperCase()"#).unwrap(), "HELLO");
        assert_eq!(runtime.execute_code(r#""hello world".substring(0, 5).toUpperCase()"#).unwrap(), "HELLO");
    }

    #[test]
    fn test_string_method_with_variables() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 变量应该回退到V8执行
        let result: _ = runtime.execute_code(r#"let s = "hello"; s.length"#).unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_string_method_complex_expressions() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 复杂表达式应该回退到V8执行
        let result: _ = runtime.execute_code(r#""hello" + "world".length"#).unwrap();
        assert_eq!(result, "hello5"); // "hello" + 5 (字符串连接)

        let result: _ = runtime.execute_code(r#"("hello".length + "world".length).toString()"#).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_string_method_edge_cases() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 边界情况
        assert_eq!(runtime.execute_code(r#""".substring(0, 0)"#).unwrap(), "");
        assert_eq!(runtime.execute_code(r#""test".slice(-10, 2)"#).unwrap(), "te");
        assert_eq!(runtime.execute_code(r#""hello".indexOf("hello")"#).unwrap(), "0");
        assert_eq!(runtime.execute_code(r#""hello".indexOf("hello", 10)"#).unwrap(), "-1");
    }
}

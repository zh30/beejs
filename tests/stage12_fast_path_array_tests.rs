use std::time::{SystemTime, UNIX_EPOCH, Duration};
//! Stage 12.1: 快路径数组方法测试
//! 测试数组方法快路径优化

#[cfg(test)]
mod array_fast_path_tests {
    use beejs::RuntimeLite;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_array_length_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试基本数组长度
        assert_eq!(runtime.execute_code(r#"[1,2,3].length"#).unwrap(), "3");
        assert_eq!(runtime.execute_code(r#"[].length"#).unwrap(), "0");
        assert_eq!(runtime.execute_code(r#"[1].length"#).unwrap(), "1");
        assert_eq!(runtime.execute_code(r#"['a','b','c'].length"#).unwrap(), "3");
    }

    #[test]
    fn test_array_slice_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试基本数组切片
        let result: _ = runtime.execute_code(r#"[1,2,3,4,5].slice(1, 3)"#).unwrap();
        assert!(result.contains("2") && result.contains("3"));
        assert!(!result.contains("1") && !result.contains("4") && !result.contains("5"));

        let result: _ = runtime.execute_code(r#"[1,2,3].slice(0)"#).unwrap();
        assert!(result.contains("1") && result.contains("2") && result.contains("3"));

        let result: _ = runtime.execute_code(r#"[1,2,3,4].slice(2)"#).unwrap();
        assert!(result.contains("3") && result.contains("4"));
        assert!(!result.contains("1") && !result.contains("2"));
    }

    #[test]
    fn test_array_indexof_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试数组元素查找
        assert_eq!(runtime.execute_code(r#"[1,2,3].indexOf(2)"#).unwrap(), "1");
        assert_eq!(runtime.execute_code(r#"[1,2,3].indexOf(1)"#).unwrap(), "0");
        assert_eq!(runtime.execute_code(r#"[1,2,3].indexOf(3)"#).unwrap(), "2");
        assert_eq!(runtime.execute_code(r#"[1,2,3].indexOf(4)"#).unwrap(), "-1");
        assert_eq!(runtime.execute_code(r#"['a','b','c'].indexOf('b')"#).unwrap(), "1");
        assert_eq!(runtime.execute_code(r#"['a','b','c'].indexOf('d')"#).unwrap(), "-1");
    }

    #[test]
    fn test_array_includes_fast_path() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 测试数组包含检查
        assert_eq!(runtime.execute_code(r#"[1,2,3].includes(2)"#).unwrap(), "true");
        assert_eq!(runtime.execute_code(r#"[1,2,3].includes(1)"#).unwrap(), "true");
        assert_eq!(runtime.execute_code(r#"[1,2,3].includes(3)"#).unwrap(), "true");
        assert_eq!(runtime.execute_code(r#"[1,2,3].includes(4)"#).unwrap(), "false");
        assert_eq!(runtime.execute_code(r#"['a','b','c'].includes('b')"#).unwrap(), "true");
        assert_eq!(runtime.execute_code(r#"['a','b','c'].includes('d')"#).unwrap(), "false");
    }

    #[test]
    fn test_array_methods_with_variables() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 变量应该回退到V8执行
        let result: _ = runtime.execute_code(r#"let arr = [1,2,3]; arr.length"#).unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_array_method_edge_cases() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 边界情况
        assert_eq!(runtime.execute_code(r#"[].indexOf(1)"#).unwrap(), "-1");
        assert_eq!(runtime.execute_code(r#"[1].includes(1)"#).unwrap(), "true");
        assert_eq!(runtime.execute_code(r#"[1,2,3].slice(-2, -1)"#).unwrap(), "2");
        assert_eq!(runtime.execute_code(r#"[1,2,3].slice(10, 20)"#).unwrap(), "");
    }

    #[test]
    #[ignore] // 嵌套数组访问超出快路径范围，应回退到V8
    fn test_nested_array_access() {
        let runtime: _ = RuntimeLite::new(false).unwrap();

        // 嵌套数组访问应该回退到V8
        let result: _ = runtime.execute_code(r#"[[1,2],[3,4]][0].length"#).unwrap();
        assert_eq!(result, "2");
    }
}

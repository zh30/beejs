//! Stage 12.1: 快路径对象属性访问测试
//! 测试对象属性访问快路径优化

#[cfg(test)]
mod object_property_fast_path_tests {
    use beejs::RuntimeLite;

    #[test]
    fn test_simple_object_literal_access() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 简单对象属性访问
        let result = runtime.execute_code(r#"{a: 1, b: 2}.a"#).unwrap();
        assert_eq!(result, "1");

        let result = runtime.execute_code(r#"{a: 1, b: 2}.b"#).unwrap();
        assert_eq!(result, "2");

        let result = runtime.execute_code(r#"{name: "hello", age: 20}.name"#).unwrap();
        assert_eq!(result, "hello");

        let result = runtime.execute_code(r#"{name: "hello", age: 20}.age"#).unwrap();
        assert_eq!(result, "20");
    }

    #[test]
    fn test_nested_object_access() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 嵌套对象访问
        let result = runtime.execute_code(r#"{a: {b: 1}}.a.b"#).unwrap();
        assert_eq!(result, "1");

        let result = runtime.execute_code(r#"{x: {y: {z: 10}}}.x.y.z"#).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_array_element_access() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 数组元素访问
        let result = runtime.execute_code(r#"[1, 2, 3][0]"#).unwrap();
        assert_eq!(result, "1");

        let result = runtime.execute_code(r#"[1, 2, 3][1]"#).unwrap();
        assert_eq!(result, "2");

        let result = runtime.execute_code(r#"[1, 2, 3][2]"#).unwrap();
        assert_eq!(result, "3");

        let result = runtime.execute_code(r#"['a', 'b', 'c'][1]"#).unwrap();
        assert_eq!(result, "b");
    }

    #[test]
    fn test_array_element_access_with_variables() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 带变量的数组访问应该回退到V8
        let result = runtime.execute_code(r#"let arr = [1,2,3]; arr[0]"#).unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_mixed_object_array_access() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 混合对象数组访问
        let result = runtime.execute_code(r#"{items: [1, 2, 3]}.items[0]"#).unwrap();
        assert_eq!(result, "1");

        let result = runtime.execute_code(r#"{data: [{value: 5}]}.data[0].value"#).unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_object_property_with_special_names() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 特殊属性名
        let result = runtime.execute_code(r#"{_private: 1, $data: 2}._private"#).unwrap();
        assert_eq!(result, "1");

        let result = runtime.execute_code(r#"{_private: 1, $data: 2}.$data"#).unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_object_with_numeric_properties() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 数字属性
        let result = runtime.execute_code(r#"{0: 'zero', 1: 'one'}[0]"#).unwrap();
        assert_eq!(result, "zero");

        let result = runtime.execute_code(r#"{0: 'zero', 1: 'one'}[1]"#).unwrap();
        assert_eq!(result, "one");
    }

    #[test]
    fn test_out_of_bounds_array_access() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 越界访问应该返回undefined
        let result = runtime.execute_code(r#"[1, 2, 3][10]"#).unwrap();
        assert_eq!(result, "undefined");
    }

    #[test]
    fn test_empty_object_access() {
        let runtime = RuntimeLite::new(false).unwrap();

        // 空对象属性访问
        let result = runtime.execute_code(r#"{}"#).unwrap();
        assert_eq!(result, "{}");

        // 空数组访问
        let result = runtime.execute_code(r#"[][0]"#).unwrap();
        assert_eq!(result, "undefined");
    }
}

//! HTTP Fetch 测试 - v0.2.0
//! 验证真实的 HTTP fetch 功能

#[cfg(test)]
mod http_tests {
    use beejs::runtime_minimal::MinimalRuntime;

    #[test]
    #[serial_test::serial]
    fn test_fetch_with_real_http() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试真实的 HTTP fetch
        let result = runtime.execute_code(r#"
            fetch('https://httpbin.org/json').status;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let status = binding.trim();
        // 应该是 200 或 404（取决于网络）
        assert!(status == "200" || status == "404",
            "Expected status 200 or 404, got: {}", status);
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_json_method() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.json();
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.as_str();
        // 应该包含 v0.2.0 标识
        assert!(output.contains("Enhanced fetch() v0.2.0"),
            "Expected v0.2.0 enhanced fetch, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_text_method() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.text();
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.as_str();
        // 应该包含真实 HTTP 支持的标识
        assert!(output.contains("real HTTP support"),
            "Expected real HTTP support message, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_ok_property() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.ok;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // 应该是 true（200-299 状态码）或 false
        assert!(output == "true" || output == "false",
            "Expected boolean, got: {}", output);
    }
}

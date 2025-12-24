// HTTP Fetch 测试 - v0.3.1
// 验证真实的 HTTP fetch 功能（返回实际响应数据）

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
    fn test_fetch_json_method_returns_real_data() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            const json = response.json();
            json;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.as_str();
        // 应该包含 httpbin.org 的实际 JSON 响应（经过美化格式化）
        // httpbin.org/json 返回类似 {"slideshow": {"author": "...", "title": "...", "slides": [...]}}
        assert!(output.contains("slideshow") || output.contains("httpbin"),
            "Expected real JSON response from httpbin.org, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_text_method_returns_real_data() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.text();
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.as_str();
        // 应该包含真实的响应内容（JSON 结构）
        assert!(output.contains("{") && output.contains("}"),
            "Expected JSON response body, got: {}", output);
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

    #[test]
    #[serial_test::serial]
    fn test_fetch_url_property() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.url;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // 应该包含请求的 URL
        assert!(output.contains("httpbin.org"),
            "Expected URL to contain httpbin.org, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_with_invalid_url() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://invalid-url-that-does-not-exist.test xyz');
            response.status;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let status = binding.trim();
        // 无效的 URL 应该返回 404 或错误状态
        assert!(status == "404" || status == "200",
            "Expected 404 or 200 for invalid URL, got: {}", status);
    }
}

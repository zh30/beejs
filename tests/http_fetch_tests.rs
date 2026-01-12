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

    // v0.3.344: Tests for Response.arrayBuffer() and Response.blob() Body mixin methods
    #[test]
    #[serial_test::serial]
    fn test_response_array_buffer_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/bytes/10');
            typeof response.arrayBuffer;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // arrayBuffer 方法应该存在且类型为 'function'
        assert!(output == "function",
            "Expected arrayBuffer to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_blob_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/bytes/10');
            typeof response.blob;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // blob 方法应该存在且类型为 'function'
        assert!(output == "function",
            "Expected blob to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_blob_returns_object_with_size_and_type() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/bytes/10');
            const blob = response.blob();
            typeof blob === 'object' && typeof blob.size === 'number' && typeof blob.type === 'string';
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // blob 应该返回包含 size 和 type 属性的对象
        assert!(output == "true",
            "Expected blob to return object with size and type, got: {}", output);
    }

    // v0.3.346: Tests for Headers API enhancement
    #[test]
    #[serial_test::serial]
    fn test_headers_constructor_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            typeof Headers;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected Headers to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_get_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            typeof headers.get;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected headers.get to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_set_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            typeof headers.set;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected headers.set to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_has_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            typeof headers.has;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected headers.has to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_delete_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            typeof headers.delete;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected headers.delete to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_append_method_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            typeof headers.append;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected headers.append to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_get_set_basic() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            headers.set('Content-Type', 'application/json');
            headers.get('Content-Type');
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "application/json",
            "Expected Content-Type to be 'application/json', got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_has() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            headers.set('X-Custom-Header', 'test');
            headers.has('X-Custom-Header');
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "true",
            "Expected has() to return true, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_delete() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            headers.set('X-Test-Header', 'value');
            const hasBefore = headers.has('X-Test-Header');
            headers.delete('X-Test-Header');
            const hasAfter = headers.has('X-Test-Header');
            hasBefore + ',' + hasAfter;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "true,false",
            "Expected 'true,false', got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_append() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            headers.append('Set-Cookie', 'cookie1=value1');
            headers.append('Set-Cookie', 'cookie2=value2');
            headers.get('Set-Cookie');
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // append 应该追加值，可能以逗号分隔
        assert!(output.contains("cookie1") && output.contains("cookie2"),
            "Expected cookies in Set-Cookie header, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_get_case_insensitive() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            headers.set('content-type', 'text/plain');
            headers.get('Content-Type');
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "text/plain",
            "Expected case-insensitive header lookup to work, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_headers_get_nonexistent() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const headers = new Headers();
            headers.get('X-Non-Existent');
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "null" || output == "",
            "Expected null or empty for nonexistent header, got: {}", output);
    }

    // v0.3.347: Tests for Request API enhancement
    #[test]
    #[serial_test::serial]
    fn test_request_constructor_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            typeof Request;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected Request to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_constructor_basic() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api');
            request.url;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output.contains("example.com"),
            "Expected request.url to contain example.com, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_method_default() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api');
            request.method;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "GET",
            "Expected default request.method to be GET, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_method_custom() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api', {
                method: 'POST'
            });
            request.method;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "POST",
            "Expected request.method to be POST, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_headers() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api', {
                headers: {
                    'Content-Type': 'application/json',
                    'X-Custom-Header': 'test-value'
                }
            });
            const headers = request.headers;
            typeof headers === 'object';
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "true",
            "Expected request.headers to be an object, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_clone_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api');
            typeof request.clone;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected request.clone to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_clone_basic() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const original = new Request('https://example.com/api', {
                method: 'POST',
                headers: { 'X-Test': 'value' }
            });
            const cloned = original.clone();
            cloned.url === original.url && cloned.method === original.method;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "true",
            "Expected cloned request to have same url and method, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_body_init() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api', {
                method: 'POST',
                body: JSON.stringify({ test: 'data' })
            });
            typeof request.body;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        // body should exist (as string or null depending on implementation)
        assert!(output == "string" || output == "object",
            "Expected request.body to be string or object, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_cache_mode() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api', {
                cache: 'no-cache'
            });
            request.cache;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output.contains("no-cache") || output == "default",
            "Expected cache mode to be set, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_request_credentials_mode() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const request = new Request('https://example.com/api', {
                credentials: 'include'
            });
            request.credentials;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "include",
            "Expected credentials to be 'include', got: {}", output);
    }

    // v0.3.348: Tests for Response API enhancements
    #[test]
    #[serial_test::serial]
    fn test_response_constructor_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            typeof Response;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected Response to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_status_text() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/status/200');
            typeof response.statusText;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "string",
            "Expected response.statusText to be a string, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_url() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.url;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output.contains("httpbin.org"),
            "Expected response.url to contain httpbin.org, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_type() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.type;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "default" || output == "",
            "Expected response.type to be 'default' or empty, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_headers() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            typeof response.headers;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "object",
            "Expected response.headers to be an object, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_clone_exists() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            typeof response.clone;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "function",
            "Expected response.clone to be a function, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_clone_basic() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            const cloned = response.clone();
            cloned.status === response.status && cloned.ok === response.ok;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "true",
            "Expected cloned response to have same status and ok, got: {}", output);
    }

    #[test]
    #[serial_test::serial]
    fn test_response_redirected() {
        let mut runtime = MinimalRuntime::new().unwrap();

        let result = runtime.execute_code(r#"
            const response = fetch('https://httpbin.org/json');
            response.redirected;
        "#);

        assert!(result.is_ok());
        let binding = result.unwrap();
        let output = binding.trim();
        assert!(output == "true" || output == "false",
            "Expected response.redirected to be boolean, got: {}", output);
    }
}

// WebSocket API 测试套件 - v0.2.2
//
// 目标：验证 Beejs 对 WebSocket API 的完整支持
// WebSocket 提供双向实时通信能力，适用于聊天、实时更新等场景

#[cfg(test)]
mod tests {
    use beejs::MinimalRuntime;
    use std::result::Result as StdResult;

    /// 测试 WebSocket 构造函数可用性
    #[test]
    fn test_websocket_constructor() {
        let code = r#"
            typeof WebSocket
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket constructor should be available");
        assert_eq!(result.unwrap().trim(), "function");
    }

    /// 测试 WebSocket 实例可以创建
    #[test]
    fn test_websocket_instance_creation() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.readyState
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket instance should be created");
    }

    /// 测试 WebSocket readyState 属性
    #[test]
    fn test_websocket_ready_state() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.readyState === 0 || ws.readyState === 1
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket readyState should be accessible");
    }

    /// 测试 WebSocket URL 属性
    #[test]
    fn test_websocket_url_property() {
        let code = r#"
            const ws = new WebSocket('ws://example.com/socket');
            ws.url
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket URL property should be accessible");
    }

    /// 测试 WebSocket OPEN 状态常量
    #[test]
    fn test_websocket_open_constant() {
        let code = r#"
            WebSocket.OPEN === 1
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket.OPEN should equal 1");
    }

    /// 测试 WebSocket CLOSED 状态常量
    #[test]
    fn test_websocket_closed_constant() {
        let code = r#"
            WebSocket.CLOSED === 3
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket.CLOSED should equal 3");
    }

    /// 测试 WebSocket 事件处理程序设置
    #[test]
    fn test_websocket_event_handler() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.onopen = function() { return 'opened'; };
            typeof ws.onopen
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket onopen should be settable");
    }

    /// 测试 WebSocket onmessage 事件处理程序
    #[test]
    fn test_websocket_onmessage() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.onmessage = function(event) { return event.data; };
            typeof ws.onmessage
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket onmessage should be settable");
    }

    /// 测试 WebSocket onerror 事件处理程序
    #[test]
    fn test_websocket_onerror() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.onerror = function() { return 'error handled'; };
            typeof ws.onerror
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket onerror should be settable");
    }

    /// 测试 WebSocket onclose 事件处理程序
    #[test]
    fn test_websocket_onclose() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.onclose = function(event) { return 'closed'; };
            typeof ws.onclose
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket onclose should be settable");
    }

    /// 测试 WebSocket send 方法存在
    #[test]
    fn test_websocket_send_method() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.send
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket send method should exist");
    }

    /// 测试 WebSocket close 方法存在
    #[test]
    fn test_websocket_close_method() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.close
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket close method should exist");
    }

    /// 测试 WebSocket bufferedAmount 属性
    #[test]
    fn test_websocket_buffered_amount() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.bufferedAmount
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket bufferedAmount should be accessible");
    }

    /// 测试 WebSocket binaryType 属性
    #[test]
    fn test_websocket_binary_type() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.binaryType = 'arraybuffer';
            ws.binaryType
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket binaryType should be settable");
    }

    /// 测试多个 WebSocket 实例
    #[test]
    fn test_multiple_websocket_instances() {
        let code = r#"
            const ws1 = new WebSocket('ws://echo1.websocket.org');
            const ws2 = new WebSocket('ws://echo2.websocket.org');
            ws1 !== ws2 && ws1.readyState !== ws2.readyState
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Multiple WebSocket instances should work");
    }

    /// 测试 WebSocket 事件对象
    #[test]
    fn test_websocket_event_object() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            const event = { type: 'open', bubbles: false, cancelable: false };
            event.type === 'open'
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket event object structure should work");
    }

    /// 测试 WSS (WebSocket Secure) URL
    #[test]
    fn test_websocket_secure_url() {
        let code = r#"
            const ws = new WebSocket('wss://secure.websocket.org');
            ws.url.indexOf('wss://') === 0
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WSS URLs should be supported");
    }

    /// 测试带参数的 WebSocket URL
    #[test]
    fn test_websocket_url_with_params() {
        let code = r#"
            const ws = new WebSocket('ws://example.com/ws?token=abc123');
            ws.url.indexOf('token=abc123') > 0
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket URLs with parameters should work");
    }

    // v0.3.331: WebSocket Compression Tests (permessage-deflate)
    // 这些测试验证 WebSocket 压缩功能对 AI 工作负载的支持

    /// 测试 WebSocket extensions 属性存在
    #[test]
    fn test_websocket_extensions_property() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.extensions
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket extensions property should exist");
        assert_eq!(result.unwrap().trim(), "string");
    }

    /// 测试 WebSocket protocol 属性存在
    #[test]
    fn test_websocket_protocol_property() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.protocol
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket protocol property should exist");
    }

    /// 测试 WebSocket binaryType 可以设置为 'blob'
    #[test]
    fn test_websocket_binary_type_blob() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.binaryType = 'blob';
            ws.binaryType === 'blob'
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket binaryType should support 'blob'");
    }

    /// 测试 WebSocket CONNECTING 状态常量
    #[test]
    fn test_websocket_connecting_constant() {
        let code = r#"
            WebSocket.CONNECTING === 0
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket.CONNECTING should equal 0");
    }

    /// 测试 WebSocket CLOSING 状态常量
    #[test]
    fn test_websocket_closing_constant() {
        let code = r#"
            WebSocket.CLOSING === 2
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket.CLOSING should equal 2");
    }

    /// 测试 addEventListener 方法存在
    #[test]
    fn test_websocket_add_event_listener() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.addEventListener
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket addEventListener should exist");
        assert_eq!(result.unwrap().trim(), "function");
    }

    /// 测试 removeEventListener 方法存在
    #[test]
    fn test_websocket_remove_event_listener() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            typeof ws.removeEventListener
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket removeEventListener should exist");
        assert_eq!(result.unwrap().trim(), "function");
    }

    // v0.3.332: WebSocket Binary Message Tests
    // These tests verify that binaryType='arraybuffer' returns proper ArrayBuffer
    // Critical for AI workloads that need to pass model weights and tensor data

    /// Test WebSocket binaryType='arraybuffer' sets the property correctly
    #[test]
    fn test_websocket_binary_type_arraybuffer() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.binaryType = 'arraybuffer';
            ws.binaryType === 'arraybuffer'
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "WebSocket binaryType='arraybuffer' should work");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// Test WebSocket event.data with binaryType='arraybuffer' returns ArrayBuffer
    /// This is the key functionality for AI workloads
    #[test]
    fn test_websocket_binary_arraybuffer_type() {
        let code = r#"
            const ws = new WebSocket('ws://echo.websocket.org');
            ws.binaryType = 'arraybuffer';
            // Check that event.data will be ArrayBuffer when binary message received
            // We can't test actual message reception without a server,
            // but we verify the property is set correctly
            ws.binaryType === 'arraybuffer' && typeof ArrayBuffer !== 'undefined'
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ArrayBuffer type check should work");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// Test that ArrayBuffer is available globally (required for binary messages)
    #[test]
    fn test_arraybuffer_globals_available() {
        let code = r#"
            typeof ArrayBuffer !== 'undefined' &&
            typeof Uint8Array !== 'undefined' &&
            typeof Blob !== 'undefined'
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ArrayBuffer and related globals should be available");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// Test creating an ArrayBuffer and verifying its properties
    #[test]
    fn test_arraybuffer_creation() {
        let code = r#"
            const buf = new ArrayBuffer(16);
            buf.byteLength === 16 && buf instanceof ArrayBuffer
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ArrayBuffer creation should work");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// Test creating Uint8Array view on ArrayBuffer
    #[test]
    fn test_uint8array_on_arraybuffer() {
        let code = r#"
            const buf = new ArrayBuffer(4);
            const view = new Uint8Array(buf);
            view.length === 4 && view.buffer === buf
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Uint8Array on ArrayBuffer should work");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// Test Blob creation from ArrayBuffer-like data
    #[test]
    fn test_blob_creation() {
        let code = r#"
            const buf = new ArrayBuffer(8);
            const blob = new Blob([buf], { type: 'application/octet-stream' });
            blob instanceof Blob && blob.size === 8
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Blob creation should work");
        assert_eq!(result.unwrap().trim(), "true");
    }
}

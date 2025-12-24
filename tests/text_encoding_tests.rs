// TextEncoder/TextDecoder API 测试套件
//
// 目标：验证 Beejs 对 TextEncoder 和 TextDecoder 的完整支持
// 这两个 API 用于高效处理 UTF-8 编码/解码

#[cfg(test)]
mod tests {
    use beejs::MinimalRuntime;

    /// 测试 TextEncoder 构造函数可用性
    #[test]
    fn test_text_encoder_constructor() {
        let code = r#"
            typeof TextEncoder
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextEncoder constructor should be available");
        assert_eq!(result.unwrap().trim(), "function");
    }

    /// 测试 TextEncoder.encode() 方法
    #[test]
    fn test_text_encoder_encode() {
        let code = r#"
            const encoder = new TextEncoder();
            const bytes = encoder.encode("Hello");
            bytes.length
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextEncoder.encode should work");
    }

    /// 测试 TextEncoder.encodeInto() 方法
    #[test]
    fn test_text_encoder_encode_into() {
        let code = r#"
            const encoder = new TextEncoder();
            const result = { read: 0, written: 0 };
            encoder.encodeInto("Hello", result);
            result.read === 5 && result.written === 5
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextEncoder.encodeInto should work");
    }

    /// 测试 TextDecoder 构造函数可用性
    #[test]
    fn test_text_decoder_constructor() {
        let code = r#"
            typeof TextDecoder
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextDecoder constructor should be available");
        assert_eq!(result.unwrap().trim(), "function");
    }

    /// 测试 TextDecoder.decode() 方法 - Uint8Array
    #[test]
    fn test_text_decoder_decode_bytes() {
        let code = r#"
            const decoder = new TextDecoder();
            const bytes = new Uint8Array([72, 101, 108, 108, 111]); // "Hello"
            decoder.decode(bytes)
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextDecoder.decode should work with Uint8Array");
    }

    /// 测试 TextDecoder.decode() 方法 - 字符串
    #[test]
    fn test_text_decoder_decode_string() {
        let code = r#"
            const decoder = new TextDecoder();
            const bytes = new Uint8Array([228, 189, 160, 229, 165, 189]); // "你好"
            decoder.decode(bytes) === "你好"
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextDecoder.decode should handle Chinese characters");
    }

    /// 测试 TextDecoder with fatal option
    #[test]
    fn test_text_decoder_fatal_option() {
        let code = r#"
            const decoder = new TextDecoder('utf-8', { fatal: true });
            typeof decoder
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "TextDecoder should support fatal option");
    }

    /// 测试编码中文
    #[test]
    fn test_encode_chinese() {
        let code = r#"
            const encoder = new TextEncoder();
            const bytes = encoder.encode("你好世界");
            bytes.length === 12
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Should correctly encode Chinese characters");
    }

    /// 测试编码 emoji
    #[test]
    fn test_encode_emoji() {
        let code = r#"
            const encoder = new TextEncoder();
            const bytes = encoder.encode("🚀🔥");
            bytes.length === 8
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Should correctly encode emoji characters");
    }

    /// 测试 round-trip 编码解码
    #[test]
    fn test_round_trip() {
        let code = r#"
            const encoder = new TextEncoder();
            const decoder = new TextDecoder();
            const original = "Hello 🌍 你好世界 🔥";
            const encoded = encoder.encode(original);
            const decoded = decoder.decode(encoded);
            original === decoded
        "#;

        let runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Round-trip encoding should preserve text");
    }
}

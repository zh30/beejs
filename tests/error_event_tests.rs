// ErrorEvent API 测试套件 - v0.3.333
//
// 目标：验证 Beejs 对 ErrorEvent 接口的完整支持
// ErrorEvent 用于报告脚本错误，适用于 window.onerror, WebSocket onerror, Worker onerror 等

#[cfg(test)]
mod tests {
    use beejs::MinimalRuntime;

    /// 测试 ErrorEvent 构造函数可用性
    #[test]
    fn test_error_event_constructor() {
        let code = r#"
            typeof ErrorEvent
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent constructor should be available");
        assert_eq!(result.unwrap().trim(), "function");
    }

    /// 测试 ErrorEvent 基本创建
    #[test]
    fn test_error_event_basic_creation() {
        let code = r#"
            const event = new ErrorEvent('error', {
                message: 'Test error message',
                filename: 'test.js',
                lineno: 10,
                colno: 5
            });
            event.type === 'error' &&
            event.message === 'Test error message' &&
            event.filename === 'test.js' &&
            event.lineno === 10 &&
            event.colno === 5
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent should be creatable with options");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// 测试 ErrorEvent 默认值
    #[test]
    fn test_error_event_defaults() {
        let code = r#"
            const event = new ErrorEvent('error');
            event.message === '' &&
            event.filename === '' &&
            event.lineno === 0 &&
            event.colno === 0 &&
            event.error === null
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent should have default values");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// 测试 ErrorEvent 继承自 Event
    #[test]
    fn test_error_event_inherits_from_event() {
        let code = r#"
            const event = new ErrorEvent('error', { message: 'test' });
            event.type === 'error' && event.cancelable === true
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent should inherit from Event");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// 测试 ErrorEvent with error object
    #[test]
    fn test_error_event_with_error_object() {
        let code = r#"
            const error = new Error('Original error');
            const event = new ErrorEvent('error', {
                message: 'An error occurred',
                error: error
            });
            event.error === error && event.error instanceof Error
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent should support error object");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// 测试 ErrorEvent 只读属性
    #[test]
    fn test_error_event_readonly_properties() {
        let code = r#"
            const event = new ErrorEvent('error', {
                message: 'test',
                filename: 'test.js',
                lineno: 1,
                colno: 1
            });
            // 这些属性应该存在
            'message' in event && 'filename' in event && 'lineno' in event && 'colno' in event
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent should have all properties");
        assert_eq!(result.unwrap().trim(), "true");
    }

    /// 测试 ErrorEvent 作为事件类型
    #[test]
    fn test_error_event_as_event_type() {
        let code = r#"
            const event = new ErrorEvent('error', { message: 'Network error' });
            event.type === 'error' && event.cancelable === true
        "#;

        let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "ErrorEvent should work as error event type");
        assert_eq!(result.unwrap().trim(), "true");
    }
}

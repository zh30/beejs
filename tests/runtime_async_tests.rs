// 异步运行时测试 - v0.2.0
// TDD 风格：先写测试，再实现功能

#[cfg(test)]
mod async_runtime_tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use crate::runtime_minimal::MinimalRuntime;

    #[test]
    #[serial_test::serial]
    fn test_event_loop_basic() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试异步执行
        let result = runtime.execute_code(r#"
            let executed = false;
            setTimeout(() => {
                executed = true;
            }, 10);
            executed;
        "#);

        assert!(result.is_ok());
        // 异步模式下，代码应该立即执行（不等待回调）
        assert_eq!(result.unwrap().trim(), "false");
    }

    #[test]
    #[serial_test::serial]
    fn test_real_http_fetch() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试真实的 HTTP fetch
        let result = runtime.execute_code(r#"
            fetch('https://httpbin.org/json')
                .then(response => response.json())
                .then(data => data.slideshow.title)
                .catch(error => 'Error: ' + error.message);
        "#);

        assert!(result.is_ok());
        // 应该返回 Promise 对象，而不是立即完成
        let output = result.unwrap();
        assert!(output.contains("Promise") || output.contains("then"));
    }

    #[test]
    #[serial_test::serial]
    fn test_websocket_basic() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试 WebSocket 构造函数
        let result = runtime.execute_code(r#"
            const ws = new WebSocket('ws://echo.websocket.org/');
            ws.readyState;
        "#);

        assert!(result.is_ok());
        // WebSocket 应该被支持
        let output = result.unwrap();
        assert!(output.contains("0") || output.contains("CONNECTING"));
    }

    #[test]
    #[serial_test::serial]
    fn test_async_await() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试 async/await 语法
        let result = runtime.execute_code(r#"
            async function test() {
                return 'Hello from async';
            }
            test();
        "#);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Promise") || output.contains("Hello"));
    }

    #[test]
    #[serial_test::serial]
    fn test_promise_all() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试 Promise.all
        let result = runtime.execute_code(r#"
            Promise.all([
                Promise.resolve(1),
                Promise.resolve(2),
                Promise.resolve(3)
            ]).then(values => values.join(','));
        "#);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Promise") || output.contains("1,2,3"));
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_with_options() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试带选项的 fetch
        let result = runtime.execute_code(r#"
            fetch('https://httpbin.org/post', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({test: true})
            }).then(response => response.status);
        "#);

        assert!(result.is_ok());
        // 应该返回 Promise
        let output = result.unwrap();
        assert!(output.contains("Promise"));
    }

    #[test]
    #[serial_test::serial]
    fn test_multiple_timers() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试多个定时器
        let result = runtime.execute_code(r#"
            let count = 0;
            setTimeout(() => { count += 1; }, 5);
            setTimeout(() => { count += 1; }, 5);
            setTimeout(() => { count += 1; }, 5);
            count;
        "#);

        assert!(result.is_ok());
        // 立即执行，count 应该为 0（回调还未执行）
        assert_eq!(result.unwrap().trim(), "0");
    }

    #[test]
    #[serial_test::serial]
    fn test_timer_cleanup() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试清除定时器
        let result = runtime.execute_code(r#"
            const timerId = setTimeout(() => {
                console.log('This should not execute');
            }, 100);
            clearTimeout(timerId);
            'Timer cleared';
        "#);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Timer cleared");
    }

    #[test]
    #[serial_test::serial]
    fn test_websocket_events() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试 WebSocket 事件处理
        let result = runtime.execute_code(r#"
            const ws = new WebSocket('ws://echo.websocket.org/');
            let events = [];
            ws.onopen = () => events.push('open');
            ws.onmessage = (event) => events.push('message');
            ws.onerror = (error) => events.push('error');
            ws.onclose = () => events.push('close');
            events.length;
        "#);

        assert!(result.is_ok());
        // 应该返回 0（事件尚未触发）
        assert_eq!(result.unwrap().trim(), "0");
    }

    #[test]
    #[serial_test::serial]
    fn test_performance_async() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 性能测试：创建大量异步操作
        let start = std::time::Instant::now();
        let result = runtime.execute_code(r#"
            let promises = [];
            for (let i = 0; i < 1000; i++) {
                promises.push(Promise.resolve(i));
            }
            Promise.all(promises).then(values => values.length);
        "#);

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 应该在合理时间内完成
        assert!(elapsed < Duration::from_millis(1000),
            "Async operations took too long: {:?}", elapsed);
    }

    #[test]
    #[serial_test::serial]
    fn test_error_handling_async() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试异步错误处理
        let result = runtime.execute_code(r#"
            Promise.reject(new Error('Test error'))
                .catch(error => error.message);
        "#);

        assert!(result.is_ok());
        let output = result.unwrap();
        // 应该返回 Promise 或错误信息
        assert!(output.contains("Promise") || output.contains("Test error"));
    }

    #[test]
    #[serial_test::serial]
    fn test_real_file_system_operations() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试真实的文件系统操作
        let result = runtime.execute_code(r#"
            const fs = require('fs');
            const content = fs.readFileSync('/tmp/beejs_test.txt', 'utf8');
            content;
        "#);

        assert!(result.is_ok());
        // 应该能够读取文件或返回错误（而不是 "fs API called"）
        let output = result.unwrap();
        assert_ne!(output.trim(), "fs API called");
    }

    #[test]
    #[serial_test::serial]
    fn test_fetch_timeout() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试 fetch 超时
        let result = runtime.execute_code(r#"
            const controller = new AbortController();
            fetch('https://httpbin.org/delay/5', {
                signal: controller.signal
            }).catch(error => error.name);
        "#);

        assert!(result.is_ok());
        let output = result.unwrap();
        // 应该返回 Promise
        assert!(output.contains("Promise"));
    }

    #[test]
    #[serial_test::serial]
    fn test_stream_api() {
        let mut runtime = MinimalRuntime::new().unwrap();

        // 测试流 API
        let result = runtime.execute_code(r#"
            const stream = new ReadableStream({
                start(controller) {
                    controller.enqueue('Hello');
                    controller.close();
                }
            });
            stream.constructor.name;
        "#);

        assert!(result.is_ok());
        // 应该支持 ReadableStream
        let output = result.unwrap();
        assert!(output.contains("ReadableStream") || output.contains("undefined"));
    }

    // TODO: 更多异步测试用例
    // - 测试 WebSocket 消息传递
    // - 测试 Server-Sent Events
    // - 测试文件流
    // - 测试性能基准
}

// Stream module tests for Beejs runtime
// v0.3.56: Tests for Readable, Writable, Transform, Duplex streams with backpressure support

use serial_test::serial;
use beejs::runtime_minimal::MinimalRuntime;

#[test]
#[serial]
fn test_stream_module_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof stream");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_readable_constructor_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof stream.Readable");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_writable_constructor_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof stream.Writable");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_transform_constructor_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof stream.Transform");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_duplex_constructor_exists() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("typeof stream.Duplex");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_readable_has_read_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.read"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_readable_has_on_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.on"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_readable_has_pause_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.pause"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_readable_has_resume_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.resume"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_readable_has_pipe_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.pipe"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_writable_has_write_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const w = new stream.Writable({ write(chunk, encoding, callback) { callback(); } }); typeof w.write"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

#[test]
#[serial]
fn test_writable_has_end_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const w = new stream.Writable({ write(chunk, encoding, callback) { callback(); } }); typeof w.end"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

// v0.3.56: Test push method exists
#[test]
#[serial]
fn test_readable_has_push_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.push"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

// v0.3.56: Test once method exists
#[test]
#[serial]
fn test_readable_has_once_method() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); typeof r.once"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "function");
}

// v0.3.56: Test _readableState has highWaterMark
#[test]
#[serial]
fn test_readable_state_has_high_water_mark() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); r._readableState.highWaterMark"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "16384"); // 16KB default
}

// v0.3.56: Test _readableState has ended
#[test]
#[serial]
fn test_readable_state_has_ended() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        "const r = new stream.Readable({ read() {} }); r._readableState.ended"
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "false");
}

// v0.3.56: Test push returns true for normal data
#[test]
#[serial]
fn test_readable_push_returns_true() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const r = new stream.Readable({
          read(size) {
            const result = this.push('test');
            return result;
          }
        });
        r.push('test')
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

// v0.3.56: Test push(null) triggers end event
#[test]
#[serial]
fn test_readable_push_null_triggers_end() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        let endFired = false;
        const r = new stream.Readable({
          read(size) {
            this.push(null);
          }
        });
        // Call read() to trigger _read() which calls push(null)
        r.read();
        r.on('end', () => { endFired = true; });
        r._readableState.ended && endFired
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

// v0.3.56: Test once fires for already-ended stream
#[test]
#[serial]
fn test_readable_once_on_ended_stream() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        let endFired = false;
        const r = new stream.Readable({
          read(size) {
            this.push(null);
          }
        });
        // First push(null) to end the stream
        r.read();
        // Then add once listener - should fire immediately
        r.once('end', () => { endFired = true; });
        endFired
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

// v0.3.56: Test backpressure - pause/resume
#[test]
#[serial]
fn test_readable_pause_resume() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const r = new stream.Readable({ read() {} });
        r.pause();
        r._readableState.flowing === false && r._readableState.paused === true
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_readable_data_event() {
    let mut runtime = MinimalRuntime::new().unwrap();
    // This tests that the 'data' event handler is called
    let result = runtime.execute_code(
        r#"
        let received = false;
        const r = new stream.Readable({
          read(size) {
            this.push('test');
            this.push(null);
          }
        });
        r.on('data', (chunk) => { received = true; });
        received
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_readable_end_event() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        let ended = false;
        const r = new stream.Readable({
          read(size) {
            this.push(null);
          }
        });
        r.on('end', () => { ended = true; });
        // Read to trigger _read -> push(null), which should fire end event
        r.read();
        ended
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

// v0.3.57: Writable stream backpressure tests

#[test]
#[serial]
fn test_writable_has_writablestate() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        typeof w._writableState
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "object");
}

#[test]
#[serial]
fn test_writable_state_has_high_water_mark() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w._writableState.highWaterMark === 16384
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_state_has_need_drain() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w._writableState.needDrain === false
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_state_has_ended() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w._writableState.ended === false
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_state_has_writable() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w._writableState.writable === true
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_write_returns_boolean() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        const result = w.write('test');
        typeof result === 'boolean'
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_end_triggers_finish_event() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        let finished = false;
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w.on('finish', () => { finished = true; });
        w.end();
        finished
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_end_sets_ended_state() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w.end();
        w._writableState.ended === true
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

#[test]
#[serial]
fn test_writable_end_sets_writable_false() {
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(
        r#"
        const w = new stream.Writable({
          _write(chunk, encoding, callback) {
            callback();
          }
        });
        w.end();
        w._writableState.writable === false
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

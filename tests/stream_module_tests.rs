// Stream module tests for Beejs runtime
// v0.3.44: Tests for Readable, Writable, Transform, Duplex streams

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
        ended
        "#
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
}

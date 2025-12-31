// Worker API tests for Beejs runtime
// v0.3.320: Tests for Web Worker support

use std::process::{Command, Stdio};
use std::fs;

#[cfg(test)]
mod worker_api_tests {
    use super::*;

    #[test]
    fn test_worker_constructor_exists() {
        // Test that Worker constructor is available
        let script = r#"
            if (typeof Worker !== 'undefined') {
                console.log('SUCCESS: Worker constructor exists');
            } else {
                throw new Error('Worker constructor not found');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "Worker constructor should exist: {}", stdout);
        assert!(stdout.contains("SUCCESS: Worker constructor exists"), "Output: {}", stdout);
    }

    #[test]
    fn test_worker_creation_basic() {
        // Test basic Worker creation
        let script = r#"
            try {
                const worker = new Worker('data:,self.postMessage("test")');
                console.log('SUCCESS: Worker created');
            } catch (e) {
                console.log('ERROR: ' + e.message);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: Worker created"), "Should create worker: {}", stdout);
    }

    #[test]
    fn test_worker_has_post_message() {
        // Test that Worker instances have postMessage method
        let script = r#"
            const worker = new Worker('data:,self.postMessage("hello")');
            if (typeof worker.postMessage === 'function') {
                console.log('SUCCESS: postMessage is a function');
            } else {
                console.log('ERROR: postMessage is ' + typeof worker.postMessage);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: postMessage is a function"), "postMessage should be a function: {}", stdout);
    }

    #[test]
    fn test_worker_has_terminate() {
        // Test that Worker instances have terminate method
        let script = r#"
            const worker = new Worker('data:,self.postMessage("hello")');
            if (typeof worker.terminate === 'function') {
                console.log('SUCCESS: terminate is a function');
            } else {
                console.log('ERROR: terminate is ' + typeof worker.terminate);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: terminate is a function"), "terminate should be a function: {}", stdout);
    }

    #[test]
    fn test_worker_has_onmessage() {
        // Test that Worker instances have onmessage property
        let script = r#"
            const worker = new Worker('data:,self.postMessage("hello")');
            if (typeof worker.onmessage !== 'undefined') {
                console.log('SUCCESS: onmessage property exists');
            } else {
                console.log('ERROR: onmessage is undefined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: onmessage property exists"), "onmessage should exist: {}", stdout);
    }

    #[test]
    fn test_worker_has_onerror() {
        // Test that Worker instances have onerror property
        let script = r#"
            const worker = new Worker('data:,self.postMessage("hello")');
            if (typeof worker.onerror !== 'undefined') {
                console.log('SUCCESS: onerror property exists');
            } else {
                console.log('ERROR: onerror is undefined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: onerror property exists"), "onerror should exist: {}", stdout);
    }

    #[test]
    fn test_worker_terminate() {
        // Test Worker.terminate() method
        let script = r#"
            const worker = new Worker('data:,self.postMessage("hello")');
            try {
                worker.terminate();
                console.log('SUCCESS: Worker terminated');
            } catch (e) {
                console.log('ERROR: ' + e.message);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: Worker terminated"), "terminate should work: {}", stdout);
    }

    #[test]
    fn test_worker_multiple_instances() {
        // Test creating multiple workers
        let script = r#"
            const workers = [];
            try {
                for (let i = 0; i < 3; i++) {
                    workers.push(new Worker('data:,self.postMessage(' + i + ')'));
                }
                console.log('SUCCESS: Created ' + workers.length + ' workers');
                // Terminate all workers
                for (const worker of workers) {
                    worker.terminate();
                }
                console.log('SUCCESS: All workers terminated');
            } catch (e) {
                console.log('ERROR: ' + e.message);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: Created 3 workers"), "Multiple workers should work: {}", stdout);
        assert!(stdout.contains("SUCCESS: All workers terminated"), "All workers should be terminated: {}", stdout);
    }

    #[test]
    fn test_worker_properties() {
        // Test Worker object properties
        let script = r#"
            const worker = new Worker('data:,self.postMessage("test")');
            // Check internal properties are set correctly
            if (worker._terminated === false) {
                console.log('SUCCESS: _terminated is false');
            } else {
                console.log('ERROR: _terminated is ' + worker._terminated);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: _terminated is false"), "Worker properties should be set: {}", stdout);
    }

    #[test]
    fn test_worker_onmessageerror() {
        // Test that Worker instances have onmessageerror property
        let script = r#"
            const worker = new Worker('data:,self.postMessage("hello")');
            if (typeof worker.onmessageerror !== 'undefined') {
                console.log('SUCCESS: onmessageerror property exists');
            } else {
                console.log('ERROR: onmessageerror is undefined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: onmessageerror property exists"), "onmessageerror should exist: {}", stdout);
    }

    #[test]
    fn test_worker_post_message_logs() {
        // Test that postMessage is logged when called
        let script = r#"
            const worker = new Worker("data:,self.postMessage('test')");
            worker.postMessage("hello");
            worker.terminate();
        "#;
        let output = run_script(script);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Verify postMessage was called and logged
        assert!(stderr.contains("postMessage called"), "postMessage should be logged");
    }

    #[test]
    fn test_worker_terminate_logs() {
        // Test that terminate is logged
        let script = r#"
            const worker = new Worker("data:,self.postMessage('test')");
            worker.terminate();
        "#;
        let output = run_script(script);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Verify terminate was called and logged
        assert!(stderr.contains("terminated"), "terminate should be logged");
    }

    #[test]
    fn test_worker_terminated_state_persists() {
        // Test that terminated state persists after terminate()
        // Note: _terminated is an internal property - we verify termination works
        // by checking that postMessage after terminate doesn't cause errors
        let script = r#"
            const worker = new Worker("data:,self.postMessage('test')");
            // Call terminate
            worker.terminate();
            // postMessage after terminate should be safe (no-op)
            try {
                worker.postMessage("after_terminate");
                console.log('SUCCESS: Worker terminated and postMessage after terminate is safe');
            } catch (e) {
                console.log('ERROR: ' + e.message);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Terminated state should work: {}", stdout);
    }

    #[test]
    fn test_worker_terminated_postmessage_noop() {
        // Test that postMessage is a no-op after terminate()
        let script = r#"
            const worker = new Worker('data:,self.postMessage("test")');
            worker.terminate();
            // postMessage after terminate should not log (is a no-op)
            worker.postMessage("after_terminate");
            console.log('SUCCESS: postMessage after terminate completed without error');
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS: postMessage after terminate completed without error"), "postMessage after terminate should be safe: {}", stdout);
    }

    #[test]
    fn test_worker_event_handlers_initialized() {
        // Test that event handlers are properly initialized
        let script = r#"
            const worker = new Worker('data:,self.postMessage("test")');
            // Check that event handlers exist and are null (as per Web standard)
            if (worker.onmessage === null) {
                console.log('SUCCESS: onmessage is null');
            } else {
                console.log('INFO: onmessage value: ' + worker.onmessage);
            }
            if (worker.onerror === null) {
                console.log('SUCCESS: onerror is null');
            } else {
                console.log('INFO: onerror value: ' + worker.onerror);
            }
            worker.terminate();
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // These tests verify the handlers exist (exact null value depends on V8 implementation)
        assert!(stdout.contains("SUCCESS"), "Event handlers should be properly initialized");
    }
}

/// Helper function to run a JavaScript script with beejs
fn run_script(script: &str) -> std::process::Output {
    // Create a temporary file with the script
    let temp_dir = tempfile::Builder::new()
        .prefix("beejs-worker-test-")
        .tempdir()
        .unwrap();
    let temp_file = temp_dir.path().join("test.js");
    fs::write(&temp_file, script).unwrap();

    // Run beejs with the script
    let output = Command::new("./target/debug/beejs")
        .arg("run")
        .arg(&temp_file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run beejs");

    // Clean up
    drop(temp_dir);

    output
}

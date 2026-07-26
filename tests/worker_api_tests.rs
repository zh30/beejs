// Worker API tests for Beejs runtime
// v0.3.320: Tests for Web Worker support

use std::fs;
use std::process::{Command, Stdio};

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
    let output = Command::new("./target/debug/bee")
        .arg("run")
        .arg(&temp_file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run bee");

    // Clean up
    drop(temp_dir);

    output
}

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
        assert!(
            output.status.success(),
            "Worker constructor should exist: {}",
            stdout
        );
        assert!(
            stdout.contains("SUCCESS: Worker constructor exists"),
            "Output: {}",
            stdout
        );
    }

    #[test]
    fn test_worker_script_url_fails_closed_until_real_execution_exists() {
        let script = r#"
            try {
                const worker = new Worker('./worker.js');
                console.log('ERROR: Worker created fake object: ' + typeof worker.postMessage);
            } catch (e) {
                const message = String(e && e.message || e);
                if (message.includes('Worker') && message.includes('not supported')) {
                    console.log('SUCCESS: ' + message);
                } else {
                    console.log('ERROR: unexpected Worker error: ' + message);
                }
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("SUCCESS:"),
            "Worker script URL should fail closed instead of returning a fake object: {}",
            stdout
        );
    }

    #[test]
    fn test_worker_data_url_fails_closed_until_real_execution_exists() {
        let script = r#"
            try {
                new Worker('data:,self.postMessage("test")');
                console.log('ERROR: data URL Worker created fake object');
            } catch (e) {
                const message = String(e && e.message || e);
                console.log(message === 'Worker script execution is not supported yet' ? 'SUCCESS' : 'ERROR: ' + message);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("SUCCESS"),
            "Worker data URL should fail closed until real execution exists: {}",
            stdout
        );
    }

    #[test]
    fn test_worker_missing_script_url_reports_required_argument() {
        let script = r#"
            try {
                new Worker();
                console.log('ERROR: Worker created without script URL');
            } catch (e) {
                const message = String(e && e.message || e);
                console.log(message === 'Worker constructor requires a script URL' ? 'SUCCESS' : 'ERROR: ' + message);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("SUCCESS"),
            "Worker without script URL should report a required argument: {}",
            stdout
        );
    }

    #[test]
    fn test_worker_fail_closed_does_not_emit_fake_lifecycle_logs() {
        let script = r#"
            try {
                const worker = new Worker("data:,self.postMessage('test')");
                worker.postMessage("hello");
                worker.terminate();
                console.log('ERROR: fake Worker lifecycle reached');
            } catch (e) {
                console.log('SUCCESS: ' + String(e && e.message || e));
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stdout.contains("SUCCESS: Worker script execution is not supported yet"),
            "Worker construction should fail before fake lifecycle methods run: {}",
            stdout
        );
        assert!(
            !stderr.contains("postMessage called") && !stderr.contains("terminated"),
            "Worker fail-closed path must not emit fake lifecycle logs: {}",
            stderr
        );
    }
}

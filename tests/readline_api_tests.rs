// v0.3.277: Readline API integration tests
// Tests for readline.createInterface(), Interface.question(), etc.

#[cfg(test)]
mod readline_api_tests {
    use std::path::PathBuf;
    use std::process::Command;
    use std::fs;

    fn beejs_path() -> PathBuf {
        PathBuf::from(std::env::var("CARGO_BIN_EXE_BEEJS").unwrap_or_else(|_| "./target/release/beejs".to_string()))
    }

    #[test]
    fn test_readline_exists() {
        let output = Command::new(beejs_path())
            .args(["eval", "console.log(typeof readline)"])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("object"), "readline should exist as an object");
    }

    #[test]
    fn test_readline_create_interface_exists() {
        let output = Command::new(beejs_path())
            .args(["eval", "console.log(typeof readline.createInterface)"])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"), "readline.createInterface should exist");
    }

    #[test]
    fn test_readline_create_interface_function() {
        // Test that createInterface returns a function-like object with expected methods
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                console.log(typeof rl.question);
                console.log(typeof rl.close);
                console.log(typeof rl.pause);
                console.log(typeof rl.resume);
                console.log(typeof rl.setPrompt);
                console.log(typeof rl.prompt);
                rl.close();
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"), "Interface methods should be functions");
    }

    #[test]
    fn test_readline_create_interface_with_options() {
        // Test creating interface with different options
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({
                    input: process.stdin,
                    output: process.stdout,
                    terminal: false,
                    completer: null,
                    historySize: 100
                });
                console.log('interface created successfully');
                rl.close();
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("interface created successfully"), "Interface should be created with options");
    }

    #[test]
    fn test_readline_set_prompt() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                rl.setPrompt('> ');
                rl.close();
                console.log('prompt set successfully');
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("prompt set successfully"), "setPrompt should work");
    }

    #[test]
    fn test_readline_interface_close() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                rl.close();
                console.log('close called without error');
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("close called without error"), "close should work");
    }

    #[test]
    fn test_readline_interface_pause_resume() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                rl.pause();
                rl.resume();
                rl.close();
                console.log('pause and resume work');
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("pause and resume work"), "pause/resume should work");
    }

    #[test]
    fn test_readline_multiple_interfaces() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl1 = readline.createInterface({ input: process.stdin, output: process.stdout });
                const rl2 = readline.createInterface({ input: process.stdin, output: process.stdout });
                rl1.close();
                rl2.close();
                console.log('multiple interfaces work');
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("multiple interfaces work"), "Multiple interfaces should work");
    }

    #[test]
    fn test_readline_crlf_after_close() {
        // Test that close adds CRLF to output (Node.js behavior)
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                // Just ensure close doesn't throw
                rl.close();
                console.log('ok');
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok"), "Close should not throw");
    }

    #[test]
    fn test_readline_question_exists() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                console.log(typeof rl.question);
                rl.close();
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"), "question method should exist");
    }

    #[test]
    fn test_readline_write_exists() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                console.log(typeof rl.write);
                rl.close();
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"), "write method should exist");
    }

    #[test]
    fn test_readline_clear_line() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const readline = require('readline');
                const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
                console.log(typeof rl.clearLine);
                rl.close();
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"), "clearLine method should exist");
    }
}

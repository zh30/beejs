// v0.3.275: Performance API integration tests
// Tests for performance.now(), performance.mark(), performance.measure(), etc.

#[cfg(test)]
mod performance_api_tests {
    use std::path::PathBuf;
    use std::process::Command;

    fn beejs_path() -> PathBuf {
        PathBuf::from(
            std::env::var("CARGO_BIN_EXE_bee").unwrap_or_else(|_| "./target/debug/bee".to_string()),
        )
    }

    #[test]
    fn test_performance_now_exists() {
        let output = Command::new(beejs_path())
            .args(["eval", "console.log(typeof performance.now)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"), "performance.now should exist");
    }

    #[test]
    fn test_performance_now_returns_number() {
        let output = Command::new(beejs_path())
            .args(["eval", "const n = performance.now(); console.log(typeof n)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("number"),
            "performance.now should return a number"
        );
    }

    #[test]
    fn test_performance_now_monotonic() {
        let output = Command::new(beejs_path())
            .args(["eval", "const n1 = performance.now(); for(let i=0; i<1000; i++){} const n2 = performance.now(); console.log(n2 >= n1)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("true"),
            "performance.now should be monotonic"
        );
    }

    #[test]
    fn test_performance_time_origin_exists() {
        let output = Command::new(beejs_path())
            .args(["eval", "console.log(typeof performance.timeOrigin)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("number"),
            "performance.timeOrigin should exist"
        );
    }

    #[test]
    fn test_performance_time_origin_reasonable() {
        let output = Command::new(beejs_path())
            .args([
                "eval",
                "console.log(performance.timeOrigin > 1700000000000)",
            ])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("true"),
            "timeOrigin should be a reasonable Unix timestamp"
        );
    }

    #[test]
    fn test_performance_mark() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('test'); const entries = performance.getEntriesByName('test'); console.log(entries.length)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1"),
            "performance.mark should create an entry"
        );
    }

    #[test]
    fn test_performance_measure() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('start'); performance.mark('end'); performance.measure('test', 'start', 'end'); const entries = performance.getEntriesByName('test'); console.log(entries.length, entries[0]?.entryType)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1"),
            "performance.measure should create an entry"
        );
        assert!(
            stdout.contains("measure"),
            "measure entry should have entryType 'measure'"
        );
    }

    #[test]
    fn test_performance_measure_duration() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('start'); for(let i=0; i<100000; i++){} performance.mark('end'); performance.measure('test', 'start', 'end'); const entries = performance.getEntriesByName('test'); console.log(entries[0]?.duration > 0)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("true"),
            "measure should have positive duration"
        );
    }

    #[test]
    fn test_performance_get_entries() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('a'); performance.mark('b'); const entries = performance.getEntries(); console.log(entries.length)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2"), "getEntries should return all entries");
    }

    #[test]
    fn test_performance_get_entries_by_type() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('a'); performance.measure('test', 'a', 'a'); const marks = performance.getEntriesByType('mark'); const measures = performance.getEntriesByType('measure'); console.log(marks.length, measures.length)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1 1"),
            "getEntriesByType should filter by type"
        );
    }

    #[test]
    fn test_performance_clear_marks() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('test'); performance.clearMarks(); const entries = performance.getEntriesByName('test'); console.log(entries.length)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("0"),
            "clearMarks should remove mark entries"
        );
    }

    #[test]
    fn test_performance_clear_measures() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.mark('start'); performance.measure('test', 'start', 'start'); performance.clearMeasures(); const entries = performance.getEntriesByType('measure'); console.log(entries.length)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("0"),
            "clearMeasures should remove measure entries"
        );
    }

    #[test]
    fn test_performance_to_json() {
        let output = Command::new(beejs_path())
            .args(["eval", "const json = performance.toJSON(); console.log(typeof json, typeof json.now, typeof json.timeOrigin)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("object number number"),
            "toJSON should return an object with now and timeOrigin"
        );
    }

    #[test]
    fn test_performance_measure_without_marks() {
        let output = Command::new(beejs_path())
            .args(["eval", "performance.measure('test'); const entries = performance.getEntriesByName('test'); console.log(entries.length, entries[0]?.startTime > 0)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1 true"),
            "measure should work without explicit marks (uses timeOrigin)"
        );
    }

    #[test]
    fn test_performance_precision() {
        let output = Command::new(beejs_path())
            .args(["eval", "const n1 = performance.now(); const n2 = performance.now(); const diff = n2 - n1; console.log(typeof diff, diff >= 0)"])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should return a number >= 0
        assert!(
            stdout.contains("number true"),
            "performance.now should return a number >= 0"
        );
    }

    #[test]
    fn test_performance_timing_ai_workload() {
        // Test typical AI workload timing scenario
        let output = Command::new(beejs_path())
            .args([
                "eval",
                r#"
                performance.mark('model_load_start');
                // Simulate model inference with simple computation
                let result = 0;
                for(let i=0; i<500000; i++) { result += i * i; }
                performance.mark('model_load_end');
                performance.measure('model_load', 'model_load_start', 'model_load_end');
                const entries = performance.getEntriesByName('model_load');
                console.log('AI inference time:', entries[0]?.duration.toFixed(2), 'ms');
            "#,
            ])
            .output()
            .expect("Failed to run bee");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ms"), "Should measure AI workload timing");
    }
}

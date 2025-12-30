// structuredClone Tests for Beejs
// Tests for v0.3.299: structuredClone global function
// Enables deep cloning of objects for AI workloads

#[cfg(test)]
mod structured_clone_tests {
    use std::path::PathBuf;
    use std::process::Command;

    fn beejs_path() -> PathBuf {
        PathBuf::from(std::env::var("CARGO_BIN_EXE_BEEJS").unwrap_or_else(|_| "./target/debug/beejs".to_string()))
    }

    /// Test 1: structuredClone with null
    #[test]
    fn test_clone_null() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const result = structuredClone(null);
                console.log('null result:', result === null);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("null result: true"), "Expected null to clone to null. Got: {}", stdout);
    }

    /// Test 2: structuredClone with undefined
    #[test]
    fn test_clone_undefined() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const result = structuredClone(undefined);
                console.log('undefined result:', result === undefined);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("undefined result: true"), "Expected undefined to clone to undefined. Got: {}", stdout);
    }

    /// Test 3: structuredClone with string
    #[test]
    fn test_clone_string() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = "Hello, Beejs!";
                const cloned = structuredClone(original);
                console.log('string result:', cloned === original);
                console.log('string value:', cloned);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("string result: true"), "Expected string to clone correctly. Got: {}", stdout);
        assert!(stdout.contains("string value: Hello, Beejs!"), "Expected correct string value. Got: {}", stdout);
    }

    /// Test 4: structuredClone with number
    #[test]
    fn test_clone_number() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const result = structuredClone(42);
                console.log('number result:', result === 42);
                console.log('number value:', result);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("number result: true"), "Expected number to clone correctly. Got: {}", stdout);
    }

    /// Test 5: structuredClone with boolean
    #[test]
    fn test_clone_boolean() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const result = structuredClone(true);
                console.log('boolean result:', result === true);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("boolean result: true"), "Expected boolean to clone correctly. Got: {}", stdout);
    }

    /// Test 6: structuredClone with plain object
    #[test]
    fn test_clone_plain_object() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = { name: "Beejs", version: "0.3.299" };
                const cloned = structuredClone(original);
                console.log('object cloned:', cloned !== original);
                console.log('object name:', cloned.name);
                console.log('object version:', cloned.version);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("object cloned: true"), "Expected object to be cloned. Got: {}", stdout);
        assert!(stdout.contains("object name: Beejs"), "Expected name to be preserved. Got: {}", stdout);
        assert!(stdout.contains("object version: 0.3.299"), "Expected version to be preserved. Got: {}", stdout);
    }

    /// Test 7: structuredClone with array
    #[test]
    fn test_clone_array() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = [1, 2, 3, "four", true];
                const cloned = structuredClone(original);
                console.log('array cloned:', cloned !== original);
                console.log('array length:', cloned.length);
                console.log('array is array:', Array.isArray(cloned));
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("array cloned: true"), "Expected array to be cloned. Got: {}", stdout);
        assert!(stdout.contains("array length: 5"), "Expected array length to be preserved. Got: {}", stdout);
        assert!(stdout.contains("array is array: true"), "Expected to still be an array. Got: {}", stdout);
    }

    /// Test 8: structuredClone with nested object
    #[test]
    fn test_clone_nested_object() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = {
                    user: {
                        name: "Alice",
                        scores: [95, 87, 92]
                    }
                };
                const cloned = structuredClone(original);
                console.log('nested cloned:', cloned !== original);
                console.log('nested user:', cloned.user !== original.user);
                console.log('nested name:', cloned.user.name);
                console.log('nested scores length:', cloned.user.scores.length);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("nested cloned: true"), "Expected nested object to be cloned. Got: {}", stdout);
        assert!(stdout.contains("nested user: true"), "Expected nested user to be different reference. Got: {}", stdout);
        assert!(stdout.contains("nested name: Alice"), "Expected nested name to be preserved. Got: {}", stdout);
    }

    /// Test 9: structuredClone with empty object
    #[test]
    fn test_clone_empty_object() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = {};
                const cloned = structuredClone(original);
                console.log('empty cloned:', cloned !== original);
                console.log('empty keys:', Object.keys(cloned).length);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("empty cloned: true"), "Expected empty object to be cloned. Got: {}", stdout);
        assert!(stdout.contains("empty keys: 0"), "Expected empty object to have no keys. Got: {}", stdout);
    }

    /// Test 10: structuredClone with empty array
    #[test]
    fn test_clone_empty_array() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = [];
                const cloned = structuredClone(original);
                console.log('empty array cloned:', cloned !== original);
                console.log('empty array length:', cloned.length);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("empty array cloned: true"), "Expected empty array to be cloned. Got: {}", stdout);
        assert!(stdout.contains("empty array length: 0"), "Expected empty array to have length 0. Got: {}", stdout);
    }

    /// Test 11: structuredClone creates deep copy (no shared references)
    #[test]
    fn test_clone_creates_deep_copy() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = { data: [1, 2, 3] };
                const cloned = structuredClone(original);

                // Modify original
                original.data.push(4);
                original.data[0] = 99;

                console.log('deep copy works:', cloned.data[0] === 1);
                console.log('cloned length:', cloned.data.length);
                console.log('original length:', original.data.length);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("deep copy works: true"), "Expected deep copy (no shared refs). Got: {}", stdout);
        assert!(stdout.contains("cloned length: 3"), "Expected cloned array unchanged. Got: {}", stdout);
        assert!(stdout.contains("original length: 4"), "Expected original array modified. Got: {}", stdout);
    }

    /// Test 12: structuredClone with object containing multiple types
    #[test]
    fn test_clone_mixed_types() {
        let output = Command::new(beejs_path())
            .args(["eval", r#"
                const original = {
                    string: "hello",
                    number: 42,
                    float: 3.14159,
                    boolTrue: true,
                    boolFalse: false,
                    nullVal: null,
                    array: [1, "two", true]
                };
                const cloned = structuredClone(original);
                console.log('mixed cloned:', cloned !== original);
                console.log('mixed string:', cloned.string);
                console.log('mixed number:', cloned.number);
                console.log('mixed float:', cloned.float);
                console.log('mixed boolTrue:', cloned.boolTrue);
                console.log('mixed boolFalse:', cloned.boolFalse);
                console.log('mixed null:', cloned.nullVal === null);
                console.log('mixed array length:', cloned.array.length);
            "#])
            .output()
            .expect("Failed to run beejs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("mixed cloned: true"), "Expected mixed object to be cloned. Got: {}", stdout);
        assert!(stdout.contains("mixed string: hello"), "Expected string preserved. Got: {}", stdout);
        assert!(stdout.contains("mixed number: 42"), "Expected number preserved. Got: {}", stdout);
        assert!(stdout.contains("mixed null: true"), "Expected null preserved. Got: {}", stdout);
    }
}

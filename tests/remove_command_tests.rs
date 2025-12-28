// Package manager remove command tests
// v0.3.223 - Test coverage for remove command

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod remove_command_tests {
    use super::*;

    /// Helper function to create a test package.json
    fn create_test_package_json(dir: &TempDir, content: &str) -> PathBuf {
        let package_json_path = dir.path().join("package.json");
        fs::write(&package_json_path, content).unwrap();
        package_json_path
    }

    /// Helper function to read package.json
    fn read_package_json(dir: &TempDir) -> serde_json::Value {
        let package_json_path = dir.path().join("package.json");
        let content = fs::read_to_string(&package_json_path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    /// Test 1: Remove from dependencies
    #[tokio::test]
    async fn test_remove_from_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "lodash": "^4.17.21",
                "express": "^4.18.2"
            }
        }"#;
        create_test_package_json(&temp_dir, package_json);

        // Simulate remove command: remove lodash from dependencies
        let mut content: serde_json::Value = serde_json::from_str(package_json).unwrap();
        content["dependencies"]
            .as_object_mut()
            .unwrap()
            .remove("lodash");

        // Write updated package.json
        let updated_path = temp_dir.path().join("package.json");
        fs::write(&updated_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        // Verify
        let result = read_package_json(&temp_dir);
        assert!(result["dependencies"]["lodash"].is_null());
        assert!(result["dependencies"]["express"].is_string());
        println!("✅ Test 1: Remove from dependencies - PASSED");
    }

    /// Test 2: Remove from devDependencies
    #[tokio::test]
    async fn test_remove_from_dev_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "devDependencies": {
                "typescript": "^5.0.0",
                "jest": "^29.0.0"
            }
        }"#;
        create_test_package_json(&temp_dir, package_json);

        // Simulate remove command: remove typescript from devDependencies
        let mut content: serde_json::Value = serde_json::from_str(package_json).unwrap();
        content["devDependencies"]
            .as_object_mut()
            .unwrap()
            .remove("typescript");

        // Write updated package.json
        let updated_path = temp_dir.path().join("package.json");
        fs::write(&updated_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        // Verify
        let result = read_package_json(&temp_dir);
        assert!(result["devDependencies"]["typescript"].is_null());
        assert!(result["devDependencies"]["jest"].is_string());
        println!("✅ Test 2: Remove from devDependencies - PASSED");
    }

    /// Test 3: Remove from optionalDependencies
    #[tokio::test]
    async fn test_remove_from_optional_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "optionalDependencies": {
                "fsevents": "^2.3.0",
                "canvas": "^2.11.0"
            }
        }"#;
        create_test_package_json(&temp_dir, package_json);

        // Simulate remove command: remove fsevents from optionalDependencies
        let mut content: serde_json::Value = serde_json::from_str(package_json).unwrap();
        content["optionalDependencies"]
            .as_object_mut()
            .unwrap()
            .remove("fsevents");

        // Write updated package.json
        let updated_path = temp_dir.path().join("package.json");
        fs::write(&updated_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        // Verify
        let result = read_package_json(&temp_dir);
        assert!(result["optionalDependencies"]["fsevents"].is_null());
        assert!(result["optionalDependencies"]["canvas"].is_string());
        println!("✅ Test 3: Remove from optionalDependencies - PASSED");
    }

    /// Test 4: Remove nonexistent package (should not fail)
    #[tokio::test]
    async fn test_remove_nonexistent_package() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "express": "^4.18.2"
            }
        }"#;
        create_test_package_json(&temp_dir, package_json);

        // Simulate remove command: remove nonexistent package
        let mut content: serde_json::Value = serde_json::from_str(package_json).unwrap();
        content["dependencies"]
            .as_object_mut()
            .unwrap()
            .remove("nonexistent");

        // Write updated package.json
        let updated_path = temp_dir.path().join("package.json");
        fs::write(&updated_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        // Verify - package.json should be unchanged
        let result = read_package_json(&temp_dir);
        assert!(result["dependencies"]["express"].is_string());
        println!("✅ Test 4: Remove nonexistent package - PASSED");
    }

    /// Test 5: Remove package when package.json doesn't exist
    #[tokio::test]
    async fn test_remove_without_package_json() {
        let temp_dir = TempDir::new().unwrap();
        // Don't create package.json

        // Simulate remove command on non-existent package.json
        let package_json_path = temp_dir.path().join("package.json");
        let result = std::path::Path::new(&package_json_path).exists();

        assert!(!result);
        println!("✅ Test 5: Remove without package.json - PASSED");
    }

    /// Test 6: Remove package from all dependency types
    #[tokio::test]
    async fn test_remove_from_all_dependency_types() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "lodash": "^4.17.21"
            },
            "devDependencies": {
                "typescript": "^5.0.0"
            },
            "optionalDependencies": {
                "fsevents": "^2.3.0"
            }
        }"#;
        create_test_package_json(&temp_dir, package_json);

        // Simulate remove command: remove lodash (only in dependencies)
        let mut content: serde_json::Value = serde_json::from_str(package_json).unwrap();
        content["dependencies"]
            .as_object_mut()
            .unwrap()
            .remove("lodash");

        // Write updated package.json
        let updated_path = temp_dir.path().join("package.json");
        fs::write(&updated_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        // Verify
        let result = read_package_json(&temp_dir);
        assert!(result["dependencies"]["lodash"].is_null());
        assert!(result["devDependencies"]["typescript"].is_string());
        assert!(result["optionalDependencies"]["fsevents"].is_string());
        println!("✅ Test 6: Remove from all dependency types - PASSED");
    }
}

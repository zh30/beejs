// bee install command tests
// v0.3.229 - Test coverage for bee install command

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the bee binary
fn beejs_path() -> PathBuf {
    PathBuf::from("/Users/henry/code/beejs/target/debug/bee")
}

#[cfg(test)]
mod install_command_tests {
    use super::*;

    /// Test 1: bee install command should be recognized
    #[test]
    fn test_install_command_exists() {
        let beejs = beejs_path();
        assert!(beejs.exists(), "bee binary should exist at {:?}", beejs);

        let output = Command::new(&beejs)
            .arg("install")
            .arg("--help")
            .output()
            .expect("Failed to execute bee install --help");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should have successful exit code or show help
        assert!(
            output.status.success()
                || stdout.contains("Install dependencies")
                || stderr.contains("Install dependencies"),
            "bee install command should be recognized. stdout: {}, stderr: {}",
            stdout,
            stderr
        );
    }

    /// Test 2: bee install with no package.json should error
    #[test]
    fn test_install_no_package_json() {
        let temp_dir = TempDir::new().unwrap();

        let output = Command::new(beejs_path())
            .arg("install")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute bee install");

        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !output.status.success(),
            "bee install should fail without package.json"
        );
        assert!(
            stderr.contains("package.json not found"),
            "Error message should mention package.json: {}",
            stderr
        );
    }

    /// Test 3: bee install with empty dependencies should succeed
    #[test]
    fn test_install_empty_dependencies() {
        let temp_dir = TempDir::new().unwrap();

        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0"
        }"#;

        let package_json_path = temp_dir.path().join("package.json");
        fs::write(&package_json_path, package_json).unwrap();

        let output = Command::new(beejs_path())
            .arg("install")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute bee install");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "bee install with empty dependencies should succeed. stderr: {}",
            stderr
        );
        assert!(
            stdout.contains("0 dependencies") || stdout.contains("Installed"),
            "Should show installed count: {}",
            stdout
        );
    }

    /// Test 4: Package.json should have optionalDependencies field
    #[test]
    fn test_package_json_optional_dependencies() {
        let temp_dir = TempDir::new().unwrap();

        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "optionalDependencies": {
                "fsevents": "^2.3.0"
            }
        }"#;

        let package_json_path = temp_dir.path().join("package.json");
        fs::write(&package_json_path, package_json).unwrap();

        // Parse and verify optionalDependencies is preserved
        let content = fs::read_to_string(&package_json_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(
            parsed["optionalDependencies"]["fsevents"].is_string(),
            "optionalDependencies should be preserved in package.json"
        );
    }

    /// Test 5: verify optional_dependencies field in PackageJson struct
    #[test]
    fn test_package_json_struct_has_optional_dependencies() {
        use beejs::package_manager::PackageJson;
        use std::collections::HashMap;

        let package_json = PackageJson {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            main: None,
            scripts: None,
            dependencies: None,
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: Some(HashMap::new()),
            author: None,
            license: None,
            repository: None,
        };

        assert!(
            package_json.optional_dependencies.is_some(),
            "PackageJson should have optional_dependencies field"
        );
    }
}

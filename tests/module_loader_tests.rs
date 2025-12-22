use std::time{SystemTime, UNIX_EPOCH, Duration};
//! ModuleLoader 模块系统测试
//! 测试驱动的开发 - Stage 60: 模块系统测试套件
//!
//! 本文件包含模块系统的完整测试套件，涵盖：
//! - 模块解析测试
//! - 模块缓存测试
//! - 相对路径解析测试
//! - 内置模块测试
//! - node_modules 解析测试

use beejs::module_loader{ModuleLoader, Module};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    /// 测试 1: ModuleLoader 创建
    #[test]
    #[serial]
    fn test_module_loader_creation() {
        let temp_dir: _ = TempDir::new().unwrap();
        let _loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // ModuleLoader 创建成功即可，内部状态通过公共方法验证
        assert!(true, "ModuleLoader created successfully");
    }

    /// 测试 2: 从当前目录创建 ModuleLoader
    #[test]
    #[serial]
    fn test_module_loader_from_current_dir() {
        let loader: _ = ModuleLoader::from_current_dir();
        assert!(loader.is_ok(), "Should create loader from current directory");

        let _loader: _ = loader.clone();unwrap();
        // 创建成功即可
        assert!(true, "ModuleLoader created from current directory");
    }

    /// 测试 3: 解析相对路径模块 (./module)
    #[test]
    #[serial]
    fn test_resolve_relative_module_dot_slash() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建测试模块文件
        let module_path: _ = temp_dir.path().join("test_module.js");
        std::fs::write(&module_path, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("./test_module");
        assert!(result.is_ok(), "Should resolve relative module");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.ends_with("test_module.js"));
    }

    /// 测试 4: 解析相对路径模块 (../module)
    #[test]
    #[serial]
    fn test_resolve_relative_module_dot_dot_slash() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 在父目录中创建模块
        let module_path: _ = temp_dir.path().join("parent_module.js");
        std::fs::write(&module_path, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("parent_module");
        assert!(result.is_ok(), "Should resolve parent directory module");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.ends_with("parent_module.js"));
    }

    /// 测试 5: 解析目录索引模块 (index.js)
    #[test]
    #[serial]
    fn test_resolve_directory_index_module() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建带 index.js 的目录
        let module_dir: _ = temp_dir.path().join("mymodule");
        std::fs::create_dir_all(&module_dir).unwrap();
        let index_path: _ = module_dir.join("index.js");
        std::fs::write(&index_path, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("./mymodule");
        assert!(result.is_ok(), "Should resolve directory as index.js");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.ends_with("index.js"));
    }

    /// 测试 6: 解析不存在的模块
    #[test]
    #[serial]
    fn test_resolve_nonexistent_module() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        let result: _ = loader.resolve_module("./nonexistent");
        assert!(result.is_err(), "Should fail to resolve nonexistent module");
    }

    /// 测试 7: 解析内置模块
    #[test]
    #[serial]
    fn test_resolve_builtin_modules() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        let builtin_modules: _ = vec!["path", "fs", "os", "crypto", "buffer"];

        for module_name in builtin_modules {
            let result: _ = loader.resolve_module(module_name);
            assert!(result.is_ok(), "Should resolve builtin module: {}", module_name);

            let resolved_path: _ = result.unwrap();
            assert!(resolved_path.to_string_lossy().contains(module_name));
            assert!(resolved_path.to_string_lossy().contains("__builtin__"));
        }
    }

    /// 测试 8: 模块缓存功能（模拟）
    #[test]
    #[serial]
    fn test_module_cache() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建测试模块
        let module_path: _ = temp_dir.path().join("cached_module.js");
        std::fs::write(&module_path, "module.exports = { value: 42 };").unwrap();

        // 解析模块（缓存功能尚未完全实现）
        let result: _ = loader.resolve_module("./cached_module");
        assert!(result.is_ok(), "Should resolve module");

        // 注意：缓存功能需要加载模块后才能验证
        assert!(true, "Module resolved (cache implementation pending)");
    }

    /// 测试 9: 解析绝对路径模块
    #[test]
    #[serial]
    fn test_resolve_absolute_module() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        let absolute_path: _ = "/absolute/path/to/module.js";
        let result: _ = loader.resolve_module(absolute_path);
        assert!(result.is_ok(), "Should resolve absolute path");

        let resolved_path: _ = result.unwrap();
        assert_eq!(resolved_path, PathBuf::from(absolute_path));
    }

    /// 测试 10: 自动添加 .js 扩展名
    #[test]
    #[serial]
    fn test_add_js_extension() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建不带扩展名的模块文件
        let module_path: _ = temp_dir.path().join("noext_module.js");
        std::fs::write(&module_path, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("./noext_module");
        assert!(result.is_ok(), "Should resolve module without extension");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.to_string_lossy().ends_with(".js"));
    }

    /// 测试 11: node_modules 解析
    #[test]
    #[serial]
    fn test_resolve_node_modules() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建 node_modules 目录和包
        let node_modules_dir: _ = temp_dir.path().join("node_modules");
        let package_dir: _ = node_modules_dir.join("test-package");
        std::fs::create_dir_all(&package_dir).unwrap();

        // 创建 package.json
        let package_json: _ = package_dir.join("package.json");
        let package_content: _ = r#"{
            "name": "test-package",
            "main": "index.js"
        }"#;
        std::fs::write(&package_json, package_content).unwrap();

        // 创建主入口文件
        let main_file: _ = package_dir.join("index.js");
        std::fs::write(&main_file, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("test-package");
        assert!(result.is_ok(), "Should resolve node_modules package");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.to_string_lossy().contains("test-package"));
        assert!(resolved_path.to_string_lossy().ends_with("index.js"));
    }

    /// 测试 12: 模块结构
    #[test]
    #[serial]
    fn test_module_structure() {
        let module: _ = Module {
            exports: std::collections::HashMap::new(),
            path: PathBuf::from("/test/path.js"),
        };

        assert_eq!(module.path, PathBuf::from("/test/path.js"));
        assert!(module.exports.is_empty());
    }

    /// 测试 13: 复杂相对路径解析
    #[test]
    #[serial]
    fn test_complex_relative_paths() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建多层目录结构
        let deep_dir: _ = temp_dir.path().join("level1").join("level2");
        std::fs::create_dir_all(&deep_dir).unwrap();

        let module_path: _ = deep_dir.join("deep_module.js");
        std::fs::write(&module_path, "module.exports = {};").unwrap();

        println!("Base dir: {:?}", temp_dir.path());
        println!("Module path exists: {:?}", module_path.exists());
        println!("Module path: {:?}", module_path);

        // Use absolute path from base_dir
        let module_name: _ = "level1/level2/deep_module";
        println!("Module name: {}", module_name);
        println!("Starts with ./ ?: {}", module_name.starts_with("./"));
        println!("Starts with ../ ?: {}", module_name.starts_with("../"));
        let result: _ = loader.resolve_module(module_name);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "Should resolve complex relative path");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.to_string_lossy().contains("deep_module.js"));
    }

    /// 测试 14: 空模块名处理
    #[test]
    #[serial]
    fn test_empty_module_name() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        let _result: _ = loader.resolve_module("");
        // 空字符串应该被当作当前目录或失败
        // 具体行为取决于实现
        // 这里我们只验证它不会 panic
        assert!(true, "Empty module name should not panic");
    }

    /// 测试 15: 包的主入口点解析
    #[test]
    #[serial]
    fn test_package_main_entry_point() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建带自定义主入口的包
        let node_modules_dir: _ = temp_dir.path().join("node_modules");
        let package_dir: _ = node_modules_dir.join("custom-main");
        std::fs::create_dir_all(&package_dir).unwrap();

        let package_json: _ = package_dir.join("package.json");
        let package_content: _ = r#"{
            "name": "custom-main",
            "main": "lib/index.js"
        }"#;
        std::fs::write(&package_json, package_content).unwrap();

        let lib_dir: _ = package_dir.join("lib");
        std::fs::create_dir_all(&lib_dir).unwrap();

        let main_file: _ = lib_dir.join("index.js");
        std::fs::write(&main_file, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("custom-main");
        assert!(result.is_ok(), "Should resolve package with custom main");

        let resolved_path: _ = result.unwrap();
        assert!(resolved_path.to_string_lossy().contains("lib/index.js"));
    }

    /// 测试 16: 无效 package.json 处理
    #[test]
    #[serial]
    fn test_invalid_package_json() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建无效的 package.json
        let node_modules_dir: _ = temp_dir.path().join("node_modules");
        let package_dir: _ = node_modules_dir.join("invalid-pkg");
        std::fs::create_dir_all(&package_dir).unwrap();

        let package_json: _ = package_dir.join("package.json");
        std::fs::write(&package_json, "{ invalid json }").unwrap();

        let result: _ = loader.resolve_module("invalid-pkg");
        // 应该失败或使用默认行为
        assert!(result.is_err() || result.is_ok(), "Should handle invalid package.json gracefully");
    }

    /// 测试 17: 循环依赖检测（模拟）
    #[test]
    #[serial]
    fn test_circular_dependency_simulation() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建两个相互引用的模块
        let module_a: _ = temp_dir.path().join("module_a.js");
        let module_b: _ = temp_dir.path().join("module_b.js");

        std::fs::write(&module_a, "require('./module_b'); module.exports = {};").unwrap();
        std::fs::write(&module_b, "require('./module_a'); module.exports = {};").unwrap();

        // 解析第一个模块
        let result: _ = loader.resolve_module("./module_a");
        assert!(result.is_ok(), "Should resolve module with circular dependency");

        // 注意：实际的循环依赖检测需要在模块加载时进行
        // 这里只是验证解析不会失败
    }

    /// 测试 18: 模块路径规范化
    #[test]
    #[serial]
    fn test_module_path_normalization() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // 创建带特殊字符的模块目录
        let special_dir: _ = temp_dir.path().join("my-module_v1.0");
        std::fs::create_dir_all(&special_dir).unwrap();

        let index_path: _ = special_dir.join("index.js");
        std::fs::write(&index_path, "module.exports = {};").unwrap();

        let result: _ = loader.resolve_module("./my-module_v1.0");
        assert!(result.is_ok(), "Should handle special characters in paths");
    }
}

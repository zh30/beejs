use beejs::nodejs_core::commonjs_resolver::{resolve_commonjs_module, ResolvedModule};
use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;
use std::fs;

fn path_for_js(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

fn resolved_file_path(module: ResolvedModule) -> std::path::PathBuf {
    match module {
        ResolvedModule::File(path) => path,
        ResolvedModule::Builtin(name) => panic!("expected file module, got builtin {name}"),
    }
}

fn resolved_builtin_name(module: ResolvedModule) -> String {
    match module {
        ResolvedModule::Builtin(name) => name,
        ResolvedModule::File(path) => {
            panic!("expected builtin module, got file {}", path.display())
        }
    }
}

#[test]
fn resolves_node_prefix_builtin_module() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();

    let resolved = resolved_builtin_name(resolve_commonjs_module("node:path", &app_dir).unwrap());

    assert_eq!(resolved, "path");
}

#[test]
fn resolves_relative_module_with_js_extension() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    let util_path = app_dir.join("util.js");
    fs::write(&util_path, "exports.value = 42;").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("./util", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(util_path).unwrap());
}

#[test]
fn resolves_relative_typescript_module_extension() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    let typed_path = app_dir.join("typed.ts");
    fs::write(
        &typed_path,
        "const answer: number = 42; module.exports = { answer };",
    )
    .unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("./typed", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(typed_path).unwrap());
}

#[test]
fn resolves_directory_index_module() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("lib");
    fs::create_dir_all(&package_dir).unwrap();
    let index_path = package_dir.join("index.js");
    fs::write(&index_path, "exports.value = 7;").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("./lib", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(index_path).unwrap());
}

#[test]
fn resolves_dot_as_current_directory_package() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("package.json"),
        r#"{"name":"app","main":"src/main.js"}"#,
    )
    .unwrap();
    let main_path = app_dir.join("src/main.js");
    fs::write(&main_path, "module.exports = { value: 1 };").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module(".", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(main_path).unwrap());
}

#[test]
fn resolves_dot_dot_as_parent_directory_package() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let nested_dir = app_dir.join("src");
    fs::create_dir_all(&nested_dir).unwrap();
    let index_path = app_dir.join("index.js");
    fs::write(&index_path, "exports.value = 2;").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("..", &nested_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(index_path).unwrap());
}

#[test]
fn resolves_package_json_main_from_node_modules() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","main":"dist/main.js"}"#,
    )
    .unwrap();
    let main_path = package_dir.join("dist/main.js");
    fs::write(&main_path, "module.exports = { answer: 42 };").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("pkg", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(main_path).unwrap());
}

#[test]
fn resolves_package_json_exports_string_before_main() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":"./dist/cjs.js","main":"legacy.js"}"#,
    )
    .unwrap();
    let exports_path = package_dir.join("dist/cjs.js");
    fs::write(&exports_path, "module.exports = { answer: 2026 };").unwrap();
    fs::write(
        package_dir.join("legacy.js"),
        "module.exports = { answer: 1 };",
    )
    .unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("pkg", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(exports_path).unwrap());
}

#[test]
fn resolves_package_json_conditional_exports_for_require() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{".":{"default":"./dist/esm.js","node":"./dist/node.js","require":"./dist/cjs.js"}}}"#,
    )
    .unwrap();
    let cjs_path = package_dir.join("dist/cjs.js");
    fs::write(&cjs_path, "module.exports = { mode: 'cjs' };").unwrap();
    fs::write(
        package_dir.join("dist/node.js"),
        "module.exports = { mode: 'node' };",
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/esm.js"),
        "module.exports = { mode: 'esm' };",
    )
    .unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("pkg", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(cjs_path).unwrap());
}

#[test]
fn resolves_package_json_subpath_exports_string() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{"./feature":"./dist/feature.js"}}"#,
    )
    .unwrap();
    let feature_path = package_dir.join("dist/feature.js");
    fs::write(&feature_path, "module.exports = { feature: true };").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("pkg/feature", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(feature_path).unwrap());
}

#[test]
fn resolves_package_json_subpath_exports_pattern() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist/features")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{"./features/*":"./dist/features/*.js"}}"#,
    )
    .unwrap();
    let feature_path = package_dir.join("dist/features/button.js");
    fs::write(&feature_path, "module.exports = { feature: 'button' };").unwrap();

    let resolved =
        resolved_file_path(resolve_commonjs_module("pkg/features/button", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(feature_path).unwrap());
}

#[test]
fn package_json_exports_blocks_unexported_subpath_with_specific_error() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{"./public":"./public.js"}}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("public.js"),
        "module.exports = { public: true };",
    )
    .unwrap();
    fs::write(
        package_dir.join("private.js"),
        "module.exports = { private: true };",
    )
    .unwrap();

    let error = resolve_commonjs_module("pkg/private", &app_dir)
        .expect_err("unexported package subpath must be rejected");
    let message = error.to_string();

    assert!(
        message.contains("ERR_PACKAGE_PATH_NOT_EXPORTED"),
        "expected package exports error, got: {message}"
    );
    assert!(
        message.contains("./private"),
        "error should identify the blocked subpath, got: {message}"
    );
}

#[test]
fn resolves_scoped_package_json_subpath_exports_string() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/@scope/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"@scope/pkg","exports":{"./feature":"./dist/feature.js"}}"#,
    )
    .unwrap();
    let feature_path = package_dir.join("dist/feature.js");
    fs::write(&feature_path, "module.exports = { feature: true };").unwrap();

    let resolved =
        resolved_file_path(resolve_commonjs_module("@scope/pkg/feature", &app_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(feature_path).unwrap());
}

#[test]
fn resolves_node_modules_by_walking_parent_directories() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let nested_dir = app_dir.join("src/features");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(&nested_dir).unwrap();
    fs::create_dir_all(&package_dir).unwrap();
    let index_path = package_dir.join("index.js");
    fs::write(&index_path, "exports.answer = 42;").unwrap();

    let resolved = resolved_file_path(resolve_commonjs_module("pkg", &nested_dir).unwrap());

    assert_eq!(resolved, fs::canonicalize(index_path).unwrap());
}

#[test]
#[serial]
fn runtime_require_resolves_directory_index() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let lib_dir = app_dir.join("lib");
    fs::create_dir_all(&lib_dir).unwrap();
    fs::write(lib_dir.join("index.js"), "exports.value = 99;").unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("./lib").value;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "99");
}

#[test]
#[serial]
fn runtime_require_resolves_package_main() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","main":"dist/main.js"}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/main.js"),
        "module.exports = { answer: 123 };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("pkg").answer;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "123");
}

#[test]
#[serial]
fn runtime_require_resolves_package_exports_string_before_main() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":"./dist/cjs.js","main":"legacy.js"}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/cjs.js"),
        "module.exports = { answer: 2026 };",
    )
    .unwrap();
    fs::write(
        package_dir.join("legacy.js"),
        "module.exports = { answer: 1 };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("pkg").answer;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "2026");
}

#[test]
#[serial]
fn runtime_require_resolves_package_conditional_exports_for_require() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{".":{"default":"./dist/esm.js","node":"./dist/node.js","require":"./dist/cjs.js"}}}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/cjs.js"),
        "module.exports = { mode: 'cjs' };",
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/node.js"),
        "module.exports = { mode: 'node' };",
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/esm.js"),
        "module.exports = { mode: 'esm' };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("pkg").mode;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "cjs");
}

#[test]
#[serial]
fn runtime_require_resolves_package_subpath_exports_string() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{"./feature":"./dist/feature.js"}}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/feature.js"),
        "module.exports = { answer: 808 };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("pkg/feature").answer;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "808");
}

#[test]
#[serial]
fn runtime_require_resolves_package_subpath_exports_pattern() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(package_dir.join("dist/features")).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{"./features/*":"./dist/features/*.js"}}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("dist/features/button.js"),
        "module.exports = { feature: 'button' };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("pkg/features/button").feature;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "button");
}

#[test]
#[serial]
fn runtime_require_reports_unexported_package_subpath() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let package_dir = app_dir.join("node_modules/pkg");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"pkg","exports":{"./public":"./public.js"}}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("public.js"),
        "module.exports = { public: true };",
    )
    .unwrap();
    fs::write(
        package_dir.join("private.js"),
        "module.exports = { private: true };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        try {{
            require("pkg/private");
            "loaded";
        }} catch (error) {{
            String(error && error.message || error);
        }}
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert!(
        result.contains("ERR_PACKAGE_PATH_NOT_EXPORTED"),
        "runtime require should report exports error, got: {result}"
    );
}

#[test]
#[serial]
fn runtime_module_require_uses_module_directory_not_global_dirname() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    let lib_dir = app_dir.join("lib");
    fs::create_dir_all(&lib_dir).unwrap();
    fs::write(lib_dir.join("child.js"), "exports.value = 456;").unwrap();
    fs::write(
        lib_dir.join("index.js"),
        r#"
        globalThis.__dirname = "/";
        module.exports = require("./child");
        "#,
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("./lib").value;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "456");
}

#[test]
#[serial]
fn runtime_require_resolve_returns_resolved_file_path() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    let util_path = app_dir.join("util.js");
    fs::write(&util_path, "exports.value = 1;").unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require.resolve("./util");
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(
        result.trim(),
        fs::canonicalize(util_path).unwrap().to_string_lossy()
    );
}

#[test]
#[serial]
fn runtime_require_loads_typescript_module() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    fs::write(
        app_dir.join("typed.ts"),
        "const answer: number = 42; module.exports = { answer };",
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        require("./typed").answer;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "42");
}

#[test]
#[serial]
fn runtime_require_loads_json_module_as_object() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("app");
    fs::create_dir_all(&app_dir).unwrap();
    fs::write(
        app_dir.join("data.json"),
        r#"{"answer":42,"nested":{"ok":true},"items":[1,2,3]}"#,
    )
    .unwrap();

    let mut runtime = MinimalRuntime::new().unwrap();
    let code = format!(
        r#"
        globalThis.__dirname = "{}";
        const first = require("./data");
        const second = require("./data.json");
        first.answer === 42 &&
          first.nested.ok === true &&
          first.items.length === 3 &&
          first === second;
        "#,
        path_for_js(&app_dir)
    );

    let result = runtime.execute_code(&code).unwrap();

    assert_eq!(result.trim(), "true");
}

#[test]
#[serial]
fn runtime_require_supports_node_prefix_for_builtin_modules() {
    let mut runtime = MinimalRuntime::new().unwrap();

    let result = runtime
        .execute_code(
            r#"
            const pathFromNodePrefix = require("node:path");
            const pathFromPlainName = require("path");
            pathFromNodePrefix.sep === pathFromPlainName.sep &&
              pathFromNodePrefix.join("a", "b") === pathFromPlainName.join("a", "b");
            "#,
        )
        .unwrap();

    assert_eq!(result.trim(), "true");
}

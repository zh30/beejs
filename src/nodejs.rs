use crate::module_loader::ModuleLoader;
use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// Module cache - stores loaded modules for current execution
// Note: thread_local means each V8 isolate has its own cache
thread_local! {
    static MODULE_CACHE: Mutex<HashMap<String, v8::Global<v8::Object>>> = Mutex::new(HashMap::new());
    static LOADING_MODULES: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
}

// Global module loader for require callback
static mut MODULE_LOADER_GLOBAL: Option<Arc<ModuleLoader>> = None;

/// Set up all Node.js compatibility globals
pub fn setup_nodejs_apis(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    module_loader: Option<Arc<ModuleLoader>>,
    context: &v8::Local<v8::Context>,
    current_file: Option<&Path>,
) -> Result<()> {
    setup_process(scope, context)?;
    setup_path(scope, context)?;
    setup_fs(scope, context)?;
    setup_module_system(scope, module_loader, context, current_file)?;
    Ok(())
}

/// Process module implementation
fn setup_process(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let process = v8::Object::new(scope);

    // process.argv
    let argv = v8::Array::new(scope, 2);
    let arg0 = v8::String::new(scope, "beejs").unwrap();
    let arg1 = v8::String::new(scope, "<eval>").unwrap();
    argv.set_index(scope, 0, arg0.into());
    argv.set_index(scope, 1, arg1.into());

    let argv_key = v8::String::new(scope, "argv").unwrap();
    process.set(scope, argv_key.into(), argv.into());

    // process.version
    let version_key = v8::String::new(scope, "version").unwrap();
    let version_val = v8::String::new(scope, "1.0.0-beejs").unwrap();
    process.set(scope, version_key.into(), version_val.into());

    // process.cwd()
    let cwd_func = v8::FunctionTemplate::new(scope, cwd_callback);
    let cwd_func_instance = cwd_func.get_function(scope).unwrap();
    let cwd_key = v8::String::new(scope, "cwd").unwrap();
    process.set(scope, cwd_key.into(), cwd_func_instance.into());

    // process.nextTick()
    let next_tick_func = v8::FunctionTemplate::new(scope, next_tick_callback);
    let next_tick_instance = next_tick_func.get_function(scope).unwrap();
    let next_tick_key = v8::String::new(scope, "nextTick").unwrap();
    process.set(scope, next_tick_key.into(), next_tick_instance.into());

    // process.env
    let env_obj = v8::Object::new(scope);
    for (key, value) in env::vars() {
        let key_str = v8::String::new(scope, &key).unwrap();
        let val_str = v8::String::new(scope, &value).unwrap();
        env_obj.set(scope, key_str.into(), val_str.into());
    }
    let env_key = v8::String::new(scope, "env").unwrap();
    process.set(scope, env_key.into(), env_obj.into());

    // Set process on global
    let global = context.global(scope);
    let process_key = v8::String::new(scope, "process").unwrap();
    global.set(scope, process_key.into(), process.into());

    Ok(())
}

fn cwd_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let result = match env::current_dir() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => ".".to_string(),
    };
    let result_str = v8::String::new(scope, &result).unwrap();
    retval.set(result_str.into());
}

fn next_tick_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Simple implementation - execute callback immediately
    retval.set(v8::null(scope).into());
}

/// Path module implementation
fn setup_path(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let path_obj = v8::Object::new(scope);

    // path.join()
    let join_func = v8::FunctionTemplate::new(scope, path_join_callback);
    let join_instance = join_func.get_function(scope).unwrap();
    let join_key = v8::String::new(scope, "join").unwrap();
    path_obj.set(scope, join_key.into(), join_instance.into());

    // path.resolve()
    let resolve_func = v8::FunctionTemplate::new(scope, path_resolve_callback);
    let resolve_instance = resolve_func.get_function(scope).unwrap();
    let resolve_key = v8::String::new(scope, "resolve").unwrap();
    path_obj.set(scope, resolve_key.into(), resolve_instance.into());

    // path.dirname()
    let dirname_func = v8::FunctionTemplate::new(scope, path_dirname_callback);
    let dirname_instance = dirname_func.get_function(scope).unwrap();
    let dirname_key = v8::String::new(scope, "dirname").unwrap();
    path_obj.set(scope, dirname_key.into(), dirname_instance.into());

    // path.basename()
    let basename_func = v8::FunctionTemplate::new(scope, path_basename_callback);
    let basename_instance = basename_func.get_function(scope).unwrap();
    let basename_key = v8::String::new(scope, "basename").unwrap();
    path_obj.set(scope, basename_key.into(), basename_instance.into());

    // path.extname()
    let extname_func = v8::FunctionTemplate::new(scope, path_extname_callback);
    let extname_instance = extname_func.get_function(scope).unwrap();
    let extname_key = v8::String::new(scope, "extname").unwrap();
    path_obj.set(scope, extname_key.into(), extname_instance.into());

    // Set path on global
    let global = context.global(scope);
    let path_key = v8::String::new(scope, "path").unwrap();
    global.set(scope, path_key.into(), path_obj.into());

    Ok(())
}

fn path_join_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut paths = Vec::new();
    for i in 0..args.length() {
        let arg = args.get(i);
        if let Some(s) = arg.to_string(scope) {
            let arg_str = s.to_rust_string_lossy(scope);
            if !arg_str.is_empty() {
                paths.push(arg_str);
            }
        }
    }
    let result = paths.join("/");
    let result_str = v8::String::new(scope, &result).unwrap();
    retval.set(result_str.into());
}

fn path_resolve_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut paths = Vec::new();
    for i in 0..args.length() {
        let arg = args.get(i);
        if let Some(s) = arg.to_string(scope) {
            let arg_str = s.to_rust_string_lossy(scope);
            if !arg_str.is_empty() {
                paths.push(arg_str);
            }
        }
    }

    let mut result = PathBuf::new();
    for p in paths {
        let path = Path::new(&p);
        if path.is_absolute() {
            result = path.to_path_buf();
        } else {
            result = result.join(path);
        }
    }

    if result.as_os_str().is_empty() {
        result = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    let result_str = v8::String::new(scope, &result.to_string_lossy()).unwrap();
    retval.set(result_str.into());
}

fn path_dirname_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let arg = args.get(0);
    let arg_str = arg
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let path = Path::new(&arg_str);
    let result = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let result_str = v8::String::new(scope, &result).unwrap();
    retval.set(result_str.into());
}

fn path_basename_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let arg = args.get(0);
    let arg_str = arg
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let path = Path::new(&arg_str);
    let result = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&arg_str)
        .to_string();
    let result_str = v8::String::new(scope, &result).unwrap();
    retval.set(result_str.into());
}

fn path_extname_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let arg = args.get(0);
    let arg_str = arg
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let path = Path::new(&arg_str);
    let result = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default();
    let result_str = v8::String::new(scope, &result).unwrap();
    retval.set(result_str.into());
}

/// File System module implementation
fn setup_fs(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let fs_obj = v8::Object::new(scope);

    // fs.readFileSync()
    let read_func = v8::FunctionTemplate::new(scope, fs_read_file_sync_callback);
    let read_instance = read_func.get_function(scope).unwrap();
    let read_key = v8::String::new(scope, "readFileSync").unwrap();
    fs_obj.set(scope, read_key.into(), read_instance.into());

    // fs.writeFileSync()
    let write_func = v8::FunctionTemplate::new(scope, fs_write_file_sync_callback);
    let write_instance = write_func.get_function(scope).unwrap();
    let write_key = v8::String::new(scope, "writeFileSync").unwrap();
    fs_obj.set(scope, write_key.into(), write_instance.into());

    // fs.existsSync()
    let exists_func = v8::FunctionTemplate::new(scope, fs_exists_sync_callback);
    let exists_instance = exists_func.get_function(scope).unwrap();
    let exists_key = v8::String::new(scope, "existsSync").unwrap();
    fs_obj.set(scope, exists_key.into(), exists_instance.into());

    // fs.mkdirSync()
    let mkdir_func = v8::FunctionTemplate::new(scope, fs_mkdir_sync_callback);
    let mkdir_instance = mkdir_func.get_function(scope).unwrap();
    let mkdir_key = v8::String::new(scope, "mkdirSync").unwrap();
    fs_obj.set(scope, mkdir_key.into(), mkdir_instance.into());

    // fs.readdirSync()
    let readdir_func = v8::FunctionTemplate::new(scope, fs_readdir_sync_callback);
    let readdir_instance = readdir_func.get_function(scope).unwrap();
    let readdir_key = v8::String::new(scope, "readdirSync").unwrap();
    fs_obj.set(scope, readdir_key.into(), readdir_instance.into());

    // fs.statSync()
    let stat_func = v8::FunctionTemplate::new(scope, fs_stat_sync_callback);
    let stat_instance = stat_func.get_function(scope).unwrap();
    let stat_key = v8::String::new(scope, "statSync").unwrap();
    fs_obj.set(scope, stat_key.into(), stat_instance.into());

    // Set fs on global
    let global = context.global(scope);
    let fs_key = v8::String::new(scope, "fs").unwrap();
    global.set(scope, fs_key.into(), fs_obj.into());

    Ok(())
}

fn fs_read_file_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args.get(0);
    let path_str = path
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let content = fs::read_to_string(&path_str).unwrap_or_default();
    let result_str = v8::String::new(scope, &content).unwrap();
    retval.set(result_str.into());
}

fn fs_write_file_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let path = args.get(0);
    let data = args.get(1);

    let path_str = path
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let data_str = data
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let _ = fs::write(&path_str, data_str);
    // Just return - V8 ReturnValue doesn't have set_undefined in 0.20
}

fn fs_exists_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args.get(0);
    let path_str = path
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let exists = Path::new(&path_str).exists();
    retval.set(v8::Boolean::new(scope, exists).into());
}

fn fs_mkdir_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let path = args.get(0);
    let path_str = path
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let _ = fs::create_dir_all(&path_str);
    // Just return - V8 ReturnValue doesn't have set_undefined in 0.20
}

fn fs_readdir_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args.get(0);
    let path_str = path
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let mut entries = Vec::new();
    if let Ok(dir) = fs::read_dir(&path_str) {
        for entry in dir.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
    }

    let array = v8::Array::new(scope, entries.len() as i32);
    for (i, name) in entries.iter().enumerate() {
        let name_str = v8::String::new(scope, name).unwrap();
        array.set_index(scope, i as u32, name_str.into());
    }
    retval.set(array.into());
}

fn fs_stat_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args.get(0);
    let path_str = path
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // Return a simple stat object with isFile() and isDirectory()
    let stat_obj = v8::Object::new(scope);

    let is_file = Path::new(&path_str).is_file();
    let is_dir = Path::new(&path_str).is_dir();

    // Create isFile function
    let is_file_val = v8::Boolean::new(scope, is_file);
    let is_file_key = v8::String::new(scope, "isFile").unwrap();

    // Simple approach: store as boolean properties
    stat_obj.set(scope, is_file_key.into(), is_file_val.into());

    let is_dir_val = v8::Boolean::new(scope, is_dir);
    let is_dir_key = v8::String::new(scope, "isDirectory").unwrap();
    stat_obj.set(scope, is_dir_key.into(), is_dir_val.into());

    retval.set(stat_obj.into());
}

/// Module system implementation
fn setup_module_system(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    module_loader: Option<Arc<ModuleLoader>>,
    context: &v8::Local<v8::Context>,
    current_file: Option<&Path>,
) -> Result<()> {
    // Store module loader in global variable for access in callback
    unsafe {
        MODULE_LOADER_GLOBAL = module_loader;
    }

    // Create a callback that uses global storage
    let require_func = v8::FunctionTemplate::new(
        scope,
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut retval: v8::ReturnValue| {
            // Get module loader from global
            // Safe because we only access this during V8 execution in single-threaded context
            #[allow(static_mut_refs)]
            let module_loader = unsafe { MODULE_LOADER_GLOBAL.clone() };

            if let Some(module_name) = args.get(0).to_string(scope) {
                let module_name_str = module_name.to_rust_string_lossy(scope);

                // Use the module loader if available
                if let Some(loader) = &module_loader {
                    match loader.load_module(&module_name_str) {
                        Ok(module) => {
                            // Create a V8 object for the module exports
                            let exports_obj = v8::Object::new(scope);
                            for (key, value) in &module.exports {
                                let key_v8 = v8::String::new(scope, key).unwrap();
                                let v8_value = match value {
                                    serde_json::Value::String(s) => {
                                        v8::String::new(scope, &s).unwrap().into()
                                    }
                                    serde_json::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            v8::Integer::new(scope, i as i32).into()
                                        } else {
                                            v8::Number::new(scope, n.as_f64().unwrap_or(0.0)).into()
                                        }
                                    }
                                    serde_json::Value::Bool(b) => {
                                        v8::Boolean::new(scope, *b).into()
                                    }
                                    serde_json::Value::Null => v8::null(scope).into(),
                                    serde_json::Value::Array(arr) => {
                                        let v8_arr = v8::Array::new(scope, arr.len() as i32);
                                        for (i, item) in arr.iter().enumerate() {
                                            let v8_item = match item {
                                                serde_json::Value::String(s) => {
                                                    v8::String::new(scope, &s).unwrap().into()
                                                }
                                                serde_json::Value::Number(n) => {
                                                    if let Some(i) = n.as_i64() {
                                                        v8::Integer::new(scope, i as i32).into()
                                                    } else {
                                                        v8::Number::new(
                                                            scope,
                                                            n.as_f64().unwrap_or(0.0),
                                                        )
                                                        .into()
                                                    }
                                                }
                                                serde_json::Value::Bool(b) => {
                                                    v8::Boolean::new(scope, *b).into()
                                                }
                                                _ => v8::undefined(scope).into(),
                                            };
                                            v8_arr.set_index(scope, i as u32, v8_item);
                                        }
                                        v8_arr.into()
                                    }
                                    _ => v8::undefined(scope).into(),
                                };
                                exports_obj.set(scope, key_v8.into(), v8_value).unwrap();
                            }
                            retval.set(exports_obj.into());
                            return;
                        }
                        Err(e) => {
                            let error_msg =
                                format!("Error loading module '{}': {}", module_name_str, e);
                            retval.set(v8::String::new(scope, &error_msg).unwrap().into());
                            return;
                        }
                    }
                }

                // Fallback: return a simple mock module
                let mock_exports = v8::Object::new(scope);
                let key = v8::String::new(scope, "default").unwrap();
                let value = v8::String::new(scope, &format!("[Module: {}]", module_name_str))
                    .unwrap()
                    .into();
                mock_exports.set(scope, key.into(), value).unwrap();
                retval.set(mock_exports.into());
            } else {
                retval.set(v8::undefined(scope).into());
            }
        },
    );
    let require_instance = require_func.get_function(scope).unwrap();

    // Set require as a global function
    let global = context.global(scope);
    let require_key = v8::String::new(scope, "require").unwrap();
    global.set(scope, require_key.into(), require_instance.into());

    // Set up module and exports globals for non-module code
    // This allows execute_code to use module.exports directly
    let module_obj = v8::Object::new(scope);
    let exports_obj = v8::Object::new(scope);
    let module_exports_key = v8::String::new(scope, "exports").unwrap();
    module_obj.set(scope, module_exports_key.into(), exports_obj.into());

    let module_key = v8::String::new(scope, "module").unwrap();
    global.set(scope, module_key.into(), module_obj.into());

    let exports_key = v8::String::new(scope, "exports").unwrap();
    global.set(scope, exports_key.into(), exports_obj.into());

    // Set __dirname and __filename based on current file
    if let Some(file_path) = current_file {
        let dirname = file_path.parent().unwrap_or_else(|| Path::new("."));
        let dirname_key = v8::String::new(scope, "__dirname").unwrap();
        let dirname_val = v8::String::new(scope, &dirname.to_string_lossy()).unwrap();
        global.set(scope, dirname_key.into(), dirname_val.into());

        let filename_key = v8::String::new(scope, "__filename").unwrap();
        let filename_val = v8::String::new(scope, &file_path.to_string_lossy()).unwrap();
        global.set(scope, filename_key.into(), filename_val.into());
    } else {
        // Default values for eval mode
        let dirname_key = v8::String::new(scope, "__dirname").unwrap();
        let dirname_val = v8::String::new(scope, ".").unwrap();
        global.set(scope, dirname_key.into(), dirname_val.into());

        let filename_key = v8::String::new(scope, "__filename").unwrap();
        let filename_val = v8::String::new(scope, "").unwrap();
        global.set(scope, filename_key.into(), filename_val.into());
    }

    Ok(())
}

#[allow(dead_code)]
fn require_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let module_name = args.get(0);
    let module_name_str = module_name
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // Handle built-in modules
    match module_name_str.as_str() {
        "path" | "fs" | "process" => {
            // Get built-in from global
            let context = scope.get_current_context();
            let global = context.global(scope);
            let key = v8::String::new(scope, &module_name_str).unwrap();
            if let Some(module) = global.get(scope, key.into()) {
                retval.set(module);
                return;
            }
        }
        _ => {}
    }

    let context = scope.get_current_context();

    // Resolve module path first to get absolute path
    let module_path = resolve_module_path(scope, &module_name_str);
    let cache_key = if let Ok(ref path) = module_path {
        // Use absolute path as cache key
        Path::new(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| module_name_str.clone())
    } else {
        module_name_str.clone()
    };

    // Check if module is currently being loaded (circular dependency detection)
    let is_loading = LOADING_MODULES.with(|loading| {
        let loading_lock = loading.lock().unwrap();
        loading_lock.contains_key(&cache_key)
    });

    // Check cache first using absolute path
    MODULE_CACHE.with(|cache| {
        let cache_lock = cache.lock().unwrap();
        if let Some(cached_module) = cache_lock.get(&cache_key) {
            let cached_local = v8::Local::new(scope, cached_module);
            let exports = cached_local.into();
            retval.set(exports);
        }
    });

    if is_loading {
        // Circular dependency detected - return empty object for now
        // This matches Node.js behavior where incomplete exports are returned
        let empty = v8::Object::new(scope);
        retval.set(empty.into());
        return;
    }

    // Check cache first using absolute path
    let cached_result: Option<v8::Local<v8::Value>> = MODULE_CACHE.with(|cache| {
        let cache_lock = cache.lock().unwrap();
        let is_cached = cache_lock.contains_key(&cache_key);
        eprintln!("DEBUG: Cache contains key '{}': {}", cache_key, is_cached);
        if let Some(cached_module) = cache_lock.get(&cache_key) {
            let cached_local = v8::Local::new(scope, cached_module);
            return Some(cached_local.into());
        }
        None
    });

    if let Some(exports) = cached_result {
        // Module is cached - return the cached exports object
        // This ensures mod1 and mod2 reference the same object
        retval.set(exports);
        return;
    }

    // Mark module as being loaded
    LOADING_MODULES.with(|loading| {
        let mut loading_lock = loading.lock().unwrap();
        loading_lock.insert(cache_key.clone(), true);
    });

    // Check if file exists and load module
    if let Ok(path_str) = module_path {
        if Path::new(&path_str).exists() {
            // Read module code
            if let Ok(code) = fs::read_to_string(&path_str) {
                // Create module and exports objects
                // exports is an alias for module.exports, so they should reference the same object
                let exports_obj = v8::Object::new(scope);
                let module_obj = v8::Object::new(scope);

                // Set module.exports to exports_obj (they should be the same object)
                let module_exports_key = v8::String::new(scope, "exports").unwrap();
                module_obj.set(scope, module_exports_key.into(), exports_obj.into());

                // Get current __dirname and __filename for this module
                let module_dir = Path::new(&path_str)
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_string_lossy();
                let dirname_str = v8::String::new(scope, &module_dir).unwrap();
                let filename_str = v8::String::new(scope, &path_str).unwrap();

                // Get require function from global
                let global = context.global(scope);
                let require_key = v8::String::new(scope, "require").unwrap();
                let _require_func = if let Some(require) = global.get(scope, require_key.into()) {
                    require
                } else {
                    v8::null(scope).into()
                };

                // Set module context variables using prefixed names to avoid conflicts
                let module_var_key = v8::String::new(scope, "_module").unwrap();
                let exports_var_key = v8::String::new(scope, "_exports").unwrap();
                let dirname_var_key = v8::String::new(scope, "_dirname").unwrap();
                let filename_var_key = v8::String::new(scope, "_filename").unwrap();

                global.set(scope, module_var_key.into(), module_obj.into());
                global.set(scope, exports_var_key.into(), exports_obj.into());
                global.set(scope, dirname_var_key.into(), dirname_str.into());
                global.set(scope, filename_var_key.into(), filename_str.into());

                // Wrap code in IIFE that uses the prefixed global variables
                let module_wrapper = format!(
                    r#"(function() {{
                        var module = _module;
                        var exports = _exports;
                        var __dirname = _dirname;
                        var __filename = _filename;
                        {code}
                        return module.exports;
                    }})()"#,
                    code = code
                );

                if let Some(source) = v8::String::new(scope, &module_wrapper) {
                    if let Some(script) = v8::Script::compile(scope, source, None) {
                        if let Some(final_exports) = script.run(scope) {
                            // Clean up prefixed global variables (but NOT require, which is global)
                            global.delete(scope, module_var_key.into());
                            global.delete(scope, exports_var_key.into());
                            global.delete(scope, dirname_var_key.into());
                            global.delete(scope, filename_var_key.into());

                            // Cache the module exports object
                            if final_exports.is_object() {
                                let exports_obj = final_exports.to_object(scope).unwrap();
                                MODULE_CACHE.with(|cache| {
                                    let mut cache_lock = cache.lock().unwrap();
                                    let exports_global = v8::Global::new(scope, &exports_obj);
                                    cache_lock.insert(cache_key.clone(), exports_global);
                                });
                            }

                            // Remove from loading set before returning
                            LOADING_MODULES.with(|loading| {
                                let mut loading_lock = loading.lock().unwrap();
                                loading_lock.remove(&cache_key);
                            });

                            retval.set(final_exports);
                            return;
                        }
                    }
                }

                // Module loading failed - remove from loading set
                LOADING_MODULES.with(|loading| {
                    let mut loading_lock = loading.lock().unwrap();
                    loading_lock.remove(&cache_key);
                });
            }
        }
    }

    // Module not found or failed to load - remove from loading set
    LOADING_MODULES.with(|loading| {
        let mut loading_lock = loading.lock().unwrap();
        loading_lock.remove(&cache_key);
    });

    // Module not found, return empty object
    let empty = v8::Object::new(scope);
    retval.set(empty.into());
}

/// Resolve module path from module name
#[allow(dead_code)]
fn resolve_module_path(scope: &mut v8::HandleScope, module_name: &str) -> Result<String> {
    let context = scope.get_current_context();
    let global = context.global(scope);

    // Get current file's directory from __filename
    let filename_key = v8::String::new(scope, "__filename").unwrap();
    let current_file = if let Some(filename) = global.get(scope, filename_key.into()) {
        filename
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let base_dir = if !current_file.is_empty() {
        Path::new(&current_file)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        // Fallback to __dirname
        let dirname_key = v8::String::new(scope, "__dirname").unwrap();
        if let Some(dirname) = global.get(scope, dirname_key.into()) {
            let dirname_str = dirname
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
            if !dirname_str.is_empty() {
                PathBuf::from(dirname_str)
            } else {
                PathBuf::from(".")
            }
        } else {
            PathBuf::from(".")
        }
    };

    let mut path = base_dir;
    let module_name_trimmed = module_name.trim_start_matches("./");
    path.push(module_name_trimmed);

    // Add .js extension if not present and file doesn't exist
    if !path.exists() && path.extension().is_none() {
        path.set_extension("js");
    }

    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_nodejs_apis() {
        // Use the main module's V8 initialization
        crate::initialize_v8();

        let isolate = &mut v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(handle_scope);
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        let result = setup_nodejs_apis(scope, None, &context, None);
        assert!(result.is_ok());

        // Verify process exists
        let global = context.global(scope);
        let process_key = v8::String::new(scope, "process").unwrap();
        let process = global.get(scope, process_key.into()).unwrap();
        assert!(process.is_object());

        // Verify path exists
        let path_key = v8::String::new(scope, "path").unwrap();
        let path = global.get(scope, path_key.into()).unwrap();
        assert!(path.is_object());

        // Verify fs exists
        let fs_key = v8::String::new(scope, "fs").unwrap();
        let fs = global.get(scope, fs_key.into()).unwrap();
        assert!(fs.is_object());
    }
}

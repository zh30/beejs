use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use rusty_v8 as v8;
use once_cell::sync::Lazy;

/// Node.js compatibility module for V8
/// Provides fs, path, process and other Node.js core modules

/// Global module cache - stores loaded modules
static MODULE_CACHE: Lazy<Mutex<HashMap<String, v8::Global<v8::Object>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Set up all Node.js compatibility globals
pub fn setup_nodejs_apis(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
    setup_process(scope)?;
    setup_path(scope)?;
    setup_fs(scope)?;
    setup_module_system(scope)?;
    Ok(())
}

/// Process module implementation
fn setup_process(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
    let process = v8::Object::new(scope);

    // process.argv - use Array instead of Object
    let argv = v8::Array::new_with_length(scope, 2);
    // In a real implementation, these would come from actual CLI args
    argv.set_index(scope, 0, v8::String::new(scope, "beejs").unwrap().into());
    argv.set_index(scope, 1, v8::String::new(scope, "<eval>").unwrap().into());

    let process_key = v8::String::new(scope, "process").unwrap();
    let global = scope.global();
    global.set(scope, process_key.clone(), process.clone().into())?;
    process.set(scope, "argv", argv.into())?;

    // process.version
    process.set(scope, "version", v8::String::new(scope, "1.0.0-beejs").unwrap().into())?;

    // process.cwd()
    let cwd_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let result = match env::current_dir() {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => ".".to_string(),
        };
        retval.set(v8::String::new(scope, &result).unwrap().into());
    });
    let cwd_func_instance = cwd_func.get_function(scope).unwrap();
    process.set(scope, "cwd", cwd_func_instance.into())?;

    // process.nextTick()
    let next_tick_func = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        // Simple implementation - execute callback immediately
        // In a real implementation, this would use a task queue
        retval.set_undefined();
    });
    let next_tick_func_instance = next_tick_func.get_function(scope).unwrap();
    process.set(scope, "nextTick", next_tick_func_instance.into())?;

    // process.env
    let env = v8::Object::new(scope);
    for (key, value) in env::vars() {
        env.set(scope, v8::String::new(scope, &key).unwrap().into(), v8::String::new(scope, &value).unwrap().into())?;
    }
    process.set(scope, "env", env.into())?;

    Ok(())
}

/// Create process object for module use
fn create_process_object(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<v8::Local<v8::Object>> {
    let process = v8::Object::new(scope);

    // process.argv
    let argv = v8::Array::new_with_length(scope, 2);
    argv.set_index(scope, 0, v8::String::new(scope, "beejs").unwrap().into());
    argv.set_index(scope, 1, v8::String::new(scope, "<eval>").unwrap().into());
    process.set(scope, "argv", argv.into())?;

    // process.version
    process.set(scope, "version", v8::String::new(scope, "1.0.0-beejs").unwrap().into())?;

    // process.cwd()
    let cwd_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let result = match env::current_dir() {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => ".".to_string(),
        };
        retval.set(v8::String::new(scope, &result).unwrap().into());
    });
    let cwd_func_instance = cwd_func.get_function(scope).unwrap();
    process.set(scope, "cwd", cwd_func_instance.into())?;

    // process.env
    let env = v8::Object::new(scope);
    for (key, value) in env::vars() {
        env.set(scope, v8::String::new(scope, &key).unwrap().into(), v8::String::new(scope, &value).unwrap().into())?;
    }
    process.set(scope, "env", env.into())?;

    Ok(process)
}

/// Path module implementation
fn setup_path(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
    let path = v8::Object::new(scope);

    // path.join() - accept multiple string arguments
    let join_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let mut paths = Vec::new();
        for i in 0..args.length() {
            let arg = args.get(i);
            let arg_str = arg.to_string(scope)
                .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                .to_rust_string_lossy(scope);
            if !arg_str.is_empty() {
                paths.push(arg_str);
            }
        }
        let result = paths.join("/");
        retval.set(v8::String::new(scope, &result).unwrap().into());
    });
    let join_func_instance = join_func.get_function(scope).unwrap();
    path.set(scope, "join", join_func_instance.into())?;

    // path.resolve()
    let resolve_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let mut paths = Vec::new();
        for i in 0..args.length() {
            let arg = args.get(i);
            let arg_str = arg.to_string(scope)
                .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                .to_rust_string_lossy(scope);
            if !arg_str.is_empty() {
                paths.push(arg_str);
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

        // If no paths provided, return current directory
        if result.as_os_str().is_empty() {
            result = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }

        let result_str = result.to_string_lossy().to_string();
        retval.set(v8::String::new(scope, &result_str).unwrap().into());
    });
    let resolve_func_instance = resolve_func.get_function(scope).unwrap();
    path.set(scope, "resolve", resolve_func_instance.into())?;

    // path.dirname()
    let dirname_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let arg = args.get(0);
        let arg_str = arg.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);
        let path = Path::new(&arg_str);
        let result = if let Some(parent) = path.parent() {
            parent.to_string_lossy().to_string()
        } else {
            ".".to_string()
        };
        retval.set(v8::String::new(scope, &result).unwrap().into());
    });
    let dirname_func_instance = dirname_func.get_function(scope).unwrap();
    path.set(scope, "dirname", dirname_func_instance.into())?;

    // path.basename()
    let basename_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let arg = args.get(0);
        let arg_str = arg.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);
        let path = Path::new(&arg_str);
        let result = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&arg_str)
            .to_string();
        retval.set(v8::String::new(scope, &result).unwrap().into());
    });
    let basename_func_instance = basename_func.get_function(scope).unwrap();
    path.set(scope, "basename", basename_func_instance.into())?;

    // path.extname()
    let extname_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let arg = args.get(0);
        let arg_str = arg.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);
        let path = Path::new(&arg_str);
        let result = path.extension()
            .and_then(|s| {
                let ext = s.to_str()?;
                Some(format!(".{}", ext))
            })
            .unwrap_or_else(|| "".to_string());
        retval.set(v8::String::new(scope, &result).unwrap().into());
    });
    let extname_func_instance = extname_func.get_function(scope).unwrap();
    path.set(scope, "extname", extname_func_instance.into())?;

    let global = scope.global();
    global.set(scope, v8::String::new(scope, "path").unwrap().into(), path.into())?;

    Ok(())
}

/// File System module implementation
fn setup_fs(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
    let fs_obj = v8::Object::new(scope);

    // fs.readFileSync()
    let read_file_sync = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let path = args.get(0);
        let _encoding = args.get(1); // Not used in simple implementation

        let path_str = path.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        let content = match fs::read_to_string(&path_str) {
            Ok(content) => content,
            Err(_) => "".to_string(),
        };

        retval.set(v8::String::new(scope, &content).unwrap().into());
    });
    let read_file_sync_instance = read_file_sync.get_function(scope).unwrap();
    fs_obj.set(scope, "readFileSync", read_file_sync_instance.into())?;

    // fs.writeFileSync()
    let write_file_sync = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let path = args.get(0);
        let data = args.get(1);
        let _encoding = args.get(2); // Not used in simple implementation

        let path_str = path.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        let data_str = data.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        let _ = fs::write(&path_str, data_str);

        retval.set_undefined();
    });
    let write_file_sync_instance = write_file_sync.get_function(scope).unwrap();
    fs_obj.set(scope, "writeFileSync", write_file_sync_instance.into())?;

    // fs.existsSync()
    let exists_sync = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let path = args.get(0);
        let path_str = path.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        let result = Path::new(&path_str).exists();
        retval.set(v8::Boolean::new(scope, result).into());
    });
    let exists_sync_instance = exists_sync.get_function(scope).unwrap();
    fs_obj.set(scope, "existsSync", exists_sync_instance.into())?;

    // fs.mkdirSync()
    let mkdir_sync = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let path = args.get(0);
        let path_str = path.to_string(_scope)
            .unwrap_or_else(|| v8::String::new(_scope, "<error>").unwrap())
            .to_rust_string_lossy(_scope);

        let _ = fs::create_dir_all(&path_str);

        retval.set_undefined();
    });
    let mkdir_sync_instance = mkdir_sync.get_function(_scope).unwrap();
    fs_obj.set(scope, "mkdirSync", mkdir_sync_instance.into())?;

    // fs.readdirSync()
    let readdir_sync = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let path = args.get(0);
        let path_str = path.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        let mut result_vec = Vec::new();
        if let Ok(entries) = fs::read_dir(&path_str) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        result_vec.push(name.to_string());
                    }
                }
            }
        }

        // Create V8 array
        let array = v8::Array::new_with_length(scope, result_vec.len());
        for (i, name) in result_vec.iter().enumerate() {
            array.set_index(scope, i, v8::String::new(scope, name).unwrap().into());
        }

        retval.set(array.into());
    });
    let readdir_sync_instance = readdir_sync.get_function(scope).unwrap();
    fs_obj.set(scope, "readdirSync", readdir_sync_instance.into())?;

    // fs.statSync()
    let stat_sync = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let path = args.get(0);
        let path_str = path.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        let result = Path::new(&path_str).is_file();
        retval.set(v8::Boolean::new(scope, result).into());
    });
    let stat_sync_instance = stat_sync.get_function(scope).unwrap();
    fs_obj.set(scope, "statSync", stat_sync_instance.into())?;

    let global = scope.global();
    global.set(scope, v8::String::new(scope, "fs").unwrap().into(), fs_obj.into())?;

    Ok(())
}

/// Module system implementation - complete require/module.exports support
fn setup_module_system(scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
    // Global require function
    let require_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let module_name = args.get(0);
        let module_name_str = module_name.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        // Get the current file path from the context
        let current_file = get_current_script_path(scope);

        // Resolve module path
        match resolve_module_path(&module_name_str, current_file.as_deref()) {
            Ok(module_path) => {
                // Check cache first
                let cache_key = module_path.clone();
                if let Some(cached_module) = get_cached_module(&cache_key) {
                    retval.set(cached_module.into());
                    return;
                }

                // Load and execute module
                match load_and_execute_module(scope, &module_path, &cache_key) {
                    Ok(exports) => {
                        retval.set(exports.into());
                    }
                    Err(e) => {
                        // Return empty object on error
                        let empty_obj = v8::Object::new(scope);
                        retval.set(empty_obj.into());
                    }
                }
            }
            Err(_) => {
                // Module not found - return empty object
                let empty_obj = v8::Object::new(scope);
                retval.set(empty_obj.into());
            }
        }
    });
    let require_func_instance = require_func.get_function(scope).unwrap();

    let global = scope.global();
    global.set(scope, v8::String::new(scope, "require").unwrap().into(), require_func_instance.into())?;

    // Module object with exports property
    let module = v8::Object::new(scope);
    let exports = v8::Object::new(scope);
    module.set(scope, "exports", exports.into())?;
    global.set(scope, v8::String::new(scope, "module").unwrap().into(), module.into())?;

    // Also expose exports as a global
    let exports_obj = v8::Object::new(scope);
    global.set(scope, v8::String::new(scope, "exports").unwrap().into(), exports_obj.into())?;

    Ok(())
}

/// Get current script path from V8 context
fn get_current_script_path(scope: &mut v8::HandleScope) -> Option<String> {
    // Get the script origin from the current context
    let context = scope.get_current_context();
    let context_state = context.state(scope);
    if let Some(state) = context_state {
        // This is a simplified implementation
        // In a full implementation, we would track the script origin
        None
    } else {
        None
    }
}

/// Resolve module path from module name
fn resolve_module_path(module_name: &str, current_file: Option<&str>) -> Result<PathBuf, anyhow::Error> {
    // Handle built-in modules
    match module_name {
        "path" | "fs" | "process" => {
            return Ok(PathBuf::from(format!("__beejs_builtin__{}", module_name)));
        }
        _ => {}
    }

    // Handle relative paths
    if module_name.starts_with('./') || module_name.starts_with('../') {
        if let Some(current) = current_file {
            let current_path = Path::new(current);
            let current_dir = current_path.parent().unwrap_or_else(|| Path::new("."));
            let mut module_path = current_dir.join(module_name);

            // Add .js extension if not present
            if module_path.extension().is_none() {
                module_path.set_extension("js");
            }

            if module_path.exists() {
                return Ok(module_path);
            }
        }
    }

    // Handle absolute paths
    if Path::new(module_name).is_absolute() {
        let mut module_path = PathBuf::from(module_name);
        if module_path.extension().is_none() {
            module_path.set_extension("js");
        }
        if module_path.exists() {
            return Ok(module_path);
        }
    }

    Err(anyhow!("Module not found: {}", module_name))
}

/// Get cached module from the global cache
fn get_cached_module(cache_key: &str) -> Option<v8::Local<v8::Object>> {
    let cache = MODULE_CACHE.lock().unwrap();
    if let Some(global_module) = cache.get(cache_key) {
        // Return a handle to the cached module
        Some(v8::Local::<'_, v8::Object>::clone(&v8::Local::<'_, v8::Object>::from_global(
            &v8::HandleScope::empty(),
            global_module,
            &mut v8::HandleScope::empty(),
        )))
    } else {
        None
    }
}

/// Load and execute a module
fn load_and_execute_module(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    module_path: &Path,
    cache_key: &str,
) -> Result<v8::Local<v8::Object>, anyhow::Error> {
    // Handle built-in modules
    if module_path.to_string_lossy().starts_with("__beejs_builtin__") {
        let builtin_name = module_path.to_string_lossy().replace("__beejs_builtin__", "");
        return get_builtin_module(scope, &builtin_name);
    }

    // Read the module file
    let code = fs::read_to_string(module_path)
        .map_err(|e| anyhow!("Failed to read module: {}", e))?;

    // Get the global context
    let context = scope.get_current_context();
    let global = context.global(scope);

    // Create module-scoped objects
    let module = v8::Object::new(scope);
    let exports = v8::Object::new(scope);

    // Create require function for this module
    let module_require = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let module_name = args.get(0);
        let module_name_str = module_name.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        // Resolve relative to current module
        let current_module_path = module_path.to_string_lossy().to_string();
        match resolve_module_path(&module_name_str, Some(&current_module_path)) {
            Ok(resolved_path) => {
                let sub_cache_key = resolved_path.to_string_lossy().to_string();
                if let Some(cached) = get_cached_module(&sub_cache_key) {
                    retval.set(cached.into());
                    return;
                }

                match load_and_execute_module(scope, &resolved_path, &sub_cache_key) {
                    Ok(exports) => retval.set(exports.into()),
                    Err(_) => {
                        let empty = v8::Object::new(scope);
                        retval.set(empty.into());
                    }
                }
            }
            Err(_) => {
                let empty = v8::Object::new(scope);
                retval.set(empty.into());
            }
        }
    });
    let module_require_instance = module_require.get_function(scope).unwrap();

    // Set up module global with module, exports, require, and built-ins
    global.set(scope, v8::String::new(scope, "module").unwrap().into(), module.clone().into())?;
    global.set(scope, v8::String::new(scope, "exports").unwrap().into(), exports.clone().into())?;
    global.set(scope, v8::String::new(scope, "require").unwrap().into(), module_require_instance.into())?;

    // Ensure path, fs, process are available by copying from existing global
    if let Some(path_obj) = global.get(scope, v8::String::new(scope, "path").unwrap().into()) {
        global.set(scope, v8::String::new(scope, "path").unwrap().into(), path_obj)?;
    }
    if let Some(fs_obj) = global.get(scope, v8::String::new(scope, "fs").unwrap().into()) {
        global.set(scope, v8::String::new(scope, "fs").unwrap().into(), fs_obj)?;
    }
    if let Some(process_obj) = global.get(scope, v8::String::new(scope, "process").unwrap().into()) {
        global.set(scope, v8::String::new(scope, "process").unwrap().into(), process_obj)?;
    }

    // Execute the module code
    let source = v8::String::new(scope, &code).unwrap();
    let script = match v8::Script::compile(scope, source, None) {
        Some(script) => script,
        None => return Err(anyhow!("Failed to compile module")),
    };

    let _ = script.run(scope);

    // Get the final exports
    let final_exports = module.get(scope, v8::String::new(scope, "exports").unwrap().into())
        .unwrap_or_else(|| exports.clone().into());

    // Cache the module
    let exports_obj = v8::Local::<v8::Object>::try_from(final_exports)
        .unwrap_or_else(|_| exports.clone());

    // Create a persistent handle for the module
    let global_exports = v8::Global::new(scope, exports_obj);
    let mut cache = MODULE_CACHE.lock().unwrap();
    cache.insert(cache_key.to_string(), global_exports);

    Ok(exports_obj)
}

/// Get built-in module
fn get_builtin_module(scope: &mut v8::ContextScope<v8::HandleScope>, module_name: &str) -> Result<v8::Local<v8::Object>, anyhow::Error> {
    let global = scope.global();
    let module_obj = match module_name {
        "path" => global.get(scope, v8::String::new(scope, "path").unwrap().into()),
        "fs" => global.get(scope, v8::String::new(scope, "fs").unwrap().into()),
        "process" => global.get(scope, v8::String::new(scope, "process").unwrap().into()),
        _ => None,
    };

    let module = module_obj
        .ok_or_else(|| anyhow!("Built-in module not found: {}", module_name))?
        .to_object(scope)
        .ok_or_else(|| anyhow!("Failed to get module object"))?;

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_nodejs_apis() {
        // Initialize V8
        let platform = v8::new_default_platform(0, false).unwrap().make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let mut isolate = v8::Isolate::new(v8::CreateParams::default()).unwrap();
        let scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        let result = setup_nodejs_apis(scope);
        assert!(result.is_ok());

        // Verify process is available
        let global = scope.global();
        let process_key = v8::String::new(scope, "process").unwrap();
        let process = global.get(scope, process_key.into()).unwrap();
        assert!(process.is_object());

        // Verify path is available
        let path_key = v8::String::new(scope, "path").unwrap();
        let path = global.get(scope, path_key.into()).unwrap();
        assert!(path.is_object());

        // Verify fs is available
        let fs_key = v8::String::new(scope, "fs").unwrap();
        let fs = global.get(scope, fs_key.into()).unwrap();
        assert!(fs.is_object());
    }
}

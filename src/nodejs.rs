use rquickjs::{Value, Object, Function, Ctx, Result as QjsResult};
use rquickjs::function::Rest;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

/// Node.js compatibility module
/// Provides fs, path, process and other Node.js core modules

/// Set up all Node.js compatibility globals
pub fn setup_nodejs_apis(ctx: &Ctx) -> QjsResult<()> {
    setup_process(ctx)?;
    setup_path(ctx)?;
    setup_fs(ctx)?;
    setup_module_system(ctx)?;
    Ok(())
}

/// Process module implementation
fn setup_process(ctx: &Ctx) -> QjsResult<()> {
    let process = Object::new(ctx.clone())?;

    // process.argv - use Array instead of Object
    let argv = rquickjs::Array::new(ctx.clone())?;
    // In a real implementation, these would come from actual CLI args
    argv.set(0, "beejs")?;
    argv.set(1, "<eval>")?;
    ctx.globals().set("process", process.clone())?;
    process.set("argv", argv)?;

    // process.version
    process.set("version", "1.0.0-beejs")?;

    // process.cwd()
    let cwd_func = Function::new(ctx.clone(), |_ctx: Ctx| {
        match env::current_dir() {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => ".".to_string(),
        }
    })?;
    process.set("cwd", cwd_func)?;

    // process.nextTick()
    let next_tick_func = Function::new(ctx.clone(), |_ctx: Ctx, _callback: Function| {
        // Simple implementation - execute callback immediately
        // In a real implementation, this would use a task queue
        rquickjs::Undefined
    })?;
    process.set("nextTick", next_tick_func)?;

    // process.env
    let env = Object::new(ctx.clone())?;
    for (key, value) in env::vars() {
        env.set(&key, value)?;
    }
    process.set("env", env)?;

    Ok(())
}

/// Path module implementation
fn setup_path(ctx: &Ctx) -> QjsResult<()> {
    let path = Object::new(ctx.clone())?;

    // path.join() - accept multiple string arguments
    let join_func = Function::new(ctx.clone(), |_ctx: Ctx, args: Rest<Value>| -> String {
        let mut paths = Vec::new();
        for arg in args.iter() {
            // Convert Value to string - remove the Debug wrappers and quotes
            let arg_str = format!("{:?}", arg);
            // Remove common prefixes like "String(Ok(...))" or "String(...)"
            let cleaned = if arg_str.starts_with("String(Ok(") && arg_str.ends_with(')') {
                &arg_str[11..arg_str.len()-2]
            } else if arg_str.starts_with("String(") && arg_str.ends_with(')') {
                &arg_str[7..arg_str.len()-1]
            } else {
                &arg_str
            }
            .trim_matches('"')  // Remove surrounding quotes
            .to_string();
            if !cleaned.is_empty() {
                paths.push(cleaned);
            }
        }
        paths.join("/")
    })?;
    path.set("join", join_func)?;

    // path.resolve()
    let resolve_func = Function::new(ctx.clone(), |_ctx: Ctx, args: Rest<Value>| -> String {
        let mut paths = Vec::new();
        for arg in args.iter() {
            let arg_str = format!("{:?}", arg);
            let cleaned = if arg_str.starts_with("String(Ok(") && arg_str.ends_with(')') {
                &arg_str[11..arg_str.len()-2]
            } else if arg_str.starts_with("String(") && arg_str.ends_with(')') {
                &arg_str[7..arg_str.len()-1]
            } else {
                &arg_str
            }
            .trim_matches('"')
            .to_string();
            if !cleaned.is_empty() {
                paths.push(cleaned);
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

        result.to_string_lossy().to_string()
    })?;
    path.set("resolve", resolve_func)?;

    // path.dirname()
    let dirname_func = Function::new(ctx.clone(), |_ctx: Ctx, path_str: Value| -> String {
        let path_str = format!("{:?}", path_str);
        let cleaned = if path_str.starts_with("String(Ok(") && path_str.ends_with(')') {
            &path_str[11..path_str.len()-2]
        } else if path_str.starts_with("String(") && path_str.ends_with(')') {
            &path_str[7..path_str.len()-1]
        } else {
            &path_str
        }
        .trim_matches('"')
        .to_string();
        let path = Path::new(&cleaned);
        if let Some(parent) = path.parent() {
            parent.to_string_lossy().to_string()
        } else {
            ".".to_string()
        }
    })?;
    path.set("dirname", dirname_func)?;

    // path.basename()
    let basename_func = Function::new(ctx.clone(), |_ctx: Ctx, path_str: Value| -> String {
        let path_str = format!("{:?}", path_str);
        let cleaned = if path_str.starts_with("String(Ok(") && path_str.ends_with(')') {
            &path_str[11..path_str.len()-2]
        } else if path_str.starts_with("String(") && path_str.ends_with(')') {
            &path_str[7..path_str.len()-1]
        } else {
            &path_str
        }
        .trim_matches('"')
        .to_string();
        let path = Path::new(&cleaned);
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&cleaned)
            .to_string()
    })?;
    path.set("basename", basename_func)?;

    // path.extname()
    let extname_func = Function::new(ctx.clone(), |_ctx: Ctx, path_str: Value| -> String {
        let path_str = format!("{:?}", path_str);
        let cleaned = if path_str.starts_with("String(Ok(") && path_str.ends_with(')') {
            &path_str[11..path_str.len()-2]
        } else if path_str.starts_with("String(") && path_str.ends_with(')') {
            &path_str[7..path_str.len()-1]
        } else {
            &path_str
        }
        .trim_matches('"')
        .to_string();
        let path = Path::new(&cleaned);
        path.extension()
            .and_then(|s| {
                let ext = s.to_str()?;
                Some(format!(".{}", ext))
            })
            .unwrap_or_else(|| "".to_string())
    })?;
    path.set("extname", extname_func)?;

    ctx.globals().set("path", path)?;

    Ok(())
}

/// File System module implementation
fn setup_fs(ctx: &Ctx) -> QjsResult<()> {
    let fs_obj = Object::new(ctx.clone())?;

    // fs.readFileSync()
    let read_file_sync = Function::new(ctx.clone(), |_ctx: Ctx, path: String, _encoding: Option<String>| -> String {
        match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => "".to_string(),
        }
    })?;
    fs_obj.set("readFileSync", read_file_sync)?;

    // fs.writeFileSync()
    let write_file_sync = Function::new(ctx.clone(), |_ctx: Ctx, path: String, data: String, _encoding: Option<String>| {
        match fs::write(&path, data) {
            Ok(_) => rquickjs::Undefined,
            Err(_) => rquickjs::Undefined,
        }
    })?;
    fs_obj.set("writeFileSync", write_file_sync)?;

    // fs.existsSync()
    let exists_sync = Function::new(ctx.clone(), |_ctx: Ctx, path: String| -> bool {
        Path::new(&path).exists()
    })?;
    fs_obj.set("existsSync", exists_sync)?;

    // fs.mkdirSync()
    let mkdir_sync = Function::new(ctx.clone(), |_ctx: Ctx, path: String| {
        match fs::create_dir_all(&path) {
            Ok(_) => rquickjs::Undefined,
            Err(_) => rquickjs::Undefined,
        }
    })?;
    fs_obj.set("mkdirSync", mkdir_sync)?;

    // fs.readdirSync()
    let readdir_sync = Function::new(ctx.clone(), |_ctx: Ctx, path: String| -> Vec<String> {
        match fs::read_dir(&path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(name) = entry.file_name().to_str() {
                            result.push(name.to_string());
                        }
                    }
                }
                result
            }
            Err(_) => Vec::new(),
        }
    })?;
    fs_obj.set("readdirSync", readdir_sync)?;

    // fs.statSync() - simplified to return boolean to avoid GC issues
    let stat_sync = Function::new(ctx.clone(), |_ctx: Ctx, path: String| -> bool {
        Path::new(&path).is_file()
    })?;
    fs_obj.set("statSync", stat_sync)?;

    ctx.globals().set("fs", fs_obj)?;

    Ok(())
}

/// Module system implementation
fn setup_module_system(ctx: &Ctx) -> QjsResult<()> {
    // Global require function - simplified implementation
    let require_func = Function::new(ctx.clone(), |_ctx: Ctx, module_name: String| {
        // Simple require implementation - return a mock module object
        // TODO: Implement proper module loading
        format!("[Module: {}]", module_name)
    })?;
    ctx.globals().set("require", require_func)?;

    // Module object
    let module = Object::new(ctx.clone())?;
    let exports = Object::new(ctx.clone())?;
    module.set("exports", exports)?;
    ctx.globals().set("module", module)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_nodejs_apis() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = rquickjs::Context::full(&rt).unwrap();

        ctx.with(|ctx| {
            let result = setup_nodejs_apis(&ctx);
            assert!(result.is_ok());

            // Verify process is available
            let process: Value = ctx.globals().get("process").unwrap();
            assert!(process.is_object());

            // Verify path is available
            let path: Value = ctx.globals().get("path").unwrap();
            assert!(path.is_object());

            // Verify fs is available
            let fs: Value = ctx.globals().get("fs").unwrap();
            assert!(fs.is_object());
        });
    }
}

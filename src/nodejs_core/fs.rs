// Node.js fs模块实现 - 真正的文件系统操作
use anyhow::Result;
use rusty_v8 as v8;
use std::path::Path;

/// 设置fs API到全局作用域
pub fn setup_fs_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let fs_obj = v8::Object::new(scope);

    // readFileSync - 读取文件内容
    let read_func = v8::FunctionTemplate::new(scope, fs_read_file_sync_callback);
    let read_instance = read_func.get_function(scope).unwrap();
    let read_key = v8::String::new(scope, "readFileSync").unwrap();
    fs_obj.set(scope, read_key.into(), read_instance.into());

    // writeFileSync - 写入文件内容
    let write_func = v8::FunctionTemplate::new(scope, fs_write_file_sync_callback);
    let write_instance = write_func.get_function(scope).unwrap();
    let write_key = v8::String::new(scope, "writeFileSync").unwrap();
    fs_obj.set(scope, write_key.into(), write_instance.into());

    // existsSync - 检查文件是否存在
    let exists_func = v8::FunctionTemplate::new(scope, fs_exists_sync_callback);
    let exists_instance = exists_func.get_function(scope).unwrap();
    let exists_key = v8::String::new(scope, "existsSync").unwrap();
    fs_obj.set(scope, exists_key.into(), exists_instance.into());

    // mkdirSync - 创建目录
    let mkdir_func = v8::FunctionTemplate::new(scope, fs_mkdir_sync_callback);
    let mkdir_instance = mkdir_func.get_function(scope).unwrap();
    let mkdir_key = v8::String::new(scope, "mkdirSync").unwrap();
    fs_obj.set(scope, mkdir_key.into(), mkdir_instance.into());

    // readdirSync - 读取目录内容
    let readdir_func = v8::FunctionTemplate::new(scope, fs_readdir_sync_callback);
    let readdir_instance = readdir_func.get_function(scope).unwrap();
    let readdir_key = v8::String::new(scope, "readdirSync").unwrap();
    fs_obj.set(scope, readdir_key.into(), readdir_instance.into());

    // statSync - 获取文件状态
    let stat_func = v8::FunctionTemplate::new(scope, fs_stat_sync_callback);
    let stat_instance = stat_func.get_function(scope).unwrap();
    let stat_key = v8::String::new(scope, "statSync").unwrap();
    fs_obj.set(scope, stat_key.into(), stat_instance.into());

    // 设置到全局对象
    let global = context.global(scope);
    let fs_key = v8::String::new(scope, "fs").unwrap();
    global.set(scope, fs_key.into(), fs_obj.into());

    Ok(())
}

/// fs.readFileSync(path, encoding) - 读取文件
fn fs_read_file_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // 读取文件内容
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            retval.set(v8::String::new(scope, &content).unwrap().into());
        }
        Err(e) => {
            let error_msg = format!("Error reading file: {}", e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exc = v8::Exception::type_error(scope, error);
            scope.throw_exception(exc);
        }
    }
}

/// fs.writeFileSync(path, data, encoding) - 写入文件
fn fs_write_file_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let data: String = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // 写入文件
    match std::fs::write(&path, &data) {
        Ok(()) => {
            retval.set(v8::undefined(scope).into());
        }
        Err(e) => {
            let error_msg = format!("Error writing file: {}", e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exc = v8::Exception::type_error(scope, error);
            scope.throw_exception(exc);
        }
    }
}

/// fs.existsSync(path) - 检查文件是否存在
fn fs_exists_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let exists = Path::new(&path).exists();
    retval.set(v8::Boolean::new(scope, exists).into());
}

/// fs.mkdirSync(path) - 创建目录
fn fs_mkdir_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    match std::fs::create_dir_all(&path) {
        Ok(()) => {
            retval.set(v8::undefined(scope).into());
        }
        Err(e) => {
            let error_msg = format!("Error creating directory: {}", e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exc = v8::Exception::type_error(scope, error);
            scope.throw_exception(exc);
        }
    }
}

/// fs.readdirSync(path) - 读取目录内容
fn fs_readdir_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    match std::fs::read_dir(&path) {
        Ok(entries) => {
            let names: Vec<String> = entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                .collect();

            // 创建 JavaScript 数组
            let array = v8::Array::new(scope, names.len() as i32);
            for (i, name) in names.iter().enumerate() {
                let value = v8::String::new(scope, name).unwrap();
                array.set_index(scope, i as u32, value.into());
            }
            retval.set(array.into());
        }
        Err(e) => {
            let error_msg = format!("Error reading directory: {}", e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exc = v8::Exception::type_error(scope, error);
            scope.throw_exception(exc);
        }
    }
}

/// fs.statSync(path) - 获取文件状态
fn fs_stat_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    match std::fs::metadata(&path) {
        Ok(metadata) => {
            // 创建 stat 对象
            let stat_obj = v8::Object::new(scope);

            // isFile
            let is_file_key = v8::String::new(scope, "isFile").unwrap();
            let is_file_value = v8::Boolean::new(scope, metadata.is_file());
            stat_obj.set(scope, is_file_key.into(), is_file_value.into());

            // isDirectory
            let is_dir_key = v8::String::new(scope, "isDirectory").unwrap();
            let is_dir_value = v8::Boolean::new(scope, metadata.is_dir());
            stat_obj.set(scope, is_dir_key.into(), is_dir_value.into());

            // size
            let size_key = v8::String::new(scope, "size").unwrap();
            let size_value = v8::Number::new(scope, metadata.len() as f64);
            stat_obj.set(scope, size_key.into(), size_value.into());

            // mode (permissions) - 使用 0o644 转换为十进制 420
            let mode_key = v8::String::new(scope, "mode").unwrap();
            let mode_value = v8::Number::new(scope, 420.0_f64);
            stat_obj.set(scope, mode_key.into(), mode_value.into());

            // mtime (modified time as timestamp)
            let mtime_key = v8::String::new(scope, "mtime").unwrap();
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    let mtime_value = v8::Number::new(scope, duration.as_secs() as f64 * 1000.0);
                    stat_obj.set(scope, mtime_key.into(), mtime_value.into());
                }
            }

            retval.set(stat_obj.into());
        }
        Err(e) => {
            let error_msg = format!("Error getting file metadata: {}", e);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let exc = v8::Exception::type_error(scope, error);
            scope.throw_exception(exc);
        }
    }
}

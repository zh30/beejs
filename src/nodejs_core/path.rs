//! Node.js Path模块实现
//! 路径操作工具

use anyhow::Result;
use rusty_v8 as v8;

/// 设置Path API
pub fn setup_path_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let path_obj = v8::Object::new(scope);

    // join
    let join_func = v8::FunctionTemplate::new(scope, path_join_callback);
    let join_instance = join_func.get_function(scope).unwrap();
    let join_key = v8::String::new(scope, "join").unwrap();
    path_obj.set(scope, join_key.into(), join_instance.into());

    // resolve
    let resolve_func = v8::FunctionTemplate::new(scope, path_resolve_callback);
    let resolve_instance = resolve_func.get_function(scope).unwrap();
    let resolve_key = v8::String::new(scope, "resolve").unwrap();
    path_obj.set(scope, resolve_key.into(), resolve_instance.into());

    // relative
    let relative_func = v8::FunctionTemplate::new(scope, path_relative_callback);
    let relative_instance = relative_func.get_function(scope).unwrap();
    let relative_key = v8::String::new(scope, "relative").unwrap();
    path_obj.set(scope, relative_key.into(), relative_instance.into());

    // dirname
    let dirname_func = v8::FunctionTemplate::new(scope, path_dirname_callback);
    let dirname_instance = dirname_func.get_function(scope).unwrap();
    let dirname_key = v8::String::new(scope, "dirname").unwrap();
    path_obj.set(scope, dirname_key.into(), dirname_instance.into());

    // basename
    let basename_func = v8::FunctionTemplate::new(scope, path_basename_callback);
    let basename_instance = basename_func.get_function(scope).unwrap();
    let basename_key = v8::String::new(scope, "basename").unwrap();
    path_obj.set(scope, basename_key.into(), basename_instance.into());

    // extname
    let extname_func = v8::FunctionTemplate::new(scope, path_extname_callback);
    let extname_instance = extname_func.get_function(scope).unwrap();
    let extname_key = v8::String::new(scope, "extname").unwrap();
    path_obj.set(scope, extname_key.into(), extname_instance.into());

    // parse
    let parse_func = v8::FunctionTemplate::new(scope, path_parse_callback);
    let parse_instance = parse_func.get_function(scope).unwrap();
    let parse_key = v8::String::new(scope, "parse").unwrap();
    path_obj.set(scope, parse_key.into(), parse_instance.into());

    // format
    let format_func = v8::FunctionTemplate::new(scope, path_format_callback);
    let format_instance = format_func.get_function(scope).unwrap();
    let format_key = v8::String::new(scope, "format").unwrap();
    path_obj.set(scope, format_key.into(), format_instance.into());

    // isAbsolute
    let is_absolute_func = v8::FunctionTemplate::new(scope, path_is_absolute_callback);
    let is_absolute_instance = is_absolute_func.get_function(scope).unwrap();
    let is_absolute_key = v8::String::new(scope, "isAbsolute").unwrap();
    path_obj.set(scope, is_absolute_key.into(), is_absolute_instance.into());

    // normalize
    let normalize_func = v8::FunctionTemplate::new(scope, path_normalize_callback);
    let normalize_instance = normalize_func.get_function(scope).unwrap();
    let normalize_key = v8::String::new(scope, "normalize").unwrap();
    path_obj.set(scope, normalize_key.into(), normalize_instance.into());

    // sep
    let sep = if cfg!(windows) { "\\" } else { "/" };
    path_obj.set(scope, v8::String::new(scope, "sep").unwrap().into(), v8::String::new(scope, sep).unwrap().into());

    // delimiter
    let delimiter = if cfg!(windows) { ";" } else { ":" };
    path_obj.set(scope, v8::String::new(scope, "delimiter").unwrap().into(), v8::String::new(scope, delimiter).unwrap().into());

    // win32
    let win32_obj = v8::Object::new(scope);
    let posix_obj = v8::Object::new(scope);

    // 复制所有方法到win32和posix
    for &key_str in &["join", "resolve", "relative", "dirname", "basename", "extname", "parse", "format", "isAbsolute", "normalize"] {
        if let Some(method) = path_obj.get(scope, v8::String::new(scope, key_str).unwrap().into()) {
            win32_obj.set(scope, v8::String::new(scope, key_str).unwrap().into(), method);
            posix_obj.set(scope, v8::String::new(scope, key_str).unwrap().into(), method);
        }
    }

    win32_obj.set(scope, v8::String::new(scope, "sep").unwrap().into(), v8::String::new(scope, "\\").unwrap().into());
    win32_obj.set(scope, v8::String::new(scope, "delimiter").unwrap().into(), v8::String::new(scope, ";").unwrap().into());

    posix_obj.set(scope, v8::String::new(scope, "sep").unwrap().into(), v8::String::new(scope, "/").unwrap().into());
    posix_obj.set(scope, v8::String::new(scope, "delimiter").unwrap().into(), v8::String::new(scope, ":").unwrap().into());

    path_obj.set(scope, v8::String::new(scope, "win32").unwrap().into(), win32_obj.into());
    path_obj.set(scope, v8::String::new(scope, "posix").unwrap().into(), posix_obj.into());

    // 设置到全局
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

    let mut result = String::new();
    let is_windows = cfg!(windows);

    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            if is_windows && !result.ends_with('\\') {
                result.push('\\');
            } else if !is_windows && !result.ends_with('/') {
                result.push('/');
            }
        }
        result.push_str(path);
    }

    // 规范化路径
    result = normalize_path(&result, is_windows);

    retval.set(v8::String::new(scope, &result).unwrap().into());
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

    let is_windows = cfg!(windows);
    let mut result = String::new();

    // 如果没有路径，返回当前目录
    if paths.is_empty() {
        result = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).to_string_lossy().to_string();
    } else {
        // 从右向左处理路径
        for path in paths.into_iter().rev() {
            if is_windows && (path.starts_with('\\') || (path.len() > 1 && path.chars().nth(1) == ':')) {
                // 绝对路径
                result = path.to_string();
                break;
            } else if !is_windows && path.starts_with('/') {
                // 绝对路径
                result = path.to_string();
                break;
            } else {
                // 相对路径
                if result.is_empty() {
                    result = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).to_string_lossy().to_string();
                }
                result = format!("{}/{}", result.trim_end_matches(is_windows.then(|| '\\').unwrap_or('/')), path);
            }
        }
    }

    result = normalize_path(&result, is_windows);
    retval.set(v8::String::new(scope, &result).unwrap().into());
}

fn path_relative_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let from = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let to = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);

    // 简化的相对路径计算
    let from_parts: Vec<&str> = if is_windows {
        from.split('\\').filter(|s| !s.is_empty()).collect()
    } else {
        from.split('/').filter(|s| !s.is_empty()).collect()
    };

    let to_parts: Vec<&str> = if is_windows {
        to.split('\\').filter(|s| !s.is_empty()).collect()
    } else {
        to.split('/').filter(|s| !s.is_empty()).collect()
    };

    let mut result = String::new();
    let mut i = 0;

    // 找到共同前缀
    while i < from_parts.len() && i < to_parts.len() && from_parts[i] == to_parts[i] {
        i += 1;
    }

    // 添加向上一级
    for _ in i..from_parts.len() {
        result.push_str("../");
    }

    // 添加剩余的to路径
    for part in &to_parts[i..] {
        result.push_str(part);
        if part != to_parts.last().unwrap() {
            result.push('/');
        }
    }

    if result.is_empty() {
        result = ".".to_string();
    }

    retval.set(v8::String::new(scope, &result).unwrap().into());
}

fn path_dirname_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);
    let mut result = String::new();

    if is_windows {
        // Windows路径处理
        if path.len() == 2 && path.chars().nth(1) == ':' {
            result = path; // C: -> C:
        } else if path.len() > 2 && path.chars().nth(1) == ':' {
            result = &path[..2]; // C:\path -> C:\
        } else if let Some(last_slash) = path.rfind('\\') {
            if last_slash == 0 {
                result = "\\".to_string();
            } else {
                result = &path[..last_slash];
            }
        } else {
            result = ".".to_string();
        }
    } else {
        // POSIX路径处理
        if path == "/" {
            result = "/".to_string();
        } else if let Some(last_slash) = path.rfind('/') {
            if last_slash == 0 {
                result = "/".to_string();
            } else {
                result = &path[..last_slash];
            }
        } else {
            result = ".".to_string();
        }
    }

    retval.set(v8::String::new(scope, &result).unwrap().into());
}

fn path_basename_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let ext = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);
    let separator = if is_windows { '\\' } else { '/' };

    let result = if let Some(last_sep) = path.rfind(separator) {
        let basename = &path[last_sep + 1..];
        if !ext.is_empty() && basename.ends_with(&ext) {
            &basename[..basename.len() - ext.len()]
        } else {
            basename
        }
    } else {
        if !ext.is_empty() && path.ends_with(&ext) {
            &path[..path.len() - ext.len()]
        } else {
            &path
        }
    };

    retval.set(v8::String::new(scope, result).unwrap().into());
}

fn path_extname_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);
    let separator = if is_windows { '\\' } else { '/' };

    let result = if let Some(last_sep) = path.rfind(separator) {
        let basename = &path[last_sep + 1..];
        if let Some(dot_pos) = basename.rfind('.') {
            let ext = &basename[dot_pos..];
            if ext.len() > 1 && !ext.contains(separator) {
                ext
            } else {
                ""
            }
        } else {
            ""
        }
    } else if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos..];
        if ext.len() > 1 && !ext.contains(separator) {
            ext
        } else {
            ""
        }
    } else {
        ""
    };

    retval.set(v8::String::new(scope, result).unwrap().into());
}

fn path_parse_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);
    let separator = if is_windows { '\\' } else { '/' };

    let result = v8::Object::new(scope);

    // root
    let root = if is_windows {
        if path.len() > 1 && path.chars().nth(1) == ':' {
            &path[..3] // C:\
        } else if path.starts_with("\\\\") {
            "\\\\".to_string()
        } else {
            "".to_string()
        }
    } else {
        if path.starts_with('/') { "/" } else { "" }
    };

    // dir
    let dir = if let Some(last_sep) = path.rfind(separator) {
        if last_sep == 0 {
            if is_windows { "\\" } else { "/" }
        } else {
            &path[..last_sep]
        }
    } else {
        ""
    };

    // base
    let base = if let Some(last_sep) = path.rfind(separator) {
        &path[last_sep + 1..]
    } else {
        &path
    };

    // ext
    let ext = if let Some(dot_pos) = base.rfind('.') {
        &base[dot_pos..]
    } else {
        ""
    };

    // name
    let name = if !base.is_empty() && ext.len() < base.len() {
        &base[..base.len() - ext.len()]
    } else {
        &base
    };

    result.set(scope, v8::String::new(scope, "root").unwrap().into(), v8::String::new(scope, root).unwrap().into());
    result.set(scope, v8::String::new(scope, "dir").unwrap().into(), v8::String::new(scope, dir).unwrap().into());
    result.set(scope, v8::String::new(scope, "base").unwrap().into(), v8::String::new(scope, base).unwrap().into());
    result.set(scope, v8::String::new(scope, "ext").unwrap().into(), v8::String::new(scope, ext).unwrap().into());
    result.set(scope, v8::String::new(scope, "name").unwrap().into(), v8::String::new(scope, name).unwrap().into());

    retval.set(result.into());
}

fn path_format_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path_obj = args.get(0);

    if let Some(obj) = path_obj.to_object(scope) {
        let root = obj.get(scope, v8::String::new(scope, "root").unwrap().into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let dir = obj.get(scope, v8::String::new(scope, "dir").unwrap().into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let base = obj.get(scope, v8::String::new(scope, "base").unwrap().into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let name = obj.get(scope, v8::String::new(scope, "name").unwrap().into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let ext = obj.get(scope, v8::String::new(scope, "ext").unwrap().into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let result = if !dir.is_empty() {
            format!("{}/{}", dir, if !base.is_empty() { &base } else { &format!("{}{}", name, ext) })
        } else if !root.is_empty() {
            format!("{}{}", root, if !base.is_empty() { &base } else { &format!("{}{}", name, ext) })
        } else {
            if !base.is_empty() { base } else { format!("{}{}", name, ext) }
        };

        retval.set(v8::String::new(scope, &result).unwrap().into());
    } else {
        retval.set(v8::null(scope).into());
    }
}

fn path_is_absolute_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);

    let is_absolute = if is_windows {
        // Windows: C:\path or \\server\share
        (path.len() > 1 && path.chars().nth(1) == ':') || path.starts_with("\\\\")
    } else {
        // POSIX: /path
        path.starts_with('/')
    };

    retval.set(v8::Boolean::new(scope, is_absolute).into());
}

fn path_normalize_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let path = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let is_windows = cfg!(windows);
    let result = normalize_path(&path, is_windows);

    retval.set(v8::String::new(scope, &result).unwrap().into());
}

// 辅助函数：规范化路径
fn normalize_path(path: &str, is_windows: bool) -> String {
    let mut result = String::new();
    let separator = if is_windows { '\\' } else { '/' };
    let other_separator = if is_windows { '/' } else { '\\' };

    let mut parts: Vec<&str> = path
        .replace(other_separator, &separator.to_string())
        .split(separator)
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();

    let mut stack = Vec::new();

    for part in parts {
        if part == ".." {
            if !stack.is_empty() {
                stack.pop();
            }
        } else {
            stack.push(part);
        }
    }

    for (i, part) in stack.iter().enumerate() {
        if i > 0 {
            result.push(separator);
        }
        result.push_str(part);
    }

    if result.is_empty() {
        result = ".".to_string();
    }

    // 处理根路径
    if path.starts_with(&separator.to_string()) {
        result = format!("{}{}", separator, result);
    }

    result
}

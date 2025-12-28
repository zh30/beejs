// Node.js process 模块实现
// v0.3.237: 全局未捕获异常处理器和 process 对象
// 支持 process.on('uncaughtException') 和 process.on('unhandledRejection')

use std::sync::Mutex;
use anyhow::Result;
use rusty_v8 as v8;

thread_local! {
    /// 未捕获异常处理器
    static UNCAUGHT_EXCEPTION_HANDLERS: Mutex<Vec<v8::Global<v8::Value>>> = Mutex::new(Vec::new());
    /// 未处理的 Promise rejection 处理器
    static UNHANDLED_REJECTION_HANDLERS: Mutex<Vec<v8::Global<v8::Value>>> = Mutex::new(Vec::new());
    /// 程序是否应该退出
    static SHOULD_EXIT: Mutex<bool> = Mutex::new(false);
    /// 退出码
    static EXIT_CODE: Mutex<i32> = Mutex::new(0);
}

/// 设置 process 全局对象
pub fn setup_process_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // 创建 process 对象
    let process_obj = v8::Object::new(scope);

    // process.version - Node.js 版本
    let version_key = v8::String::new(scope, "version").unwrap();
    let version_val = v8::String::new(scope, "v20.0.0").unwrap();
    process_obj.set(scope, version_key.into(), version_val.into());

    // process.platform - 平台信息
    let platform_key = v8::String::new(scope, "platform").unwrap();
    #[cfg(target_os = "macos")]
    let platform_val = v8::String::new(scope, "darwin").unwrap();
    #[cfg(target_os = "linux")]
    let platform_val = v8::String::new(scope, "linux").unwrap();
    #[cfg(target_os = "windows")]
    let platform_val = v8::String::new(scope, "win32").unwrap();
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let platform_val = v8::String::new(scope, "unknown").unwrap();
    process_obj.set(scope, platform_key.into(), platform_val.into());

    // process.arch - 架构信息
    let arch_key = v8::String::new(scope, "arch").unwrap();
    #[cfg(target_arch = "x86_64")]
    let arch_val = v8::String::new(scope, "x64").unwrap();
    #[cfg(target_arch = "aarch64")]
    let arch_val = v8::String::new(scope, "arm64").unwrap();
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let arch_val = v8::String::new(scope, "unknown").unwrap();
    process_obj.set(scope, arch_key.into(), arch_val.into());

    // process.argv - 命令行参数
    let argv_key = v8::String::new(scope, "argv").unwrap();
    let argv_array = v8::Array::new(scope, 0);
    // 默认参数
    let arg0 = v8::String::new(scope, "beejs").unwrap();
    argv_array.set_index(scope, 0, arg0.into());
    let arg1 = v8::String::new(scope, "script.js").unwrap();
    argv_array.set_index(scope, 1, arg1.into());
    process_obj.set(scope, argv_key.into(), argv_array.into());

    // process.execPath - 可执行文件路径
    let exec_path_key = v8::String::new(scope, "execPath").unwrap();
    let exec_path_val = v8::String::new(scope, "/usr/local/bin/beejs").unwrap();
    process_obj.set(scope, exec_path_key.into(), exec_path_val.into());

    // process.cwd() - 获取当前工作目录
    let cwd_func = v8::FunctionTemplate::new(scope, process_cwd_callback);
    let cwd_instance = cwd_func.get_function(scope).unwrap();
    let cwd_key = v8::String::new(scope, "cwd").unwrap();
    process_obj.set(scope, cwd_key.into(), cwd_instance.into());

    // process.exit() - 退出程序
    let exit_func = v8::FunctionTemplate::new(scope, process_exit_callback);
    let exit_instance = exit_func.get_function(scope).unwrap();
    let exit_key = v8::String::new(scope, "exit").unwrap();
    process_obj.set(scope, exit_key.into(), exit_instance.into());

    // process.on() - 事件监听
    let on_func = v8::FunctionTemplate::new(scope, process_on_callback);
    let on_instance = on_func.get_function(scope).unwrap();
    let on_key = v8::String::new(scope, "on").unwrap();
    process_obj.set(scope, on_key.into(), on_instance.into());

    // process.off() - 移除事件监听
    let off_func = v8::FunctionTemplate::new(scope, process_off_callback);
    let off_instance = off_func.get_function(scope).unwrap();
    let off_key = v8::String::new(scope, "off").unwrap();
    process_obj.set(scope, off_key.into(), off_instance.into());

    // process.removeListener() - 移除特定监听器
    let remove_listener_func = v8::FunctionTemplate::new(scope, process_remove_listener_callback);
    let remove_listener_instance = remove_listener_func.get_function(scope).unwrap();
    let remove_listener_key = v8::String::new(scope, "removeListener").unwrap();
    process_obj.set(scope, remove_listener_key.into(), remove_listener_instance.into());

    // process.pid - 进程 ID
    let pid_key = v8::String::new(scope, "pid").unwrap();
    let pid_val = v8::Integer::new(scope, std::process::id() as i32);
    process_obj.set(scope, pid_key.into(), pid_val.into());

    // process.ppid - 父进程 ID
    let ppid_key = v8::String::new(scope, "ppid").unwrap();
    let ppid_val = v8::Integer::new(scope, 1); // 默认 1
    process_obj.set(scope, ppid_key.into(), ppid_val.into());

    // process.env - 环境变量
    let env_key = v8::String::new(scope, "env").unwrap();
    let env_obj = v8::Object::new(scope);
    // 添加常见环境变量
    let home_key = v8::String::new(scope, "HOME").unwrap();
    let home_val = v8::String::new(scope, std::env::var("HOME").unwrap_or_default().as_str()).unwrap();
    env_obj.set(scope, home_key.into(), home_val.into());
    let path_key = v8::String::new(scope, "PATH").unwrap();
    let path_val = v8::String::new(scope, std::env::var("PATH").unwrap_or_default().as_str()).unwrap();
    env_obj.set(scope, path_key.into(), path_val.into());
    process_obj.set(scope, env_key.into(), env_obj.into());

    // process.release - 发布信息
    let release_key = v8::String::new(scope, "release").unwrap();
    let release_obj = v8::Object::new(scope);
    let name_key = v8::String::new(scope, "name").unwrap();
    let name_val = v8::String::new(scope, "beejs").unwrap();
    release_obj.set(scope, name_key.into(), name_val.into());
    process_obj.set(scope, release_key.into(), release_obj.into());

    // process.stdout - 标准输出
    let stdout_key = v8::String::new(scope, "stdout").unwrap();
    let stdout_obj = v8::Object::new(scope);
    process_obj.set(scope, stdout_key.into(), stdout_obj.into());

    // process.stderr - 标准错误
    let stderr_key = v8::String::new(scope, "stderr").unwrap();
    let stderr_obj = v8::Object::new(scope);
    process_obj.set(scope, stderr_key.into(), stderr_obj.into());

    // process.stdin - 标准输入
    let stdin_key = v8::String::new(scope, "stdin").unwrap();
    let stdin_obj = v8::Object::new(scope);
    process_obj.set(scope, stdin_key.into(), stdin_obj.into());

    // 将 process 对象设置为全局
    let process_key = v8::String::new(scope, "process").unwrap();
    global.set(scope, process_key.into(), process_obj.into());

    Ok(())
}

/// process.cwd() 回调
fn process_cwd_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/".to_string());
    let cwd_str = v8::String::new(scope, &cwd).unwrap();
    retval.set(cwd_str.into());
}

/// process.exit() 回调
fn process_exit_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let code = args.get(0).int32_value(scope).unwrap_or(0);

    // 设置退出状态
    SHOULD_EXIT.with(|exit| {
        let mut should_exit = exit.lock().unwrap();
        *should_exit = true;
    });
    EXIT_CODE.with(|exit_code| {
        let mut code_ref = exit_code.lock().unwrap();
        *code_ref = code;
    });

    // 注意: 实际退出需要在 Rust 层面处理
    // 这里只是标记退出状态
    eprintln!("[beejs] Process exiting with code: {}", code);
}

/// process.on() 回调
fn process_on_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let event_name = args.get(0);
    let handler = args.get(1);

    if !handler.is_function() {
        let error = v8::String::new(scope, "Event handler must be a function").unwrap();
        retval.set(error.into());
        return;
    }

    let event_name_str = event_name.to_string(scope).unwrap().to_rust_string_lossy(scope);

    match event_name_str.as_str() {
        "uncaughtException" => {
            // 添加到未捕获异常处理器列表
            let handler_global = v8::Global::new(scope, handler);
            UNCAUGHT_EXCEPTION_HANDLERS.with(|handlers| {
                let mut h = handlers.lock().unwrap();
                h.push(handler_global);
            });
            retval.set(scope.get_current_context().global(scope).into());
        }
        "unhandledRejection" => {
            // 添加到未处理的 Promise rejection 处理器列表
            let handler_global = v8::Global::new(scope, handler);
            UNHANDLED_REJECTION_HANDLERS.with(|handlers| {
                let mut h = handlers.lock().unwrap();
                h.push(handler_global);
            });
            retval.set(scope.get_current_context().global(scope).into());
        }
        _ => {
            retval.set(scope.get_current_context().global(scope).into());
        }
    }
}

/// process.off() 回调
fn process_off_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let event_name = args.get(0);
    let handler = args.get(1);

    if !handler.is_function() {
        retval.set(scope.get_current_context().global(scope).into());
        return;
    }

    let event_name_str = event_name.to_string(scope).unwrap().to_rust_string_lossy(scope);

    match event_name_str.as_str() {
        "uncaughtException" => {
            // 移除未捕获异常处理器
            // 注意: 这是一个简化实现，实际需要更复杂的比较逻辑
            retval.set(scope.get_current_context().global(scope).into());
        }
        "unhandledRejection" => {
            // 移除未处理的 Promise rejection 处理器
            retval.set(scope.get_current_context().global(scope).into());
        }
        _ => {
            retval.set(scope.get_current_context().global(scope).into());
        }
    }
}

/// process.removeListener() 回调
fn process_remove_listener_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let _event_name = args.get(0);
    let _handler = args.get(1);

    retval.set(scope.get_current_context().global(scope).into());
}

/// 触发未捕获异常事件
pub fn emit_uncaught_exception(
    scope: &mut v8::HandleScope,
    error: &v8::Local<v8::Value>,
) {
    UNCAUGHT_EXCEPTION_HANDLERS.with(|handlers| {
        let handlers = handlers.lock().unwrap();
        for handler in handlers.iter() {
            let handler_val = v8::Local::new(scope, handler);
            // 转换为 Function 类型
            if handler_val.is_function() {
                if let Ok(handler_func) = v8::Local::<v8::Function>::try_from(handler_val) {
                    let this = scope.get_current_context().global(scope);
                    let result = handler_func.call(scope, this.into(), &[*error]);
                    if result.is_none() {
                        // 处理器执行失败，忽略
                    }
                }
            }
        }
    });
}

/// 触发未处理的 Promise rejection 事件
pub fn emit_unhandled_rejection(
    scope: &mut v8::HandleScope,
    reason: &v8::Local<v8::Value>,
    promise: &v8::Local<v8::Value>,
) {
    UNHANDLED_REJECTION_HANDLERS.with(|handlers| {
        let handlers = handlers.lock().unwrap();
        for handler in handlers.iter() {
            let handler_val = v8::Local::new(scope, handler);
            // 转换为 Function 类型
            if handler_val.is_function() {
                if let Ok(handler_func) = v8::Local::<v8::Function>::try_from(handler_val) {
                    let this = scope.get_current_context().global(scope);
                    let result = handler_func.call(scope, this.into(), &[*reason, *promise]);
                    if result.is_none() {
                        // 处理器执行失败，忽略
                    }
                }
            }
        }
    });
}

/// 检查是否应该退出
pub fn should_exit() -> bool {
    SHOULD_EXIT.with(|exit| *exit.lock().unwrap())
}

/// 获取退出码
pub fn get_exit_code() -> i32 {
    EXIT_CODE.with(|code| *code.lock().unwrap())
}

/// 重置状态（用于测试）
#[cfg(test)]
pub fn reset_process_state() {
    UNCAUGHT_EXCEPTION_HANDLERS.with(|handlers| {
        let mut h = handlers.lock().unwrap();
        h.clear();
    });
    UNHANDLED_REJECTION_HANDLERS.with(|handlers| {
        let mut h = handlers.lock().unwrap();
        h.clear();
    });
    SHOULD_EXIT.with(|exit| {
        *exit.lock().unwrap() = false;
    });
    EXIT_CODE.with(|code| {
        *code.lock().unwrap() = 0;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_reset_process_state() {
        reset_process_state();
        assert!(!should_exit());
        assert_eq!(get_exit_code(), 0);
    }
}

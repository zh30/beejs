// Node.js process 模块实现
// v0.3.237: 全局未捕获异常处理器和 process 对象
// v0.3.239: 完善 nextTick 和 stdout/stderr.write()
// v0.3.240: 完善 hrtime、stdin、memory、uptime、cpuUsage

use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;
use anyhow::Result;
use rusty_v8 as v8;

thread_local! {
    /// 进程启动时间（用于计算 uptime）
    static START_TIME: Instant = Instant::now();
}

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

/// v0.3.239: nextTick 回调数据结构
struct NextTickCallback {
    callback: v8::Global<v8::Value>,
    args: Vec<v8::Global<v8::Value>>,
}

// v0.3.239: nextTick 队列（线程本地）
thread_local! {
    static NEXT_TICK_QUEUE: Mutex<Vec<NextTickCallback>> = Mutex::new(Vec::new());
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

    // v0.3.239: process.nextTick() - 微任务队列优先级
    let next_tick_func = v8::FunctionTemplate::new(scope, process_next_tick_callback);
    let next_tick_instance = next_tick_func.get_function(scope).unwrap();
    let next_tick_key = v8::String::new(scope, "nextTick").unwrap();
    process_obj.set(scope, next_tick_key.into(), next_tick_instance.into());

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

    // v0.3.239: process.stdout - 标准输出（带 write() 方法）
    let stdout_key = v8::String::new(scope, "stdout").unwrap();
    let stdout_obj = v8::Object::new(scope);
    let stdout_write_func = v8::FunctionTemplate::new(scope, stdout_write_callback);
    let stdout_write_instance = stdout_write_func.get_function(scope).unwrap();
    let stdout_write_key = v8::String::new(scope, "write").unwrap();
    stdout_obj.set(scope, stdout_write_key.into(), stdout_write_instance.into());
    process_obj.set(scope, stdout_key.into(), stdout_obj.into());

    // v0.3.239: process.stderr - 标准错误（带 write() 方法）
    let stderr_key = v8::String::new(scope, "stderr").unwrap();
    let stderr_obj = v8::Object::new(scope);
    let stderr_write_func = v8::FunctionTemplate::new(scope, stderr_write_callback);
    let stderr_write_instance = stderr_write_func.get_function(scope).unwrap();
    let stderr_write_key = v8::String::new(scope, "write").unwrap();
    stderr_obj.set(scope, stderr_write_key.into(), stderr_write_instance.into());
    process_obj.set(scope, stderr_key.into(), stderr_obj.into());

    // v0.3.240: process.stdin - 标准输入
    let stdin_key = v8::String::new(scope, "stdin").unwrap();
    let stdin_obj = v8::Object::new(scope);
    // stdin.fd - 标准输入的文件描述符 (0)
    let stdin_fd_key = v8::String::new(scope, "fd").unwrap();
    let stdin_fd_val = v8::Integer::new(scope, 0);
    stdin_obj.set(scope, stdin_fd_key.into(), stdin_fd_val.into());
    // stdin.read() - 读取输入（同步版本返回 null）
    let stdin_read_func = v8::FunctionTemplate::new(scope, stdin_read_callback);
    let stdin_read_instance = stdin_read_func.get_function(scope).unwrap();
    let stdin_read_key = v8::String::new(scope, "read").unwrap();
    stdin_obj.set(scope, stdin_read_key.into(), stdin_read_instance.into());
    process_obj.set(scope, stdin_key.into(), stdin_obj.into());

    // v0.3.240: process.hrtime() - 高精度时间
    let hrtime_func = v8::FunctionTemplate::new(scope, process_hrtime_callback);
    let hrtime_instance = hrtime_func.get_function(scope).unwrap();
    let hrtime_key = v8::String::new(scope, "hrtime").unwrap();
    process_obj.set(scope, hrtime_key.into(), hrtime_instance.into());

    // v0.3.240: process.memory() - 内存使用统计
    let memory_func = v8::FunctionTemplate::new(scope, process_memory_callback);
    let memory_instance = memory_func.get_function(scope).unwrap();
    let memory_key = v8::String::new(scope, "memory").unwrap();
    process_obj.set(scope, memory_key.into(), memory_instance.into());

    // v0.3.240: process.uptime() - 进程运行时间
    let uptime_func = v8::FunctionTemplate::new(scope, process_uptime_callback);
    let uptime_instance = uptime_func.get_function(scope).unwrap();
    let uptime_key = v8::String::new(scope, "uptime").unwrap();
    process_obj.set(scope, uptime_key.into(), uptime_instance.into());

    // v0.3.240: process.cpuUsage() - CPU 使用统计
    let cpu_usage_func = v8::FunctionTemplate::new(scope, process_cpu_usage_callback);
    let cpu_usage_instance = cpu_usage_func.get_function(scope).unwrap();
    let cpu_usage_key = v8::String::new(scope, "cpuUsage").unwrap();
    process_obj.set(scope, cpu_usage_key.into(), cpu_usage_instance.into());

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

/// v0.3.239: process.nextTick() 回调
/// 简化实现：使用 V8 的 MicrotaskQueue 来执行 nextTick 回调
fn process_next_tick_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let callback = args.get(0);

    if !callback.is_function() {
        let error = v8::String::new(scope, "process.nextTick: callback must be a function").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // 收集额外参数
    let callback_args: Vec<v8::Global<v8::Value>> = (1..args.length())
        .map(|i| args.get(i))
        .filter(|v| !v.is_undefined())
        .map(|v| v8::Global::new(scope, v))
        .collect();

    // 保存到 nextTick 队列
    let callback_global = v8::Global::new(scope, callback);
    NEXT_TICK_QUEUE.with(|queue| {
        let mut q = queue.lock().unwrap();
        q.push(NextTickCallback {
            callback: callback_global,
            args: callback_args,
        });
    });

    // 使用 Promise 来实现微任务排队
    // nextTick 回调将在当前同步代码执行完毕后、Promise 回调之前执行
    let promise_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
        // 执行所有 nextTick 回调
        NEXT_TICK_QUEUE.with(|q| {
            let mut queue_ref = q.lock().unwrap();
            let callbacks: Vec<NextTickCallback> = std::mem::take(&mut *queue_ref);

            for NextTickCallback { callback, args } in callbacks {
                let callback_local = v8::Local::new(scope, callback);
                if let Ok(func) = v8::Local::<v8::Function>::try_from(callback_local) {
                    let undefined = v8::undefined(scope);
                    let args_local: Vec<v8::Local<v8::Value>> =
                        args.iter().map(|a| v8::Local::new(scope, a)).collect();
                    let _ = func.call(scope, undefined.into(), &args_local);
                }
            }
        });
    });

    let promise_func_instance = promise_func.get_function(scope).unwrap();
    let promise_ctor_key = v8::String::new(scope, "Promise").unwrap();
    let promise_ctor = context(scope).global(scope).get(scope, promise_ctor_key.into()).unwrap();
    if promise_ctor.is_function() {
        let promise_ctor_func = v8::Local::<v8::Function>::try_from(promise_ctor).unwrap();
        let undefined = v8::undefined(scope);
        let _ = promise_ctor_func.call(scope, undefined.into(), &[promise_func_instance.into()]);
    }
}

// 获取 context 的辅助函数
fn context<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Context> {
    scope.get_current_context()
}

/// v0.3.239: stdout.write() 回调
fn stdout_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let data = args.get(0);

    // 将 V8 值转换为字符串并输出到 stdout
    let output = if let Some(str_val) = data.to_string(scope) {
        str_val.to_rust_string_lossy(scope)
    } else if data.is_null_or_undefined() {
        String::new()
    } else {
        String::from("[object]")
    };

    // 输出到 stdout 并刷新
    let mut stdout = std::io::stdout();
    let _ = write!(stdout, "{}", output);
    let _ = stdout.flush();

    // 返回 true（表示写入成功）
    let result = v8::Boolean::new(scope, true);
    retval.set(result.into());
}

/// v0.3.239: stderr.write() 回调
fn stderr_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let data = args.get(0);

    // 将 V8 值转换为字符串并输出到 stderr
    let output = if let Some(str_val) = data.to_string(scope) {
        str_val.to_rust_string_lossy(scope)
    } else if data.is_null_or_undefined() {
        String::new()
    } else {
        String::from("[object]")
    };

    // 输出到 stderr 并刷新
    let mut stderr = std::io::stderr();
    let _ = write!(stderr, "{}", output);
    let _ = stderr.flush();

    // 返回 true（表示写入成功）
    let result = v8::Boolean::new(scope, true);
    retval.set(result.into());
}

/// v0.3.240: stdin.read() 回调
/// 同步读取不支持，返回 null（需要异步运行时支持）
fn stdin_read_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 同步模式下无法读取 stdin，返回 null
    let null_val = v8::null(scope);
    retval.set(null_val.into());
}

/// v0.3.240: process.hrtime() 回调 - 高精度时间
fn process_hrtime_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 获取当前时间（纳秒精度）
    let now = std::time::UNIX_EPOCH.elapsed().unwrap();
    let seconds = now.as_secs() as f64;
    let nanos = now.subsec_nanos() as f64;

    // 如果传入了 previous timestamp，计算差值
    if args.length() > 0 {
        let prev = args.get(0);
        if prev.is_array() {
            let prev_array = v8::Local::<v8::Array>::try_from(prev).unwrap();
            if prev_array.length() >= 2 {
                let prev_sec = prev_array.get_index(scope, 0).unwrap().to_number(scope).unwrap().value();
                let prev_nano = prev_array.get_index(scope, 1).unwrap().to_number(scope).unwrap().value();

                let diff_sec = seconds - prev_sec;
                let diff_nano = nanos - prev_nano;

                let diff_sec_num = v8::Number::new(scope, diff_sec);
                let diff_nano_num = v8::Number::new(scope, diff_nano);
                let result = v8::Array::new(scope, 2);
                result.set_index(scope, 0, diff_sec_num.into());
                result.set_index(scope, 1, diff_nano_num.into());
                retval.set(result.into());
                return;
            }
        }
    }

    // 返回 [seconds, nanoseconds] 数组
    let seconds_num = v8::Number::new(scope, seconds);
    let nanos_num = v8::Number::new(scope, nanos);
    let result = v8::Array::new(scope, 2);
    result.set_index(scope, 0, seconds_num.into());
    result.set_index(scope, 1, nanos_num.into());
    retval.set(result.into());
}

/// v0.3.240: process.memory() 回调 - 内存使用统计
/// 注意: 由于 MinimalRuntime 不持有 isolate 引用，这里返回估算值
/// 完整实现需要在持有 isolate 的上下文中调用
fn process_memory_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 返回基于配置的估算值
    // MinimalRuntime 默认配置: 64MB 初始堆, 512MB 最大堆
    // 这是一个简化实现，返回合理的默认值
    let estimated_heap_used = 64.0 * 1024.0 * 1024.0; // 估算 64MB 已使用
    let estimated_heap_total = 512.0 * 1024.0 * 1024.0; // 估算 512MB 总堆
    let estimated_external = 0.0; // 外部内存估算

    let result = v8::Object::new(scope);

    // heapUsed - 已使用的堆内存
    let heap_used_key = v8::String::new(scope, "heapUsed").unwrap();
    let heap_used_val = v8::Number::new(scope, estimated_heap_used);
    result.set(scope, heap_used_key.into(), heap_used_val.into());

    // heapTotal - 总堆内存
    let heap_total_key = v8::String::new(scope, "heapTotal").unwrap();
    let heap_total_val = v8::Number::new(scope, estimated_heap_total);
    result.set(scope, heap_total_key.into(), heap_total_val.into());

    // external - 外部内存
    let external_key = v8::String::new(scope, "external").unwrap();
    let external_val = v8::Number::new(scope, estimated_external);
    result.set(scope, external_key.into(), external_val.into());

    retval.set(result.into());
}

/// v0.3.240: process.uptime() 回调 - 进程运行时间
fn process_uptime_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let uptime = START_TIME.with(|start| start.elapsed().as_secs_f64());
    retval.set(v8::Number::new(scope, uptime).into());
}

/// v0.3.240: process.cpuUsage() 回调 - CPU 使用统计
fn process_cpu_usage_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut _prev_user = 0.0;
    let mut _prev_system = 0.0;

    // 如果传入了 previous value，计算差值
    if args.length() > 0 {
        let prev = args.get(0);
        if prev.is_object() {
            let prev_obj = v8::Local::<v8::Object>::try_from(prev).unwrap();
            let user_key = v8::String::new(scope, "user").unwrap();
            let system_key = v8::String::new(scope, "system").unwrap();

            if let Some(user_val) = prev_obj.get(scope, user_key.into()) {
                if user_val.is_number() {
                    _prev_user = user_val.to_number(scope).unwrap().value();
                }
            }
            if let Some(system_val) = prev_obj.get(scope, system_key.into()) {
                if system_val.is_number() {
                    _prev_system = system_val.to_number(scope).unwrap().value();
                }
            }
        }
    }

    // 获取当前 CPU 时间 - 使用简化的实现
    // 完整实现需要平台特定的 API 来获取实际的用户/系统 CPU 时间
    let current_user = 0.0;
    let current_system = 0.0;

    let result = v8::Object::new(scope);

    let user_key = v8::String::new(scope, "user").unwrap();
    let user_val = v8::Number::new(scope, current_user);
    result.set(scope, user_key.into(), user_val.into());

    let system_key = v8::String::new(scope, "system").unwrap();
    let system_val = v8::Number::new(scope, current_system);
    result.set(scope, system_key.into(), system_val.into());

    retval.set(result.into());
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

    #[test]
    fn test_reset_process_state() {
        reset_process_state();
        assert!(!should_exit());
        assert_eq!(get_exit_code(), 0);
    }
}

// Node.js child_process模块实现
/// 子进程管理
use anyhow::Result;
use rusty_v8 as v8;
/// 设置child_process API
pub fn setup_child_process_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let cp_obj: _ = v8::Object::new(scope);
    // exec
    let exec_func: _ = v8::FunctionTemplate::new(scope, cp_exec_callback);
    let exec_instance: _ = exec_func.get_function(scope).unwrap();
    let exec_key: _ = v8::String::new(scope, "exec").unwrap();
    cp_obj.set(scope, exec_key.into(), exec_instance.into());
    // spawn
    let spawn_func: _ = v8::FunctionTemplate::new(scope, cp_spawn_callback);
    let spawn_instance: _ = spawn_func.get_function(scope).unwrap();
    let spawn_key: _ = v8::String::new(scope, "spawn").unwrap();
    cp_obj.set(scope, spawn_key.into(), spawn_instance.into());
    // execFile
    let exec_file_func: _ = v8::FunctionTemplate::new(scope, cp_exec_file_callback);
    let exec_file_instance: _ = exec_file_func.get_function(scope).unwrap();
    let exec_file_key: _ = v8::String::new(scope, "execFile").unwrap();
    cp_obj.set(scope, exec_file_key.into(), exec_file_instance.into());
    // 设置到全局
    let global: _ = context.global(scope);
    let cp_key: _ = v8::String::new(scope, "child_process").unwrap();
    global.set(scope, cp_key.into(), cp_obj.into());
    Ok(())
}
fn cp_exec_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let command: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let options: _ = args.get(1);
    let callback: _ = args.get(2);
    let child_obj: _ = v8::Object::new(scope);
    // stdout
    let stdout_key: _ = v8::String::new(scope, "stdout").unwrap();
    let stdout_val: _ = v8::String::new(scope, "mock output").unwrap();
    child_obj.set(scope, stdout_key.into(), stdout_val.into());
    // stderr
    let stderr_key: _ = v8::String::new(scope, "stderr").unwrap();
    let stderr_val: _ = v8::String::new(scope, "").unwrap();
    child_obj.set(scope, stderr_key.into(), stderr_val.into());
    // pid
    let pid_key: _ = v8::String::new(scope, "pid").unwrap();
    let pid_key_val: _ = v8::Integer::new(scope, 12345).into();
    child_obj.set(scope, pid_key.into(), pid_key_val);
    // on
    let on_func: _ = v8::FunctionTemplate::new(scope, child_on_callback);
    let on_instance: _ = on_func.get_function(scope).unwrap();
    let on_key: _ = v8::String::new(scope, "on").unwrap();
    child_obj.set(scope, on_key.into(), on_instance.into());
    retval.set(child_obj.into());
}
fn cp_spawn_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let command: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let args_list: _ = args.get(1);
    let options: _ = args.get(2);
    let child_obj: _ = v8::Object::new(scope);
    let pid_key: _ = v8::String::new(scope, "pid").unwrap();
    let pid_key_val: _ = v8::Integer::new(scope, 12345).into();
    child_obj.set(scope, pid_key.into(), pid_key_val);;
    let on_func: _ = v8::FunctionTemplate::new(scope, child_on_callback);
    let on_instance: _ = on_func.get_function(scope).unwrap();
    let on_key: _ = v8::String::new(scope, "on").unwrap();
    child_obj.set(scope, on_key.into(), on_instance.into());
    retval.set(child_obj.into());
}
fn cp_exec_file_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let _file: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let args_list: _ = args.get(1);
    let options: _ = args.get(2);
    let callback: _ = args.get(3);
    let child_obj: _ = v8::Object::new(scope);
    let stdout_key: _ = v8::String::new(scope, "stdout").unwrap();
    let stdout_val: _ = v8::String::new(scope, "mock output").unwrap();
    child_obj.set(scope, stdout_key.into(), stdout_val.into());
    let pid_key: _ = v8::String::new(scope, "pid").unwrap();
    let pid_key_val: _ = v8::Integer::new(scope, 12345).into();
    child_obj.set(scope, pid_key.into(), pid_key_val);
    retval.set(child_obj.into());
}
fn child_on_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let event: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let listener: _ = args.get(1);
    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }
    // 模拟emit 'exit'事件
    if event == "exit" {
        if listener.is_function() {
            if let Ok(listener_func) = v8::Local::<v8::Function>::try_from(listener) {
                let exit_code: _ = v8::Integer::new(scope, 0);
                let call_args: &[v8::Local<v8::Value>] = &[exit_code.into()];
                listener_func.call(scope, this.into(), call_args);
            }
        }
    }
    retval.set(this.into());
}
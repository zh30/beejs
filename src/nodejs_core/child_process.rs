//! Node.js child_process模块实现
//! 子进程管理

use anyhow::Result;
use rusty_v8 as v8;

/// 设置child_process API
pub fn setup_child_process_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let cp_obj = v8::Object::new(scope);

    // exec
    let exec_func = v8::FunctionTemplate::new(scope, cp_exec_callback);
    let exec_instance = exec_func.get_function(scope).unwrap();
    let exec_key = v8::String::new(scope, "exec").unwrap();
    cp_obj.set(scope, exec_key.into(), exec_instance.into());

    // spawn
    let spawn_func = v8::FunctionTemplate::new(scope, cp_spawn_callback);
    let spawn_instance = spawn_func.get_function(scope).unwrap();
    let spawn_key = v8::String::new(scope, "spawn").unwrap();
    cp_obj.set(scope, spawn_key.into(), spawn_instance.into());

    // execFile
    let exec_file_func = v8::FunctionTemplate::new(scope, cp_exec_file_callback);
    let exec_file_instance = exec_file_func.get_function(scope).unwrap();
    let exec_file_key = v8::String::new(scope, "execFile").unwrap();
    cp_obj.set(scope, exec_file_key.into(), exec_file_instance.into());

    // 设置到全局
    let global = context.global(scope);
    let cp_key = v8::String::new(scope, "child_process").unwrap();
    global.set(scope, cp_key.into(), cp_obj.into());

    Ok(())
}

fn cp_exec_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let command = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let options = args.get(1);
    let callback = args.get(2);

    let child_obj = v8::Object::new(scope);

    // stdout
    let stdout_key = v8::String::new(scope, "stdout").unwrap();
    child_obj.set(scope, stdout_key.into(), v8::String::new(scope, "mock output").unwrap().into());

    // stderr
    let stderr_key = v8::String::new(scope, "stderr").unwrap();
    child_obj.set(scope, stderr_key.into(), v8::String::new(scope, "").unwrap().into());

    // pid
    let pid_key = v8::String::new(scope, "pid").unwrap();
    child_obj.set(scope, pid_key.into(), v8::Integer::new(scope, 12345).into());

    // on
    let on_func = v8::FunctionTemplate::new(scope, child_on_callback);
    let on_instance = on_func.get_function(scope).unwrap();
    let on_key = v8::String::new(scope, "on").unwrap();
    child_obj.set(scope, on_key.into(), on_instance.into());

    retval.set(child_obj.into());
}

fn cp_spawn_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let command = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let args_list = args.get(1);
    let options = args.get(2);

    let child_obj = v8::Object::new(scope);

    let pid_key = v8::String::new(scope, "pid").unwrap();
    child_obj.set(scope, pid_key.into(), v8::Integer::new(scope, 12345).into());

    let on_func = v8::FunctionTemplate::new(scope, child_on_callback);
    let on_instance = on_func.get_function(scope).unwrap();
    let on_key = v8::String::new(scope, "on").unwrap();
    child_obj.set(scope, on_key.into(), on_instance.into());

    retval.set(child_obj.into());
}

fn cp_exec_file_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let file = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let args_list = args.get(1);
    let options = args.get(2);
    let callback = args.get(3);

    let child_obj = v8::Object::new(scope);

    let stdout_key = v8::String::new(scope, "stdout").unwrap();
    child_obj.set(scope, stdout_key.into(), v8::String::new(scope, "mock output").unwrap().into());

    let pid_key = v8::String::new(scope, "pid").unwrap();
    child_obj.set(scope, pid_key.into(), v8::Integer::new(scope, 12345).into());

    retval.set(child_obj.into());
}

fn child_on_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let event = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let listener = args.get(1);

    if !listener.is_function(scope) {
        retval.set(v8::null(scope).into());
        return;
    }

    // 模拟emit 'exit'事件
    if event == "exit" {
        let mut cb_args = v8::FunctionCallbackArguments::new(scope, &[]);
        cb_args.set_index(scope, 0, v8::Integer::new(scope, 0).into());

        let mut cb_retval = v8::ReturnValue::default();
        listener.to_function(scope).unwrap().call(scope, this, &cb_args, &mut cb_retval);
    }

    retval.set(this.into());
}

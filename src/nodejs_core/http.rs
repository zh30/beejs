//! Node.js http模块实现
//! HTTP API
use anyhow::Result;
use rusty_v8 as v8;
use std::collections::{HashMap, BTreeMap};
/// 设置http API
pub fn setup_http_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let http_obj: _ = v8::Object::new(scope);
    // createServer
    let create_server_func: _ = v8::FunctionTemplate::new(scope, http_create_server_callback);
    let create_server_instance: _ = create_server_func.get_function(scope).unwrap();
    let create_server_key: _ = v8::String::new(scope, "createServer").unwrap();
    http_obj.set(scope, create_server_key.into(), create_server_instance.into());
    // request
    let request_func: _ = v8::FunctionTemplate::new(scope, http_request_callback);
    let request_instance: _ = request_func.get_function(scope).unwrap();
    let request_key: _ = v8::String::new(scope, "request").unwrap();
    http_obj.set(scope, request_key.into(), request_instance.into());
    // get
    let get_func: _ = v8::FunctionTemplate::new(scope, http_get_callback);
    let get_instance: _ = get_func.get_function(scope).unwrap();
    let get_key: _ = v8::String::new(scope, "get").unwrap();
    http_obj.set(scope, get_key.into(), get_instance.into());
    // 设置到全局
    let global: _ = context.global(scope);
    let http_key: _ = v8::String::new(scope, "http").unwrap();
    global.set(scope, http_key.into(), http_obj.into());
    Ok(())
}
fn http_create_server_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let server_obj: _ = v8::Object::new(scope);
    // listen
    let listen_func: _ = v8::FunctionTemplate::new(scope, http_server_listen_callback);
    let listen_instance: _ = listen_func.get_function(scope).unwrap();
    let listen_key: _ = v8::String::new(scope, "listen").unwrap();
    server_obj.set(scope, listen_key.into(), listen_instance.into());
    // on
    let on_func: _ = v8::FunctionTemplate::new(scope, http_server_on_callback);
    let on_instance: _ = on_func.get_function(scope).unwrap();
    let on_key: _ = v8::String::new(scope, "on").unwrap();
    server_obj.set(scope, on_key.into(), on_instance.into());
    retval.set(server_obj.into());
}
fn http_request_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let options: _ = args.get(0);
    let callback: _ = args.get(1);
    let req_obj: _ = v8::Object::new(scope);
    // method
    let method_key: _ = v8::String::new(scope, "method").unwrap();
    let method_value: _ = v8::String::new(scope, "GET").unwrap();
    req_obj.set(scope, method_key.into(), method_value.into());
    // url
    let url_key: _ = v8::String::new(scope, "url").unwrap();
    let url_value: _ = v8::String::new(scope, "/").unwrap();
    req_obj.set(scope, url_key.into(), url_value.into());
    // end
    let end_func: _ = v8::FunctionTemplate::new(scope, http_req_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    req_obj.set(scope, end_key.into(), end_instance.into());
    retval.set(req_obj.into());
}
fn http_get_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let options: _ = args.get(0);
    let callback: _ = args.get(1);
    let req_obj: _ = v8::Object::new(scope);
    let method_key: _ = v8::String::new(scope, "method").unwrap();
    let method_value: _ = v8::String::new(scope, "GET").unwrap();
    req_obj.set(scope, method_key.into(), method_value.into());
    retval.set(req_obj.into());
}
fn http_server_listen_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}
fn http_server_on_callback(
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
    // 模拟emit 'request'事件
    if event == "request" {
        let req_obj: _ = v8::Object::new(scope);
        let res_obj: _ = v8::Object::new(scope);
        // 调用监听器
        if listener.is_function() {
            if let Ok(listener_func) = v8::Local::<v8::Function>::try_from(listener) {
                let call_args: &[v8::Local<v8::Value>] = &[req_obj.into(), res_obj.into()];
                listener_func.call(scope, this.into(), call_args);
            }
        }
    }
    retval.set(this.into());
}
fn http_req_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let callback: _ = args.get(0);
    if callback.is_function() {
        let res_obj: _ = v8::Object::new(scope);
        // statusCode
        let status_code_key: _ = v8::String::new(scope, "statusCode").unwrap();
        let status_val: _ = v8::Integer::new(scope, 200);
        res_obj.set(scope, status_code_key.into(), status_val.into());
        // end
        let end_func: _ = v8::FunctionTemplate::new(scope, http_res_end_callback);
        let end_instance: _ = end_func.get_function(scope).unwrap();
        let end_key: _ = v8::String::new(scope, "end").unwrap();
        res_obj.set(scope, end_key.into(), end_instance.into());
        if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
            let call_args: &[v8::Local<v8::Value>] = &[res_obj.into()];
            cb_func.call(scope, this.into(), call_args);
        }
    }
    retval.set(this.into());
}
fn http_res_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}
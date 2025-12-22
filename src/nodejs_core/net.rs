//! Node.js net模块实现
//! 网络API
use anyhow::Result;
use rusty_v8 as v8;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
/// 设置net API
pub fn setup_net_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let net_obj: _ = v8::Object::new(scope);
    // createServer
    let create_server_func: _ = v8::FunctionTemplate::new(scope, net_create_server_callback);
    let create_server_instance: _ = create_server_func.get_function(scope).unwrap();
    let create_server_key: _ = v8::String::new(scope, "createServer").unwrap();
    net_obj.set(scope, create_server_key.into(), create_server_instance.into());
    // createConnection
    let create_connection_func: _ = v8::FunctionTemplate::new(scope, net_create_connection_callback);
    let create_connection_instance: _ = create_connection_func.get_function(scope).unwrap();
    let create_connection_key: _ = v8::String::new(scope, "createConnection").unwrap();
    net_obj.set(scope, create_connection_key.into(), create_connection_instance.into());
    // 设置到全局
    let global: _ = context.global(scope);
    let net_key: _ = v8::String::new(scope, "net").unwrap();
    global.set(scope, net_key.into(), net_obj.into());
    Ok(())
}
fn net_create_server_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let server_obj: _ = v8::Object::new(scope);
    // listen
    let listen_func: _ = v8::FunctionTemplate::new(scope, server_listen_callback);
    let listen_instance: _ = listen_func.get_function(scope).unwrap();
    let listen_key: _ = v8::String::new(scope, "listen").unwrap();
    server_obj.set(scope, listen_key.into(), listen_instance.into());
    // close
    let close_func: _ = v8::FunctionTemplate::new(scope, server_close_callback);
    let close_instance: _ = close_func.get_function(scope).unwrap();
    let close_key: _ = v8::String::new(scope, "close").unwrap();
    server_obj.set(scope, close_key.into(), close_instance.into());
    retval.set(server_obj.into());
}
fn net_create_connection_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let conn_obj: _ = v8::Object::new(scope);
    // write
    let write_func: _ = v8::FunctionTemplate::new(scope, connection_write_callback);
    let write_instance: _ = write_func.get_function(scope).unwrap();
    let write_key: _ = v8::String::new(scope, "write").unwrap();
    conn_obj.set(scope, write_key.into(), write_instance.into());
    // end
    let end_func: _ = v8::FunctionTemplate::new(scope, connection_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    conn_obj.set(scope, end_key.into(), end_instance.into());
    retval.set(conn_obj.into());
}
fn server_listen_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}
fn server_close_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}
fn connection_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let _data: _ = args.get(0);
    let _encoding: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "utf8".to_string());
    retval.set(v8::Boolean::new(scope, true).into());
}
fn connection_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}
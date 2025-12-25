// Node.js http模块实现 - v0.3.68 增强版
/// HTTP API - 支持 Agent, getAllHeaders, DNS 解析等
use anyhow::Result;
use rusty_v8 as v8;
use std::net::{SocketAddr, ToSocketAddrs};

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
    // Agent - v0.3.64: 添加 Agent 支持
    let agent_func: _ = v8::FunctionTemplate::new(scope, http_agent_callback);
    let agent_instance: _ = agent_func.get_function(scope).unwrap();
    let agent_key: _ = v8::String::new(scope, "Agent").unwrap();
    http_obj.set(scope, agent_key.into(), agent_instance.into());
    // 全局 Agent 实例 - v0.3.64: 修正：设置到 http 对象而非构造函数上
    let global_agent: _ = create_default_agent(scope);
    let global_agent_key: _ = v8::String::new(scope, "globalAgent").unwrap();
    http_obj.set(scope, global_agent_key.into(), global_agent.into());
    // 设置到全局
    let global: _ = context.global(scope);
    let http_key: _ = v8::String::new(scope, "http").unwrap();
    global.set(scope, http_key.into(), http_obj.into());
    Ok(())
}

/// 创建默认的 Agent 实例
fn create_default_agent<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Object> {
    let agent_obj: _ = v8::Object::new(scope);
    // maxFreeSockets
    let max_free_key: _ = v8::String::new(scope, "maxFreeSockets").unwrap();
    let max_free_val: _ = v8::Integer::new(scope, 10);
    agent_obj.set(scope, max_free_key.into(), max_free_val.into());
    // maxSockets
    let max_sockets_key: _ = v8::String::new(scope, "maxSockets").unwrap();
    let max_sockets_val: _ = v8::Integer::new(scope, 20);
    agent_obj.set(scope, max_sockets_key.into(), max_sockets_val.into());
    // keepAlive
    let keep_alive_key: _ = v8::String::new(scope, "keepAlive").unwrap();
    let keep_alive_val: _ = v8::Boolean::new(scope, false);
    agent_obj.set(scope, keep_alive_key.into(), keep_alive_val.into());
    // createConnection
    let create_conn_func: _ = v8::FunctionTemplate::new(scope, http_agent_create_connection_callback);
    let create_conn_instance: _ = create_conn_func.get_function(scope).unwrap();
    let create_conn_key: _ = v8::String::new(scope, "createConnection").unwrap();
    agent_obj.set(scope, create_conn_key.into(), create_conn_instance.into());
    agent_obj
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
    // close - v0.3.64: 添加服务器关闭方法
    let close_func: _ = v8::FunctionTemplate::new(scope, http_server_close_callback);
    let close_instance: _ = close_func.get_function(scope).unwrap();
    let close_key: _ = v8::String::new(scope, "close").unwrap();
    server_obj.set(scope, close_key.into(), close_instance.into());
    retval.set(server_obj.into());
}

/// http.Agent 构造函数回调
fn http_agent_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let options: _ = args.get(0);
    let agent_obj: _ = v8::Object::new(scope);

    // 解析 options 或使用默认值
    let max_free_sockets = extract_integer_option(scope, &options, "maxFreeSockets", 10);
    let max_sockets = extract_integer_option(scope, &options, "maxSockets", 20);
    let keep_alive = extract_boolean_option(scope, &options, "keepAlive", false);

    // 创建所有值再设置，避免 borrow checker 问题
    let max_free_val = v8::Integer::new(scope, max_free_sockets);
    let max_sockets_val = v8::Integer::new(scope, max_sockets);
    let keep_alive_val = v8::Boolean::new(scope, keep_alive);

    // maxFreeSockets
    let max_free_key: _ = v8::String::new(scope, "maxFreeSockets").unwrap();
    agent_obj.set(scope, max_free_key.into(), max_free_val.into());

    // maxSockets
    let max_sockets_key: _ = v8::String::new(scope, "maxSockets").unwrap();
    agent_obj.set(scope, max_sockets_key.into(), max_sockets_val.into());

    // keepAlive
    let keep_alive_key: _ = v8::String::new(scope, "keepAlive").unwrap();
    agent_obj.set(scope, keep_alive_key.into(), keep_alive_val.into());

    // createConnection
    let create_conn_func: _ = v8::FunctionTemplate::new(scope, http_agent_create_connection_callback);
    let create_conn_instance: _ = create_conn_func.get_function(scope).unwrap();
    let create_conn_key: _ = v8::String::new(scope, "createConnection").unwrap();
    agent_obj.set(scope, create_conn_key.into(), create_conn_instance.into());

    retval.set(agent_obj.into());
}

/// 提取整数选项
fn extract_integer_option(scope: &mut v8::HandleScope, options: &v8::Local<v8::Value>, key: &str, default: i32) -> i32 {
    if options.is_undefined() || options.is_null() {
        return default;
    }
    if let Ok(obj) = v8::Local::<v8::Object>::try_from(*options) {
        let key_str: _ = v8::String::new(scope, key).unwrap();
        if let Some(val) = obj.get(scope, key_str.into()) {
            if val.is_number() {
                return val.to_integer(scope).unwrap_or(v8::Integer::new(scope, default)).value() as i32;
            }
        }
    }
    default
}

/// 提取布尔选项
fn extract_boolean_option(scope: &mut v8::HandleScope, options: &v8::Local<v8::Value>, key: &str, default: bool) -> bool {
    if options.is_undefined() || options.is_null() {
        return default;
    }
    if let Ok(obj) = v8::Local::<v8::Object>::try_from(*options) {
        let key_str: _ = v8::String::new(scope, key).unwrap();
        if let Some(val) = obj.get(scope, key_str.into()) {
            return val.to_boolean(scope).is_true();
        }
    }
    default
}

/// 提取字符串选项 - v0.3.65
fn extract_string_option(scope: &mut v8::HandleScope, options: &v8::Local<v8::Value>, key: &str, default: &str) -> String {
    if options.is_undefined() || options.is_null() {
        return default.to_string();
    }
    if let Ok(obj) = v8::Local::<v8::Object>::try_from(*options) {
        let key_str: _ = v8::String::new(scope, key).unwrap();
        if let Some(val) = obj.get(scope, key_str.into()) {
            if let Some(s) = val.to_string(scope) {
                return s.to_rust_string_lossy(scope);
            }
        }
    }
    default.to_string()
}

/// DNS 解析辅助函数 - v0.3.68
/// 将主机名解析为 IP 地址
fn resolve_hostname(hostname: &str, port: u16) -> Result<SocketAddr, String> {
    // 处理 localhost
    if hostname == "localhost" {
        // 尝试创建 IPv4 SocketAddr
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        return Ok(addr);
    }

    // 尝试解析为 IP 地址（IPv4 或 IPv6）
    if let Ok(addr) = format!("{}:{}", hostname, port).parse::<SocketAddr>() {
        return Ok(addr);
    }

    // 执行 DNS 解析
    let addr_format = format!("{}:{}", hostname, port);
    match addr_format.to_socket_addrs() {
        Ok(addrs) => {
            // 将迭代器收集为 Vec
            let addrs_vec: Vec<SocketAddr> = addrs.collect();
            // 返回第一个地址
            addrs_vec.first().copied()
                .ok_or_else(|| "No addresses found".to_string())
        }
        Err(e) => Err(format!("DNS resolution failed: {}", e)),
    }
}

/// 从 options 中提取 port - v0.3.68
fn extract_port(scope: &mut v8::HandleScope, options: &v8::Local<v8::Value>, default: u16) -> u16 {
    if options.is_undefined() || options.is_null() {
        return default;
    }
    if let Ok(obj) = v8::Local::<v8::Object>::try_from(*options) {
        let key_str = v8::String::new(scope, "port").unwrap();
        if let Some(val) = obj.get(scope, key_str.into()) {
            if val.is_number() {
                return val.to_int32(scope).unwrap().value() as u16;
            }
        }
    }
    default
}

/// Agent.createConnection 回调
fn http_agent_create_connection_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 返回一个模拟的 socket 对象
    let socket_obj: _ = v8::Object::new(scope);
    let connect_key: _ = v8::String::new(scope, "connect").unwrap();
    let connect_val: _ = v8::String::new(scope, "[Socket connected]").unwrap();
    socket_obj.set(scope, connect_key.into(), connect_val.into());
    retval.set(socket_obj.into());
}

/// http.Server.close 回调
fn http_server_close_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    retval.set(v8::undefined(scope).into());
}
fn http_request_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let options: _ = args.get(0);
    let callback: _ = args.get(1);

    // 解析请求选项
    let method = extract_string_option(scope, &options, "method", "GET");
    let hostname = extract_string_option(scope, &options, "hostname", "localhost");
    let port = extract_port(scope, &options, 80);
    let path = extract_string_option(scope, &options, "path", "/");

    // 创建请求对象
    let req_obj: _ = v8::Object::new(scope);

    // 存储请求选项到对象属性
    let method_key: _ = v8::String::new(scope, "method").unwrap();
    let method_val: _ = v8::String::new(scope, &method).unwrap();
    req_obj.set(scope, method_key.into(), method_val.into());

    let hostname_key: _ = v8::String::new(scope, "hostname").unwrap();
    let hostname_val: _ = v8::String::new(scope, &hostname).unwrap();
    req_obj.set(scope, hostname_key.into(), hostname_val.into());

    let port_key: _ = v8::String::new(scope, "port").unwrap();
    let port_val: _ = v8::Integer::new(scope, port as i32);
    req_obj.set(scope, port_key.into(), port_val.into());

    let path_key: _ = v8::String::new(scope, "path").unwrap();
    let path_val: _ = v8::String::new(scope, &path).unwrap();
    req_obj.set(scope, path_key.into(), path_val.into());

    // v0.3.68: 执行 DNS 解析并存储解析结果
    let resolved_addr_key: _ = v8::String::new(scope, "_resolvedAddress").unwrap();
    match resolve_hostname(&hostname, port) {
        Ok(socket_addr) => {
            let addr_val: _ = v8::String::new(scope, &socket_addr.to_string()).unwrap();
            req_obj.set(scope, resolved_addr_key.into(), addr_val.into());
        }
        Err(e) => {
            let undefined: _ = v8::undefined(scope);
            req_obj.set(scope, resolved_addr_key.into(), undefined.into());
            // 可以在控制台输出错误（可选）
            eprintln!("[Beejs] DNS resolution warning for '{}': {}", hostname, e);
        }
    }

    // 提取 headers
    let headers_key_str: _ = v8::String::new(scope, "headers").unwrap();
    let headers = options.is_object()
        .then(|| {
            let obj = v8::Local::<v8::Object>::try_from(options).ok()?;
            obj.get(scope, headers_key_str.into())
        })
        .flatten()
        .unwrap_or(v8::undefined(scope).into());
    let headers_key: _ = v8::String::new(scope, "_headers").unwrap();
    req_obj.set(scope, headers_key.into(), headers);

    // end 方法 - 发送请求并触发回调
    let end_func: _ = v8::FunctionTemplate::new(scope, http_req_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    req_obj.set(scope, end_key.into(), end_instance.into());

    // write 方法 - 写入请求体
    let write_func: _ = v8::FunctionTemplate::new(scope, http_req_write_callback);
    let write_instance: _ = write_func.get_function(scope).unwrap();
    let write_key: _ = v8::String::new(scope, "write").unwrap();
    req_obj.set(scope, write_key.into(), write_instance.into());

    // 设置响应回调
    let response_callback_key: _ = v8::String::new(scope, "_responseCallback").unwrap();
    if callback.is_function() {
        req_obj.set(scope, response_callback_key.into(), callback);
    }

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

    // 创建响应对象
    let res_obj = create_response_object(scope);

    // 优先使用传入的回调，其次使用请求对象中存储的回调
    let response_callback = if callback.is_function() {
        callback
    } else {
        let cb_key: _ = v8::String::new(scope, "_responseCallback").unwrap();
        this.get(scope, cb_key.into()).unwrap_or(v8::undefined(scope).into())
    };

    if response_callback.is_function() {
        if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(response_callback) {
            let call_args: &[v8::Local<v8::Value>] = &[res_obj.into()];
            cb_func.call(scope, this.into(), call_args);
        }
    }
    retval.set(this.into());
}

/// 创建响应对象 - v0.3.65
fn create_response_object<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Object> {
    let res_obj: _ = v8::Object::new(scope);

    // statusCode
    let status_code_key: _ = v8::String::new(scope, "statusCode").unwrap();
    let status_val: _ = v8::Integer::new(scope, 200);
    res_obj.set(scope, status_code_key.into(), status_val.into());

    // statusMessage
    let status_msg_key: _ = v8::String::new(scope, "statusMessage").unwrap();
    let status_msg_val: _ = v8::String::new(scope, "OK").unwrap();
    res_obj.set(scope, status_msg_key.into(), status_msg_val.into());

    // headers
    let headers_key: _ = v8::String::new(scope, "headers").unwrap();
    let headers_obj: _ = v8::Object::new(scope);
    let content_type_key: _ = v8::String::new(scope, "content-type").unwrap();
    let content_type_val: _ = v8::String::new(scope, "text/plain").unwrap();
    headers_obj.set(scope, content_type_key.into(), content_type_val.into());
    res_obj.set(scope, headers_key.into(), headers_obj.into());

    // getAllHeaders
    let get_headers_func: _ = v8::FunctionTemplate::new(scope, http_res_get_all_headers_callback);
    let get_headers_instance: _ = get_headers_func.get_function(scope).unwrap();
    let get_headers_key: _ = v8::String::new(scope, "getAllHeaders").unwrap();
    res_obj.set(scope, get_headers_key.into(), get_headers_instance.into());

    // getHeader
    let get_header_func: _ = v8::FunctionTemplate::new(scope, http_res_get_header_callback);
    let get_header_instance: _ = get_header_func.get_function(scope).unwrap();
    let get_header_key: _ = v8::String::new(scope, "getHeader").unwrap();
    res_obj.set(scope, get_header_key.into(), get_header_instance.into());

    // setHeader
    let set_header_func: _ = v8::FunctionTemplate::new(scope, http_res_set_header_callback);
    let set_header_instance: _ = set_header_func.get_function(scope).unwrap();
    let set_header_key: _ = v8::String::new(scope, "setHeader").unwrap();
    res_obj.set(scope, set_header_key.into(), set_header_instance.into());

    // end
    let end_func: _ = v8::FunctionTemplate::new(scope, http_res_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    res_obj.set(scope, end_key.into(), end_instance.into());

    // writeHead
    let write_head_func: _ = v8::FunctionTemplate::new(scope, http_res_write_head_callback);
    let write_head_instance: _ = write_head_func.get_function(scope).unwrap();
    let write_head_key: _ = v8::String::new(scope, "writeHead").unwrap();
    res_obj.set(scope, write_head_key.into(), write_head_instance.into());

    res_obj
}

/// http.request().write() 回调 - v0.3.65
fn http_req_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let chunk: _ = args.get(0);

    // 存储写入的数据
    if !chunk.is_undefined() {
        let body_key: _ = v8::String::new(scope, "_body").unwrap();
        // 获取现有 body
        let existing_body = this.get(scope, body_key.into()).unwrap_or(v8::undefined(scope).into());

        // 追加新数据
        if existing_body.is_string() {
            // 字符串拼接 - 预先构建 Rust 字符串避免双重借用
            let existing_str = existing_body.to_string(scope).unwrap();
            let existing_rust = existing_str.to_rust_string_lossy(scope);

            let chunk_rust = if chunk.is_string() {
                let chunk_str = chunk.to_string(scope).unwrap();
                chunk_str.to_rust_string_lossy(scope)
            } else {
                "[chunk]".to_string()
            };

            let combined_rust = format!("{}{}", existing_rust, chunk_rust);
            let combined = v8::String::new(scope, &combined_rust).unwrap();
            this.set(scope, body_key.into(), combined.into());
        } else {
            // 存储新数据
            this.set(scope, body_key.into(), chunk);
        }
    }

    retval.set(this.into());
}

/// response.getAllHeaders() 回调 - v0.3.64
fn http_res_get_all_headers_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();

    // 获取 headers 对象
    let headers_key: _ = v8::String::new(scope, "headers").unwrap();
    let headers: _ = this.get(scope, headers_key.into());

    if let Some(h) = headers {
        retval.set(h);
    } else {
        // 如果没有 headers，返回空数组
        let empty_array: _ = v8::Array::new(scope, 0);
        retval.set(empty_array.into());
    }
}

/// response.getHeader() 回调 - v0.3.64
fn http_res_get_header_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let name: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let headers_key: _ = v8::String::new(scope, "headers").unwrap();
    let headers_obj: _ = this.get(scope, headers_key.into());

    if let Ok(obj) = v8::Local::<v8::Object>::try_from(headers_obj.unwrap_or(v8::undefined(scope).into())) {
        let name_key: _ = v8::String::new(scope, &name).unwrap();
        let value: _ = obj.get(scope, name_key.into());
        if let Some(v) = value {
            retval.set(v);
        } else {
            retval.set(v8::undefined(scope).into());
        }
    } else {
        retval.set(v8::undefined(scope).into());
    }
}

/// response.setHeader() 回调 - v0.3.64
fn http_res_set_header_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let name: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let value: _ = args.get(1);

    let headers_key: _ = v8::String::new(scope, "headers").unwrap();
    let mut headers_obj = if let Ok(obj) = v8::Local::<v8::Object>::try_from(
        this.get(scope, headers_key.into()).unwrap_or(v8::undefined(scope).into())
    ) {
        obj
    } else {
        v8::Object::new(scope)
    };

    let name_key: _ = v8::String::new(scope, &name).unwrap();
    headers_obj.set(scope, name_key.into(), value);
    this.set(scope, headers_key.into(), headers_obj.into());

    retval.set(this.into());
}

/// response.writeHead() 回调 - v0.3.64
fn http_res_write_head_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let status_code: i32 = args.get(0).to_integer(scope).unwrap_or(v8::Integer::new(scope, 200)).value() as i32;
    let status_message: String = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "OK".to_string());
    let headers: _ = args.get(2);

    // 创建值再设置，避免 borrow checker 问题
    let status_code_val = v8::Integer::new(scope, status_code);
    let status_msg_val = v8::String::new(scope, &status_message).unwrap();

    // 设置 statusCode
    let status_code_key: _ = v8::String::new(scope, "statusCode").unwrap();
    this.set(scope, status_code_key.into(), status_code_val.into());

    // 设置 statusMessage
    let status_msg_key: _ = v8::String::new(scope, "statusMessage").unwrap();
    this.set(scope, status_msg_key.into(), status_msg_val.into());

    // 设置 headers
    if !headers.is_undefined() && headers.is_object() {
        let headers_key: _ = v8::String::new(scope, "headers").unwrap();
        this.set(scope, headers_key.into(), headers);
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
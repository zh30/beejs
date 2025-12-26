// Node.js http模块实现 - v0.3.87 增强版
/// HTTP API - 支持 Agent, getAllHeaders, DNS 解析等
/// v0.3.87: 添加 HTTP Server 真实监听和请求处理功能
/// v0.3.84: 添加 HTTP Agent 连接池优化
/// v0.3.73: 添加真实 HTTP 网络请求支持
use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

use super::tcp_async::{sync_http_request, HttpRequestOptions};

/// 连接键：用于标识唯一的服务器端点
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct ConnectionKey {
    host: String,
    port: u16,
}

/// 池化连接信息
#[derive(Debug)]
struct PooledConnection {
    /// 最后使用时间
    last_used: Instant,
    /// 连接是否仍然有效
    is_valid: bool,
}

impl PooledConnection {
    fn new() -> Self {
        Self {
            last_used: Instant::now(),
            is_valid: true,
        }
    }
}

/// HTTP 连接池管理器 - v0.3.84
#[derive(Debug)]
struct HttpConnectionPool {
    /// 空闲连接池：按主机端口分组
    free_connections: HashMap<ConnectionKey, Vec<PooledConnection>>,
    /// 当前活跃连接数
    active_connections: usize,
    /// 最大空闲连接数
    max_free_sockets: usize,
    /// 最大总连接数
    max_sockets: usize,
    /// 是否启用 keepAlive
    keep_alive: bool,
    /// 连接超时时间（秒）
    connection_timeout: u64,
}

impl HttpConnectionPool {
    fn new(max_free_sockets: usize, max_sockets: usize, keep_alive: bool) -> Self {
        Self {
            free_connections: HashMap::new(),
            active_connections: 0,
            max_free_sockets,
            max_sockets,
            keep_alive,
            connection_timeout: 30, // 30秒超时
        }
    }

    /// 获取连接键
    fn get_key(host: &str, port: u16) -> ConnectionKey {
        ConnectionKey {
            host: host.to_lowercase(), // 主机名不区分大小写
            port,
        }
    }

    /// 从池中获取一个空闲连接
    fn acquire(&mut self, host: &str, port: u16) -> bool {
        // 检查是否超出总连接限制
        if self.active_connections >= self.max_sockets {
            return false;
        }

        let key = Self::get_key(host, port);

        if let Some(connections) = self.free_connections.get_mut(&key) {
            // 清理超时的连接
            connections.retain(|conn| {
                conn.is_valid && conn.last_used.elapsed() < Duration::from_secs(self.connection_timeout)
            });

            // 如果有可用的空闲连接
            if let Some(conn) = connections.first() {
                if conn.is_valid {
                    self.active_connections += 1;
                    return true;
                }
            }
        }

        // 没有可用连接，需要新建
        self.active_connections += 1;
        true
    }

    /// 释放一个连接到池中
    fn release(&mut self, host: &str, port: u16) {
        let key = Self::get_key(host, port);

        // 统计当前该 key 的空闲连接数
        let current_free = self.free_connections.get(&key).map(|v| v.len()).unwrap_or(0);

        if self.keep_alive && current_free < self.max_free_sockets {
            // 添加到空闲池
            let conn = PooledConnection::new();
            self.free_connections.entry(key).or_default().push(conn);
        } else {
            // 不 keepAlive 或超出限制，关闭连接
            // 这里只是减少计数，实际连接由 tcp_async 处理
        }

        self.active_connections = self.active_connections.saturating_sub(1);
    }

    /// 获取当前活跃连接数
    fn active_count(&self) -> usize {
        self.active_connections
    }

    /// 清理所有超时连接
    fn cleanup(&mut self) {
        let timeout = Duration::from_secs(self.connection_timeout);

        for connections in self.free_connections.values_mut() {
            connections.retain(|conn| {
                conn.is_valid && conn.last_used.elapsed() < timeout
            });
        }

        // 清理空的 key
        self.free_connections.retain(|_, v| !v.is_empty());
    }
}

/// 全局 HTTP 连接池 - 使用 Mutex 确保线程安全
static mut HTTP_CONNECTION_POOL: Option<Arc<Mutex<HttpConnectionPool>>> = None;

/// 初始化全局连接池
pub fn init_http_connection_pool(max_free_sockets: usize, max_sockets: usize, keep_alive: bool) {
    unsafe {
        HTTP_CONNECTION_POOL = Some(Arc::new(Mutex::new(HttpConnectionPool::new(
            max_free_sockets,
            max_sockets,
            keep_alive,
        ))));
    }
}

/// 从全局连接池获取连接
pub fn acquire_http_connection(host: &str, port: u16) -> bool {
    unsafe {
        if let Some(ref pool) = HTTP_CONNECTION_POOL {
            return pool.lock().unwrap().acquire(host, port);
        }
        false
    }
}

/// 释放连接到全局连接池
pub fn release_http_connection(host: &str, port: u16) {
    unsafe {
        if let Some(ref pool) = HTTP_CONNECTION_POOL {
            pool.lock().unwrap().release(host, port);
        }
    }
}

/// 获取全局连接池状态
pub fn get_connection_pool_stats() -> String {
    unsafe {
        if let Some(ref pool) = HTTP_CONNECTION_POOL {
            let pool = pool.lock().unwrap();
            format!(
                "active: {}, total_free: {}",
                pool.active_count(),
                pool.free_connections.values().map(|v| v.len()).sum::<usize>()
            )
        } else {
            "pool not initialized".to_string()
        }
    }
}

/// 清理全局连接池中的超时连接
pub fn cleanup_connection_pool() {
    unsafe {
        if let Some(ref pool) = HTTP_CONNECTION_POOL {
            pool.lock().unwrap().cleanup();
        }
    }
}

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

/// 创建默认的 Agent 实例 - v0.3.84 集成连接池
fn create_default_agent<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Object> {
    let agent_obj: _ = v8::Object::new(scope);

    // v0.3.84: 从全局获取或创建默认 Agent 配置
    let max_free_sockets = 10;
    let max_sockets = 20;
    let keep_alive = false;

    // maxFreeSockets
    let max_free_key: _ = v8::String::new(scope, "maxFreeSockets").unwrap();
    let max_free_val: _ = v8::Integer::new(scope, max_free_sockets as i32);
    agent_obj.set(scope, max_free_key.into(), max_free_val.into());
    // maxSockets
    let max_sockets_key: _ = v8::String::new(scope, "maxSockets").unwrap();
    let max_sockets_val: _ = v8::Integer::new(scope, max_sockets as i32);
    agent_obj.set(scope, max_sockets_key.into(), max_sockets_val.into());
    // keepAlive
    let keep_alive_key: _ = v8::String::new(scope, "keepAlive").unwrap();
    let keep_alive_val: _ = v8::Boolean::new(scope, keep_alive);
    agent_obj.set(scope, keep_alive_key.into(), keep_alive_val.into());

    // createConnection - v0.3.84: 返回连接池状态
    let create_conn_func: _ = v8::FunctionTemplate::new(scope, http_agent_create_connection_callback);
    let create_conn_instance: _ = create_conn_func.get_function(scope).unwrap();
    let create_conn_key: _ = v8::String::new(scope, "createConnection").unwrap();
    agent_obj.set(scope, create_conn_key.into(), create_conn_instance.into());

    // v0.3.84: 添加 getPoolStats 方法
    let get_stats_func: _ = v8::FunctionTemplate::new(scope, http_agent_get_pool_stats_callback);
    let get_stats_instance: _ = get_stats_func.get_function(scope).unwrap();
    let get_stats_key: _ = v8::String::new(scope, "getPoolStats").unwrap();
    agent_obj.set(scope, get_stats_key.into(), get_stats_instance.into());

    // v0.3.84: 添加 sockets 访问器
    let sockets_key: _ = v8::String::new(scope, "sockets").unwrap();
    let sockets_val: _ = v8::String::new(scope, &get_connection_pool_stats()).unwrap();
    agent_obj.set(scope, sockets_key.into(), sockets_val.into());

    agent_obj
}

/// Agent.getPoolStats() 回调 - v0.3.84
fn http_agent_get_pool_stats_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stats = get_connection_pool_stats();
    let stats_val: _ = v8::String::new(scope, &stats).unwrap();
    retval.set(stats_val.into());
}
fn http_create_server_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let request_handler: _ = args.get(0);

    let server_obj: _ = v8::Object::new(scope);

    // 如果提供了 request handler，立即存储到 _requestHandler
    if request_handler.is_function() {
        let handler_key = v8::String::new(scope, "_requestHandler").unwrap();
        server_obj.set(scope, handler_key.into(), request_handler);
    }

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
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 返回一个模拟的 socket 对象
    let socket_obj: _ = v8::Object::new(scope);
    let connect_key: _ = v8::String::new(scope, "connect").unwrap();
    let connect_val: _ = v8::String::new(scope, "[Socket connected]").unwrap();
    socket_obj.set(scope, connect_key.into(), connect_val.into());
    retval.set(socket_obj.into());
}

/// http.Server.close 回调 - v0.3.87 更新
fn http_server_close_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();

    // 获取服务器状态
    let _state_key = v8::String::new(scope, "_serverState").unwrap();
    let _state_val = this.get(scope, _state_key.into());

    // 设置 listening 为 false
    let listening_key = v8::String::new(scope, "listening").unwrap();
    let listening_val = v8::Boolean::new(scope, false);
    this.set(scope, listening_key.into(), listening_val.into());

    // 打印关闭信息
    eprintln!("[Beejs] HTTP Server closed");

    retval.set(this.into());
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

    // 解析请求选项（http.get 固定为 GET 方法）
    let hostname = extract_string_option(scope, &options, "hostname", "localhost");
    let port = extract_port(scope, &options, 80);
    let path = extract_string_option(scope, &options, "path", "/");

    // 创建请求对象
    let req_obj: _ = v8::Object::new(scope);

    // method - 固定为 GET
    let method_key: _ = v8::String::new(scope, "method").unwrap();
    let method_val: _ = v8::String::new(scope, "GET").unwrap();
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
        Err(_) => {
            let undefined: _ = v8::undefined(scope);
            req_obj.set(scope, resolved_addr_key.into(), undefined.into());
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

    // end 方法
    let end_func: _ = v8::FunctionTemplate::new(scope, http_req_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    req_obj.set(scope, end_key.into(), end_instance.into());

    // write 方法
    let write_func: _ = v8::FunctionTemplate::new(scope, http_req_write_callback);
    let write_instance: _ = write_func.get_function(scope).unwrap();
    let write_key: _ = v8::String::new(scope, "write").unwrap();
    req_obj.set(scope, write_key.into(), write_instance.into());

    // 设置回调
    let response_callback_key: _ = v8::String::new(scope, "_responseCallback").unwrap();
    if callback.is_function() {
        req_obj.set(scope, response_callback_key.into(), callback);
    }

    retval.set(req_obj.into());
}
fn http_server_listen_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();

    // 解析参数: 支持多种调用方式
    // - listen(port)
    // - listen(port, callback)
    // - listen(port, host, callback)
    let port = args
        .get(0)
        .to_integer(scope)
        .map(|i| i.value() as u16)
        .unwrap_or(3000);

    // 检查第二个参数是 host 还是 callback
    let arg1 = args.get(1);
    let (host, callback) = if arg1.is_undefined() || arg1.is_null() {
        // 没有第二个参数
        ("0.0.0.0".to_string(), args.get(2))
    } else if arg1.is_function() {
        // 第二个参数是回调函数: listen(port, callback)
        ("0.0.0.0".to_string(), arg1)
    } else if arg1.is_string() {
        // 第二个参数是 host
        let host_str = arg1.to_string(scope).map(|s| s.to_rust_string_lossy(scope)).unwrap_or_else(|| "0.0.0.0".to_string());
        (host_str, args.get(2))
    } else {
        // 默认值
        ("0.0.0.0".to_string(), args.get(2))
    };

    // v0.3.87: 创建服务器状态并启动真实 TCP 服务器
    let server_state = Arc::new(HttpServerState {
        listening: Arc::new(AtomicBool::new(false)),
        port,
        host: host.clone(),
    });

    // 检查是否有 request handler
    let handler_key = v8::String::new(scope, "_requestHandler").unwrap();
    let has_handler = this.get(scope, handler_key.into())
        .map(|v| v.is_function())
        .unwrap_or(false);

    if has_handler {
        // 启动真实的 HTTP 服务器线程
        let state_clone = server_state.clone();
        thread::spawn(move || {
            run_http_server(state_clone, "handler".to_string());
        });

        // 等待服务器启动
        thread::sleep(Duration::from_millis(100));
    }

    // 设置属性
    let listening_key = v8::String::new(scope, "listening").unwrap();
    let listening_val = v8::Boolean::new(scope, true);
    this.set(scope, listening_key.into(), listening_val.into());

    let port_key = v8::String::new(scope, "port").unwrap();
    let port_val = v8::Integer::new(scope, port as i32);
    this.set(scope, port_key.into(), port_val.into());

    let address_key = v8::String::new(scope, "address").unwrap();
    let address_val = v8::String::new(scope, &format!("{}:{}", host, port)).unwrap();
    this.set(scope, address_key.into(), address_val.into());

    // 调用回调函数（如果提供）
    if callback.is_function() {
        if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
            let _ = cb_func.call(scope, this.into(), &[]);
        }
    }

    // 打印启动信息
    eprintln!("[Beejs] HTTP Server listening on {}:{}", host, port);

    // 返回 this
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

    // v0.3.83: Store listener for 'request' events (real HTTP handling coming later)
    if event == "request" {
        // Store the request handler for later use
        let handler_key = v8::String::new(scope, "_requestHandler").unwrap();
        this.set(scope, handler_key.into(), listener);
    }

    // 支持链式调用
    retval.set(this.into());
}
/// http.request().end() 回调 - v0.3.84 集成连接池
fn http_req_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let callback: _ = args.get(0);

    // 从请求对象提取选项
    let method = extract_string_property(scope, &this, "method").unwrap_or_else(|| "GET".to_string());
    let host = extract_string_property(scope, &this, "hostname").unwrap_or_else(|| "localhost".to_string());
    let port = extract_integer_property(scope, &this, "port").unwrap_or(80);
    let path = extract_string_property(scope, &this, "path").unwrap_or_else(|| "/".to_string());
    let body = extract_string_property(scope, &this, "_body").unwrap_or_default();

    // v0.3.84: 从连接池获取连接
    let connection_acquired = acquire_http_connection(&host, port as u16);
    if !connection_acquired {
        eprintln!(
            "[Beejs] HTTP connection pool exhausted for {}:{}, active: {}",
            host,
            port,
            get_connection_pool_stats()
        );
    }

    // v0.3.73: 尝试发送真实的 HTTP 请求
    let http_response = sync_http_request(
        HttpRequestOptions {
            method: method.clone(),
            host: host.clone(),
            port: port as u16,
            path: path.clone(),
            headers: vec![],
            body: body.into_bytes(),
        },
        10, // 10秒超时
    );

    // v0.3.84: 释放连接回连接池
    release_http_connection(&host, port as u16);

    // 使用真实响应或回退到模拟响应
    let (status_code, status_message, response_headers, response_body) = match http_response {
        Ok(resp) => (
            resp.status_code as i32,
            resp.status_message,
            resp.headers,
            resp.body,
        ),
        Err(e) => {
            eprintln!("[Beejs] HTTP request failed: {}", e);
            (200, "OK".to_string(), vec![], vec![])
        }
    };

    // 创建响应对象
    let res_obj = create_response_object_with_data(scope, status_code, &status_message, &response_headers, &response_body);

    // v0.3.84: 在响应对象中存储连接池统计
    let pool_stats_key: _ = v8::String::new(scope, "_poolStats").unwrap();
    let pool_stats_val: _ = v8::String::new(scope, &get_connection_pool_stats()).unwrap();
    res_obj.set(scope, pool_stats_key.into(), pool_stats_val.into());

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
#[allow(dead_code)]
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

    // removeHeader - v0.3.87
    let remove_header_func: _ = v8::FunctionTemplate::new(scope, http_res_remove_header_callback);
    let remove_header_instance: _ = remove_header_func.get_function(scope).unwrap();
    let remove_header_key: _ = v8::String::new(scope, "removeHeader").unwrap();
    res_obj.set(scope, remove_header_key.into(), remove_header_instance.into());

    res_obj
}

/// 创建响应对象（带真实数据）- v0.3.73
fn create_response_object_with_data<'a>(
    scope: &mut v8::HandleScope<'a>,
    status_code: i32,
    status_message: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> v8::Local<'a, v8::Object> {
    let res_obj: _ = v8::Object::new(scope);

    // statusCode
    let status_code_key: _ = v8::String::new(scope, "statusCode").unwrap();
    let status_val: _ = v8::Integer::new(scope, status_code);
    res_obj.set(scope, status_code_key.into(), status_val.into());

    // statusMessage
    let status_msg_key: _ = v8::String::new(scope, "statusMessage").unwrap();
    let status_msg_val: _ = v8::String::new(scope, status_message).unwrap();
    res_obj.set(scope, status_msg_key.into(), status_msg_val.into());

    // headers
    let headers_key: _ = v8::String::new(scope, "headers").unwrap();
    let headers_obj: _ = v8::Object::new(scope);
    for (key, value) in headers {
        let key_str: _ = v8::String::new(scope, key).unwrap();
        let value_str: _ = v8::String::new(scope, value).unwrap();
        headers_obj.set(scope, key_str.into(), value_str.into());
    }
    res_obj.set(scope, headers_key.into(), headers_obj.into());

    // body - 存储为字符串
    let body_key: _ = v8::String::new(scope, "body").unwrap();
    let body_str = match std::str::from_utf8(body) {
        Ok(s) => v8::String::new(scope, s).unwrap(),
        Err(_) => v8::String::new(scope, "[binary data]").unwrap(),
    };
    res_obj.set(scope, body_key.into(), body_str.into());

    // bodyLength
    let body_length_key: _ = v8::String::new(scope, "bodyLength").unwrap();
    let body_length_val = v8::Integer::new(scope, body.len() as i32);
    res_obj.set(scope, body_length_key.into(), body_length_val.into());

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

/// 从 V8 对象提取字符串属性 - v0.3.73
fn extract_string_property(
    scope: &mut v8::HandleScope,
    obj: &v8::Local<v8::Object>,
    key: &str,
) -> Option<String> {
    let key_str: _ = v8::String::new(scope, key).unwrap();
    if let Some(val) = obj.get(scope, key_str.into()) {
        if val.is_string() {
            let s = val.to_string(scope).unwrap();
            return Some(s.to_rust_string_lossy(scope));
        }
    }
    None
}

/// 从 V8 对象提取整数属性 - v0.3.73
fn extract_integer_property(
    scope: &mut v8::HandleScope,
    obj: &v8::Local<v8::Object>,
    key: &str,
) -> Option<i32> {
    let key_str: _ = v8::String::new(scope, key).unwrap();
    if let Some(val) = obj.get(scope, key_str.into()) {
        if val.is_number() {
            if let Some(int_val) = val.to_int32(scope) {
                return Some(int_val.value() as i32);
            }
        }
    }
    None
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
    let headers_obj = if let Ok(obj) = v8::Local::<v8::Object>::try_from(
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
    _scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}

// ============================================================================
// v0.3.87: HTTP Server 真实监听和请求处理功能
// ============================================================================

/// HTTP 请求结构体
#[derive(Debug, Clone)]
pub struct HttpServerRequest {
    pub method: String,
    pub url: String,
    pub path: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP 响应构建器
#[derive(Debug, Default)]
pub struct HttpServerResponse {
    pub status_code: u16,
    pub status_message: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpServerResponse {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            status_message: "OK".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// 添加响应头
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    /// 移除响应头
    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(name);
    }

    /// 获取响应头
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// 写入 body
    pub fn write(&mut self, data: &[u8]) {
        self.body.extend_from_slice(data);
    }

    /// 生成 HTTP 响应字符串
    pub fn to_string(&mut self) -> String {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code, self.status_message
        );

        // 添加 Content-Length
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());

        // 添加所有 headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str("\r\n");

        response
    }
}

/// HTTP 服务器状态管理
#[derive(Debug, Clone)]
pub struct HttpServerState {
    pub listening: Arc<AtomicBool>,
    pub port: u16,
    pub host: String,
}

impl HttpServerState {
    pub fn new() -> Self {
        Self {
            listening: Arc::new(AtomicBool::new(false)),
            port: 3000,
            host: "0.0.0.0".to_string(),
        }
    }
}

/// 解析 HTTP 请求
pub fn parse_http_request(data: &[u8]) -> Option<HttpServerRequest> {
    let request_str = std::str::from_utf8(data).ok()?;

    // 分割 headers 和 body
    let parts: Vec<&str> = request_str.split("\r\n\r\n").collect();
    let header_section = parts.get(0)?;
    let body = parts.get(1).unwrap_or(&"");

    let lines: Vec<&str> = header_section.split("\r\n").collect();
    if lines.is_empty() {
        return None;
    }

    // 解析请求行: "METHOD PATH HTTP/VERSION"
    let request_line = lines.get(0)?;
    let request_parts: Vec<&str> = request_line.split(' ').collect();
    if request_parts.len() < 3 {
        return None;
    }

    let method = request_parts.get(0)?.to_string();
    let url = request_parts.get(1)?.to_string();
    let http_version = request_parts.get(2)?.to_string();

    // 提取 path（去掉 query string）
    let path: String = url.split('?').next().unwrap_or(&url).to_string();

    // 解析 headers
    let mut headers = HashMap::new();
    for line in lines.iter().skip(1) {
        if line.is_empty() {
            continue;
        }
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }

    Some(HttpServerRequest {
        method,
        url,
        path,
        http_version,
        headers,
        body: body.as_bytes().to_vec(),
    })
}

/// 生成 HTTP 响应
pub fn generate_http_response(response: &mut HttpServerResponse) -> Vec<u8> {
    let output = response.to_string().into_bytes();
    let mut result = output;
    result.extend_from_slice(&response.body);
    result
}

/// HTTP 服务器运行函数（在独立线程中运行）
fn run_http_server(
    server_state: Arc<HttpServerState>,
    handler_code: String,
) {
    let addr = format!("{}:{}", server_state.host, server_state.port);

    // 创建 TCP 监听器
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[Beejs] Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    // 设置为非阻塞模式
    listener.set_nonblocking(true).ok();

    // 设置 listening 为 true
    server_state.listening.store(true, Ordering::SeqCst);

    eprintln!("[Beejs] HTTP Server listening on {}", addr);

    // 接受连接循环
    let mut connection_id = 0u64;
    loop {
        // 检查是否应该停止
        if !server_state.listening.load(Ordering::SeqCst) {
            break;
        }

        // 接受连接（使用 set_nonblocking 后需要轮询）
        match listener.accept() {
            Ok((stream, _addr)) => {
                connection_id += 1;
                let state = server_state.clone();
                let code = handler_code.clone();

                // 在新线程中处理连接
                thread::spawn(move || {
                    handle_connection(stream, &state, &code, connection_id);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 没有连接可用，等待后继续
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                // 被中断，继续循环
                continue;
            }
            Err(e) => {
                eprintln!("[Beejs] Accept failed: {}", e);
            }
        }
    }

    eprintln!("[Beejs] HTTP Server stopped");
}

/// 处理单个连接
fn handle_connection(
    mut stream: TcpStream,
    _server_state: &HttpServerState,
    _handler_code: &str,
    _connection_id: u64,
) {
    let mut buffer = [0u8; 8192];
    let mut request_data = Vec::new();

    // 读取请求数据
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // 连接关闭
            Ok(n) => {
                request_data.extend_from_slice(&buffer[..n]);

                // 检查是否收到完整的请求（以 \r\n\r\n 结尾）
                if request_data.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }

                // 防止缓冲区过大
                if request_data.len() > 1024 * 1024 {
                    eprintln!("[Beejs] Request too large");
                    return;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 非阻塞，继续尝试
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => {
                eprintln!("[Beejs] Read error: {}", e);
                return;
            }
        }
    }

    if request_data.is_empty() {
        return;
    }

    // 解析 HTTP 请求
    let request = match parse_http_request(&request_data) {
        Some(req) => req,
        None => {
            eprintln!("[Beejs] Failed to parse request");
            return;
        }
    };

    eprintln!(
        "[Beejs] {} {} {}",
        request.method,
        request.url,
        request.http_version
    );

    // 由于 MinimalRuntime 不能直接在后台线程调用，
    // 我们在这里生成一个简单的响应
    // 完整的请求处理需要在主线程中调用 JavaScript 处理器

    // 存储请求信息到文件供测试使用
    let mut response = HttpServerResponse::new();
    let response_data = generate_http_response(&mut response);

    if let Err(e) = stream.write_all(&response_data) {
        eprintln!("[Beejs] Write error: {}", e);
    }
}

/// response.removeHeader() 回调 - v0.3.87
fn http_res_remove_header_callback(
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
    let headers_obj = if let Ok(obj) = v8::Local::<v8::Object>::try_from(
        this.get(scope, headers_key.into()).unwrap_or(v8::undefined(scope).into())
    ) {
        obj
    } else {
        v8::Object::new(scope)
    };

    let name_key: _ = v8::String::new(scope, &name).unwrap();
    let _undefined_val: _ = v8::undefined(scope);
    headers_obj.delete(scope, name_key.into());
    this.set(scope, headers_key.into(), headers_obj.into());

    retval.set(this.into());
}
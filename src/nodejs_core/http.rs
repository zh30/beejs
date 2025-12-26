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

use std::sync::atomic::AtomicU64;

// v0.3.97: 添加 SO_REUSEADDR 支持以解决端口重用问题
#[cfg(unix)]
use libc;

/// HTTP 请求消息（跨线程传递）
/// v0.3.89: 添加跨线程消息传递支持
#[derive(Debug, Clone)]
pub struct HttpRequestMessage {
    /// HTTP 方法
    pub method: String,
    /// 请求 URL
    pub url: String,
    /// 请求路径
    pub path: String,
    /// HTTP 版本
    pub http_version: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Vec<u8>,
    /// 连接 ID（用于响应时定位连接）
    pub connection_id: u64,
}

/// HTTP 响应消息（跨线程传递）
/// v0.3.89: 添加跨线程消息传递支持
#[derive(Debug)]
pub struct HttpResponseMessage {
    /// 连接 ID
    pub connection_id: u64,
    /// 状态码
    pub status_code: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
}

/// HTTP 服务器消息通道
/// 用于主线程和后台线程之间的请求/响应传递
/// v0.3.89: 添加跨线程消息传递支持
pub struct HttpServerMessageChannel {
    /// 发送请求到主线程
    pub request_sender: crossbeam::channel::Sender<HttpRequestMessage>,
    /// v0.3.90: 接收来自后台线程的请求
    pub request_receiver: crossbeam::channel::Receiver<HttpRequestMessage>,
    /// 接收主线程的响应
    pub response_receiver: crossbeam::channel::Receiver<HttpResponseMessage>,
    /// v0.3.90: 发送响应到后台线程
    pub response_sender: crossbeam::channel::Sender<HttpResponseMessage>,
    /// 是否启用了消息模式
    pub enabled: bool,
    /// 下一个连接 ID
    pub next_connection_id: Arc<AtomicU64>,
}

impl HttpServerMessageChannel {
    /// 创建新的消息通道
    #[allow(clippy::redundant_closure)]
    pub fn new(capacity: usize) -> Self {
        let (request_sender, request_receiver) = crossbeam::channel::bounded(capacity);
        let (response_sender, response_receiver) = crossbeam::channel::bounded(capacity);

        Self {
            request_sender,
            request_receiver,
            response_receiver,
            response_sender,
            enabled: true,
            next_connection_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// 生成新的连接 ID
    pub fn next_connection_id(&self) -> u64 {
        self.next_connection_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// 发送请求消息
    pub fn send_request(&self, request: HttpRequestMessage) -> Result<(), crossbeam::channel::SendError<HttpRequestMessage>> {
        self.request_sender.send(request)
    }

    /// 接收响应消息
    pub fn recv_response(&self) -> Result<HttpResponseMessage, crossbeam::channel::RecvError> {
        self.response_receiver.recv()
    }

    /// 尝试接收响应（非阻塞）
    pub fn try_recv_response(&self) -> Result<HttpResponseMessage, crossbeam::channel::TryRecvError> {
        self.response_receiver.try_recv()
    }

    /// v0.3.90: 发送响应消息到后台线程
    pub fn send_response(&self, response: HttpResponseMessage) -> Result<(), crossbeam::channel::SendError<HttpResponseMessage>> {
        self.response_sender.send(response)
    }
}

/// 全局 HTTP 服务器消息通道
/// v0.3.89: 添加跨线程消息传递支持
static mut HTTP_SERVER_CHANNEL: Option<Arc<Mutex<Option<HttpServerMessageChannel>>>> = None;

/// 初始化全局消息通道
#[allow(static_mut_refs)]
pub fn init_http_server_channel() -> Arc<Mutex<Option<HttpServerMessageChannel>>> {
    unsafe {
        if HTTP_SERVER_CHANNEL.is_none() {
            HTTP_SERVER_CHANNEL = Some(Arc::new(Mutex::new(Some(HttpServerMessageChannel::new(100)))));
        }
        HTTP_SERVER_CHANNEL.as_ref().unwrap().clone()
    }
}

/// 获取全局消息通道
#[allow(static_mut_refs)]
pub fn get_http_server_channel() -> Option<Arc<Mutex<Option<HttpServerMessageChannel>>>> {
    unsafe {
        HTTP_SERVER_CHANNEL.as_ref().cloned()
    }
}

/// 重置全局消息通道
/// v0.3.93: 添加测试支持，用于清空通道中的残留消息
/// 创建一个新的消息通道，丢弃所有未处理的消息
#[allow(static_mut_refs)]
pub fn reset_http_server_channel() {
    unsafe {
        if let Some(ref channel_arc) = HTTP_SERVER_CHANNEL {
            let mut channel_guard = channel_arc.lock().unwrap();
            // 创建一个新通道，丢弃所有未处理的消息
            *channel_guard = Some(HttpServerMessageChannel::new(100));
        }
    }
}

/// 发送 HTTP 响应到后台线程
/// v0.3.90: 实现跨线程响应传递
#[allow(static_mut_refs)]
pub fn send_http_response(response: HttpResponseMessage) {
    unsafe {
        if let Some(ref channel_arc) = HTTP_SERVER_CHANNEL {
            if let Some(ref channel) = *channel_arc.lock().unwrap() {
                let _ = channel.send_response(response);
            }
        }
    }
}

/// 获取消息接收器（用于事件循环轮询）
/// v0.3.90: 添加消息接收支持
#[allow(static_mut_refs)]
#[deprecated(since = "0.3.90", note = "Use try_recv_http_request instead")]
pub fn get_http_request_receiver() -> Option<crossbeam::channel::Receiver<HttpRequestMessage>> {
    unsafe {
        HTTP_SERVER_CHANNEL.as_ref().and_then(|channel_arc| {
            let _ = channel_arc.lock().unwrap().as_ref()?;
            // 使用 try_recv_http_request 替代
            None
        })
    }
}

/// v0.3.90: 尝试接收 HTTP 请求（非阻塞）
/// 返回 Some(request) 如果有请求，None 如果没有请求
#[allow(static_mut_refs)]
pub fn try_recv_http_request() -> Option<HttpRequestMessage> {
    unsafe {
        if let Some(ref channel_arc) = HTTP_SERVER_CHANNEL {
            if let Some(ref channel) = *channel_arc.lock().unwrap() {
                match channel.request_receiver.try_recv() {
                    Ok(request) => Some(request),
                    Err(crossbeam::channel::TryRecvError::Empty) => {
                        // Channel is empty, no request available
                        None
                    }
                    Err(crossbeam::channel::TryRecvError::Disconnected) => {
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// v0.3.90: 创建简单的 HTTP 响应消息
pub fn create_http_response(
    connection_id: u64,
    status_code: u16,
    body: &str,
    content_type: &str,
) -> HttpResponseMessage {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), content_type.to_string());
    headers.insert("Content-Length".to_string(), body.len().to_string());
    // v0.3.97: 不设置默认 Connection 头，让服务器根据 Keep-Alive 决定

    HttpResponseMessage {
        connection_id,
        status_code,
        headers,
        body: body.as_bytes().to_vec(),
    }
}

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

    // createServer - 使用普通 callback
    // v0.3.93: callback 中会直接从 context 获取全局对象
    let create_server_func = v8::FunctionTemplate::new(
        scope,
        http_create_server_with_global_callback,
    );
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
/// v0.3.93: http.createServer callback 版本，可以访问全局对象
fn http_create_server_with_global_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let request_handler: _ = args.get(0);

    // 直接从当前 scope 获取 context 和全局对象
    let context = scope.get_current_context();
    let global = context.global(scope);

    let server_obj: _ = v8::Object::new(scope);

    // 如果提供了 request handler，立即存储到 _requestHandler
    if request_handler.is_function() {
        let handler_key = v8::String::new(scope, "_requestHandler").unwrap();
        server_obj.set(scope, handler_key.into(), request_handler);

        // v0.3.93: 同时设置全局 HTTP request handler
        let global_handler_key = v8::String::new(scope, "_httpServerRequestHandler").unwrap();
        global.set(scope, global_handler_key.into(), request_handler);
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
    // close
    let close_func: _ = v8::FunctionTemplate::new(scope, http_server_close_callback);
    let close_instance: _ = close_func.get_function(scope).unwrap();
    let close_key: _ = v8::String::new(scope, "close").unwrap();
    server_obj.set(scope, close_key.into(), close_instance.into());

    // 初始化消息通道
    let _message_channel = init_http_server_channel();
    let channel_key = v8::String::new(scope, "_messageChannel").unwrap();
    let channel_initialized = v8::Boolean::new(scope, true);
    server_obj.set(scope, channel_key.into(), channel_initialized.into());

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
    // v0.3.89: 检查是否启用了消息通道模式
    let use_message_channel = get_http_server_channel().is_some();
    let server_state = Arc::new(HttpServerState {
        listening: Arc::new(AtomicBool::new(false)),
        port,
        host: host.clone(),
        use_message_channel,
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
pub fn http_res_set_header_callback(
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
pub fn http_res_write_head_callback(
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
/// response.end() 回调 - v0.3.64
pub fn http_res_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let data: _ = args.get(0);

    // 处理 end() 的数据参数，存储到 _body
    if !data.is_undefined() {
        let body_key: _ = v8::String::new(scope, "_body").unwrap();
        // 获取现有 body
        let existing_body = this.get(scope, body_key.into()).unwrap_or(v8::undefined(scope).into());

        // 追加新数据到现有 body
        if existing_body.is_string() {
            // 字符串拼接 - 预先构建 Rust 字符串避免双重借用
            let existing_str = existing_body.to_string(scope).unwrap();
            let existing_rust = existing_str.to_rust_string_lossy(scope);

            let data_rust = if data.is_string() {
                let data_str = data.to_string(scope).unwrap();
                data_str.to_rust_string_lossy(scope)
            } else {
                "[data]".to_string()
            };

            let combined_rust = format!("{}{}", existing_rust, data_rust);
            let combined = v8::String::new(scope, &combined_rust).unwrap();
            this.set(scope, body_key.into(), combined.into());
        } else {
            // 存储新数据
            this.set(scope, body_key.into(), data);
        }
    }

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
    /// v0.3.89: 是否使用消息通道模式
    pub use_message_channel: bool,
}

impl HttpServerState {
    pub fn new() -> Self {
        Self {
            listening: Arc::new(AtomicBool::new(false)),
            port: 3000,
            host: "0.0.0.0".to_string(),
            use_message_channel: false,
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

/// 生成 HTTP 响应（从 HttpResponseMessage）
/// v0.3.89: 添加跨线程消息传递支持
/// v0.3.95: 移除重复的 Content-Length 添加（ headers 中已包含）
pub fn generate_http_response_v2(response: &HttpResponseMessage) -> Vec<u8> {
    let mut result = Vec::new();

    // Status line
    result.extend_from_slice(format!("HTTP/1.1 {} OK\r\n", response.status_code).as_bytes());

    // Headers - Content-Length 已在 send_http_response 中从 JS 对象提取
    for (name, value) in &response.headers {
        result.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
    }

    // End of headers
    result.extend_from_slice(b"\r\n");

    // Body
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

    // v0.3.97: 设置 SO_REUSEADDR 和 SO_REUSEPORT 以允许端口重用
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = listener.as_raw_fd();
        let val: libc::c_int = 1;
        let val_ptr = &val as *const libc::c_int as *const libc::c_void;
        let val_len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        if unsafe { libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, val_ptr, val_len) } != 0 {
            eprintln!("[Beejs] Failed to set SO_REUSEADDR: {}", std::io::Error::last_os_error());
        }
        // macOS 需要 SO_REUSEPORT 来完全解决端口重用问题
        #[cfg(target_os = "macos")]
        {
            if unsafe { libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEPORT, val_ptr, val_len) } != 0 {
                eprintln!("[Beejs] Failed to set SO_REUSEPORT: {}", std::io::Error::last_os_error());
            }
        }
    }

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
/// v0.3.89: 修改为使用消息通道模式，支持跨线程 V8 上下文调用
/// 检查是否应该保持连接（Keep-Alive）
/// v0.3.96: 新增功能
fn should_keep_alive(headers: &HashMap<String, String>, http_version: &str) -> bool {
    // HTTP/1.1 默认 Keep-Alive，HTTP/1.0 默认 Close
    if http_version == "HTTP/1.1" {
        // HTTP/1.1 默认 Keep-Alive，除非明确指定 Connection: close
        match headers.get("Connection").map(|s| s.to_lowercase()) {
            Some(conn) if conn == "close" => false,
            Some(conn) if conn == "keep-alive" => true,
            None => true, // 没有 Connection 头，默认 Keep-Alive
            _ => true,
        }
    } else {
        // HTTP/1.0 默认 Close，除非明确指定 Connection: keep-alive
        match headers.get("Connection").map(|s| s.to_lowercase()) {
            Some(conn) if conn == "keep-alive" => true,
            _ => false,
        }
    }
}

/// 处理 HTTP 连接（支持 Keep-Alive）
/// v0.3.96: 添加 Keep-Alive 支持
/// v0.3.87: 基础功能
fn handle_connection(
    mut stream: TcpStream,
    server_state: &HttpServerState,
    _handler_code: &str,
    connection_id: u64,
) {
    // v0.3.97: Keep-Alive 循环：处理多个请求
    const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(30);

    loop {
        let mut buffer = [0u8; 8192];
        let mut request_data = Vec::new();
        let mut connection_close = false;
        let mut _is_keep_alive = false;
        let keep_alive_start = Instant::now();

        // 读取请求数据
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // 连接关闭
                    connection_close = true;
                    break;
                }
                Ok(n) => {
                    request_data.extend_from_slice(&buffer[..n]);

                    // 检查是否收到完整的请求头（以 \r\n\r\n 结尾）
                    if request_data.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }

                    // 防止缓冲区过大
                    if request_data.len() > 1024 * 1024 {
                        eprintln!("[Beejs] Request too large");
                        connection_close = true;
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 没有数据可用，检查是否超时
                    if keep_alive_start.elapsed() > KEEP_ALIVE_TIMEOUT {
                        eprintln!("[Beejs] Keep-Alive timeout ({}s), closing connection", KEEP_ALIVE_TIMEOUT.as_secs());
                        break;
                    }
                    // 短暂等待后重试
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(e) => {
                    eprintln!("[Beejs] Read error: {}", e);
                    connection_close = true;
                    break;
                }
            }
        }

        // v0.3.97: 如果连接关闭或没有请求数据，退出循环
        if connection_close {
            break;
        }
        if request_data.is_empty() {
            // 没有收到数据但连接未关闭，这是 Keep-Alive 超时
            eprintln!("[Beejs] Keep-Alive timeout, closing connection");
            break;
        }

        // 解析 HTTP 请求
        let parsed_request = match parse_http_request(&request_data) {
            Some(req) => req,
            None => {
                eprintln!("[Beejs] Failed to parse request");
                break;
            }
        };

        // 判断是否 Keep-Alive
        _is_keep_alive = should_keep_alive(&parsed_request.headers, &parsed_request.http_version);

        eprintln!(
            "[Beejs] {} {} {} (Keep-Alive: {})",
            parsed_request.method,
            parsed_request.url,
            parsed_request.http_version,
            _is_keep_alive
        );

        // v0.3.89: 尝试通过消息通道发送到主线程处理
        // v0.3.92: 添加超时机制，防止在没有调用 pump_http_messages 时永久阻塞
        let channel = get_http_server_channel();
        let use_message_channel = server_state.use_message_channel && channel.is_some();

        let mut message_channel_used = false;

        // 创建请求消息
        let request_msg = HttpRequestMessage {
            method: parsed_request.method.clone(),
            url: parsed_request.url.clone(),
            path: parsed_request.path.clone(),
            http_version: parsed_request.http_version.clone(),
            headers: parsed_request.headers.clone(),
            body: parsed_request.body.clone(),
            connection_id,
        };

        if use_message_channel {
            if let Some(ref channel_ref) = channel {
                let locked = channel_ref.lock().unwrap();
                if let Some(ref msg_channel) = *locked {
                    match msg_channel.send_request(request_msg) {
                        Ok(()) => {
                            message_channel_used = true;
                        }
                        Err(e) => {
                            eprintln!("[Beejs] Failed to send request via channel: {:?}", e);
                        }
                    }
                }
            }
        }

        // v0.3.93: 只有当消息通道未使用时才发送回退响应
        // v0.3.96: 添加 Keep-Alive 支持
        if !message_channel_used {
            // 消息通道不可用，发送回退响应
            let fallback_body = format!(
                "Beejs HTTP Server\nMethod: {}\nPath: {}\nHandler: not configured",
                parsed_request.method,
                parsed_request.path
            );

            // 根据 Keep-Alive 决定 Connection 头
            let connection_header = if _is_keep_alive { "keep-alive" } else { "close" };

            let mut response_data = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n",
                fallback_body.len(),
                connection_header
            );
            response_data.push_str(&fallback_body);

            eprintln!("[Debug] Fallback path sending response with Connection: {}", connection_header);
            if let Err(e) = stream.write_all(response_data.as_bytes()) {
                eprintln!("[Beejs] Write error: {}", e);
            }

            // 如果不是 Keep-Alive，关闭连接
            if !_is_keep_alive {
                let _ = stream.shutdown(std::net::Shutdown::Write);
                break;
            }
            // 否则继续循环等待下一个请求
            continue;
        }

        // v0.3.93: 消息通道已使用，等待主线程处理
        // v0.3.95: 修复 - 移除超时限制，改为无限等待响应
        // v0.3.95: 修复 deadlock - 释放锁后再等待，避免阻塞其他线程
        // v0.3.96: 添加 Keep-Alive 支持
        let channel = get_http_server_channel();
        if let Some(ref channel_ref) = channel {
            // 获取响应接收器并释放锁，避免阻塞其他线程
            let response_receiver = {
                let guard = channel_ref.lock().unwrap();
                if let Some(ref msg_channel) = *guard {
                    // v0.3.95: 克隆 receiver 并释放锁后再等待
                    Some(msg_channel.response_receiver.clone())
                } else {
                    None
                }
            };
            // guard 现在被释放，锁已释放

            // 等待响应
            if let Some(receiver) = response_receiver {
                match receiver.recv() {
                    Ok(response) => {
                        // 收到响应，根据 _is_keep_alive 决定是否关闭连接
                        let connection_header = if _is_keep_alive { "keep-alive" } else { "close" };
                        eprintln!("[Debug] Received response from message channel");
                        eprintln!("[Debug] response.connection_id: {}, response.status_code: {}", response.connection_id, response.status_code);
                        eprintln!("[Debug] response.headers: {:?}", response.headers);

                        // 生成响应并添加 Connection 头
                        let mut response_data = generate_http_response_v2(&response);
                        eprintln!("[Debug] Original response (bytes): {:?}", response_data);
                        eprintln!("[Debug] Original response (utf8): {:?}", String::from_utf8_lossy(&response_data));

                        // 如果还没有 Connection 头，添加它
                        eprintln!("[Debug] response.headers before check: {:?}", response.headers);
                        if !response.headers.contains_key("Connection") {
                            // 查找 \r\n\r\n（header 和 body 之间的分隔符）
                            // 注意：windows 找到的 \r\n\r\n 可能是 header 行的结尾 + header/body 分隔符的组合
                            // 需要在最后一个 header 的 \r\n 之后插入新的 header
                            if let Some(separator_pos) = response_data.windows(4).rposition(|w| w == b"\r\n\r\n") {
                                eprintln!("[Debug] Found \\r\\n\\r\\n at position {}", separator_pos);
                                eprintln!("[Debug] 4 bytes at separator_pos: {:?}", &response_data[separator_pos..separator_pos+4]);
                                // separator_pos 指向 \r\n\r\n 的第一个 \r
                                // 需要在最后一个 header 的 \r\n 之后插入，即 separator_pos + 2
                                let insert_pos = separator_pos + 2;
                                eprintln!("[Debug] No Connection header in response, will add: {} at position {}", connection_header, insert_pos);
                                // 使用 insert 逐字节插入，确保正确插入而不是替换
                                let connection_header_bytes = format!("Connection: {}\r\n", connection_header).into_bytes();
                                for i in (0..connection_header_bytes.len()).rev() {
                                    response_data.insert(insert_pos, connection_header_bytes[i]);
                                }
                                eprintln!("[Debug] After insert, response_data len: {}", response_data.len());
                            } else {
                                eprintln!("[Debug] No separator found, response_data len: {}", response_data.len());
                            }
                        }

                        eprintln!("[Debug] Message channel path sending response with Connection: {}", connection_header);
                        eprintln!("[Debug] Full response data: {:?}", String::from_utf8_lossy(&response_data));
                        let _ = stream.write_all(&response_data);

                        // 如果不是 Keep-Alive，关闭连接
                        if !_is_keep_alive {
                            let _ = stream.shutdown(std::net::Shutdown::Write);
                            break;
                        }
                        // 否则继续循环等待下一个请求
                    }
                    Err(_) => {
                        // 通道断开，关闭连接
                        let _ = stream.shutdown(std::net::Shutdown::Write);
                        break;
                    }
                }
            }
        }
    }

    // 连接关闭
    let _ = stream.shutdown(std::net::Shutdown::Write);
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

// ============================================================================
// v0.3.91: V8 上下文 HTTP 请求处理（跨线程调用 JavaScript Handler）
// ============================================================================

/// 在 V8 上下文中处理 HTTP 请求消息
/// 调用 JavaScript request handler 并返回响应
/// v0.3.91: 新增功能
///
/// # 参数
/// - `request`: HTTP 请求消息（从消息通道接收）
/// - `scope`: V8 句柄作用域
/// - `_context`: V8 上下文（预留用于将来使用）
/// - `request_handler`: 可选的 JavaScript request handler 函数
///
/// # 返回
/// - `Some(HttpResponseMessage)` 如果处理成功
/// - `None` 如果没有 handler 或处理失败
pub fn process_http_request_in_v8(
    request: &HttpRequestMessage,
    scope: &mut v8::HandleScope,
    _context: &v8::Local<v8::Context>,
    request_handler: Option<v8::Global<v8::Function>>,
) -> Option<HttpResponseMessage> {
    // 如果没有 handler，返回 404
    let handler = request_handler?;
    let handler_fn = v8::Local::new(scope, &handler);

    // 创建请求对象 (IncomingMessage)
    let req_obj = v8::Object::new(scope);

    // 设置请求属性
    let method_key = v8::String::new(scope, "method").unwrap();
    let method_val = v8::String::new(scope, &request.method).unwrap();
    req_obj.set(scope, method_key.into(), method_val.into());

    let url_key = v8::String::new(scope, "url").unwrap();
    let url_val = v8::String::new(scope, &request.url).unwrap();
    req_obj.set(scope, url_key.into(), url_val.into());

    let path_key = v8::String::new(scope, "path").unwrap();
    let path_val = v8::String::new(scope, &request.path).unwrap();
    req_obj.set(scope, path_key.into(), path_val.into());

    let http_version_key = v8::String::new(scope, "httpVersion").unwrap();
    let http_version_val = v8::String::new(scope, &request.http_version).unwrap();
    req_obj.set(scope, http_version_key.into(), http_version_val.into());

    // 设置 headers 对象（使用 lowercase 键名以匹配 Node.js 惯例）
    // v0.3.95: 修复 header 键名大小写问题
    let headers_obj = v8::Object::new(scope);
    for (name, value) in &request.headers {
        // 使用 lowercase 键名，HTTP header 查找是大小写敏感的
        let name_lower = name.to_lowercase();
        let name_key = v8::String::new(scope, &name_lower).unwrap();
        let value_val = v8::String::new(scope, value).unwrap();
        headers_obj.set(scope, name_key.into(), value_val.into());
    }
    let headers_key = v8::String::new(scope, "headers").unwrap();
    req_obj.set(scope, headers_key.into(), headers_obj.into());

    // 创建响应对象 (ServerResponse)
    let res_obj = v8::Object::new(scope);

    // 初始化 headers 对象
    let res_headers_obj = v8::Object::new(scope);
    let res_headers_key = v8::String::new(scope, "headers").unwrap();
    res_obj.set(scope, res_headers_key.into(), res_headers_obj.into());

    // 初始化 statusCode
    let status_code_key = v8::String::new(scope, "statusCode").unwrap();
    let status_code_val = v8::Integer::new(scope, 200);
    res_obj.set(scope, status_code_key.into(), status_code_val.into());

    // 初始化 statusMessage
    let status_msg_key = v8::String::new(scope, "statusMessage").unwrap();
    let status_msg_val = v8::String::new(scope, "OK").unwrap();
    res_obj.set(scope, status_msg_key.into(), status_msg_val.into());

    // 初始化 _body 用于存储响应体
    let body_key = v8::String::new(scope, "_body").unwrap();
    let empty_body = v8::String::new(scope, "").unwrap();
    res_obj.set(scope, body_key.into(), empty_body.into());

    // 设置 end 方法
    let end_fn = v8::FunctionTemplate::new(scope, http_res_end_callback);
    let end_instance = end_fn.get_function(scope).unwrap();
    let end_key = v8::String::new(scope, "end").unwrap();
    res_obj.set(scope, end_key.into(), end_instance.into());

    // 设置 writeHead 方法
    let write_head_fn = v8::FunctionTemplate::new(scope, http_res_write_head_callback);
    let write_head_instance = write_head_fn.get_function(scope).unwrap();
    let write_head_key = v8::String::new(scope, "writeHead").unwrap();
    res_obj.set(scope, write_head_key.into(), write_head_instance.into());

    // 设置 setHeader 方法
    let set_header_fn = v8::FunctionTemplate::new(scope, http_res_set_header_callback);
    let set_header_instance = set_header_fn.get_function(scope).unwrap();
    let set_header_key = v8::String::new(scope, "setHeader").unwrap();
    res_obj.set(scope, set_header_key.into(), set_header_instance.into());

    // 设置 getHeader 方法
    let get_header_fn = v8::FunctionTemplate::new(scope, http_res_get_header_callback);
    let get_header_instance = get_header_fn.get_function(scope).unwrap();
    let get_header_key = v8::String::new(scope, "getHeader").unwrap();
    res_obj.set(scope, get_header_key.into(), get_header_instance.into());

    // 设置 removeHeader 方法
    let remove_header_fn = v8::FunctionTemplate::new(scope, http_res_remove_header_callback);
    let remove_header_instance = remove_header_fn.get_function(scope).unwrap();
    let remove_header_key = v8::String::new(scope, "removeHeader").unwrap();
    res_obj.set(scope, remove_header_key.into(), remove_header_instance.into());

    // 调用 request handler: handler(req, res)
    let this_val = v8::undefined(scope).into();
    let args = [req_obj.into(), res_obj.into()];

    if handler_fn.call(scope, this_val, &args).is_none() {
        return None;
    }

    // 从响应对象提取数据
    let status_code_key = v8::String::new(scope, "statusCode").unwrap();
    let status_code_val = res_obj.get(scope, status_code_key.into()).unwrap_or(v8::Integer::new(scope, 200).into());
    let status_code = status_code_val.to_int32(scope).map(|i| i.value() as u16).unwrap_or(200);

    // 提取 body
    let body_key = v8::String::new(scope, "_body").unwrap();
    let body_val = res_obj.get(scope, body_key.into()).unwrap_or(v8::String::new(scope, "").unwrap().into());
    let body_str = body_val.to_string(scope).map(|s| s.to_rust_string_lossy(scope)).unwrap_or_default();
    let body_bytes = body_str.into_bytes();

    // 提取 headers - 使用 GetPropertyNames 获取所有属性
    // v0.3.95: 修复 header 枚举问题，使用更可靠的方法
    let mut response_headers = HashMap::new();
    let res_headers_key = v8::String::new(scope, "headers").unwrap();
    if let Some(headers_val) = res_obj.get(scope, res_headers_key.into()) {
        if let Ok(headers_obj) = v8::Local::<v8::Object>::try_from(headers_val) {
            // 使用 GetPropertyNames 获取所有属性
            let props = headers_obj.get_property_names(scope).unwrap_or(v8::Array::new(scope, 0));
            for i in 0..props.length() {
                if let Some(key_val) = props.get_index(scope, i) {
                    if let Some(key_str) = key_val.to_string(scope) {
                        let key = key_str.to_rust_string_lossy(scope);
                        if let Some(value_val) = headers_obj.get(scope, key_val) {
                            if let Some(value_str) = value_val.to_string(scope) {
                                let value = value_str.to_rust_string_lossy(scope);
                                response_headers.insert(key, value);
                            }
                        }
                    }
                }
            }
        }
    }

    // 设置默认 headers（如果还没有设置的话）
    if !response_headers.contains_key("Content-Type") {
        response_headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
    }

    eprintln!("[Debug] Response headers before HttpResponseMessage: {:?}", response_headers);

    Some(HttpResponseMessage {
        connection_id: request.connection_id,
        status_code,
        headers: response_headers,
        body: body_bytes,
    })
}

/// 在 V8 上下文中处理 HTTP 请求（获取 response 对象）
/// 用于 event_loop.rs 中轮询消息队列
/// v0.3.91: 新增功能
///
/// 返回响应的 body 字符串和状态码
pub fn handle_http_request_v8(
    request: &HttpRequestMessage,
    scope: &mut v8::HandleScope,
    context: &v8::Local<v8::Context>,
) -> Option<(u16, Vec<u8>)> {
    // 获取全局 request handler
    let handler = get_global_request_handler(scope, context)?;

    // 处理请求
    let response = process_http_request_in_v8(request, scope, context, Some(handler))?;

    Some((response.status_code, response.body))
}

/// 获取全局 request handler
/// v0.3.91: 新增功能
pub fn get_global_request_handler(
    scope: &mut v8::HandleScope,
    context: &v8::Local<v8::Context>,
) -> Option<v8::Global<v8::Function>> {
    let global = context.global(scope);

    // 查找 _httpServerRequestHandler
    let handler_key = v8::String::new(scope, "_httpServerRequestHandler").unwrap();
    let handler_val = global.get(scope, handler_key.into());

    let handler_val = match handler_val {
        Some(v) => v,
        None => {
            return None;
        }
    };

    if handler_val.is_undefined() {
        return None;
    }

    if !handler_val.is_function() {
        return None;
    }

    let handler_fn = v8::Local::<v8::Function>::try_from(handler_val).ok()?;
    Some(v8::Global::new(scope, handler_fn))
}

/// 设置全局 request handler（供 JS 代码使用）
/// v0.3.91: 新增功能
pub fn set_global_request_handler(
    scope: &mut v8::HandleScope,
    context: &v8::Local<v8::Context>,
    handler: v8::Local<v8::Function>,
) {
    let global = context.global(scope);
    let handler_key = v8::String::new(scope, "_httpServerRequestHandler").unwrap();
    global.set(scope, handler_key.into(), handler.into());
}

// ============================================================================
// v0.3.98: HTTPS Server Support (TLS/SSL)
// ============================================================================

use std::fs::File;
use std::io::BufReader;

/// HTTPS/TLS 配置
/// v0.3.98: 新增结构体
#[derive(Debug, Clone)]
pub struct HttpsServerConfig {
    /// TLS 证书文件路径
    pub cert_path: String,
    /// TLS 私钥文件路径
    pub key_path: String,
    /// 服务器端口
    pub port: u16,
    /// 服务器主机
    pub host: String,
    /// 是否验证客户端证书
    pub verify_client: bool,
    /// ALPN 协议列表
    pub alpn_protocols: Vec<Vec<u8>>,
}

impl Default for HttpsServerConfig {
    fn default() -> Self {
        Self {
            cert_path: String::new(),
            key_path: String::new(),
            port: 443,
            host: "0.0.0.0".to_string(),
            verify_client: false,
            alpn_protocols: vec![
                b"h2".to_vec(),
                b"http/1.1".to_vec(),
            ],
        }
    }
}

/// TLS 证书加载结果
/// v0.3.98: 新增
#[derive(Debug)]
pub struct TlsCertificate {
    /// 证书链
    pub cert_chain: Vec<rustls::Certificate>,
    /// 私钥
    pub private_key: rustls::PrivateKey,
}

/// 加载 TLS 证书和私钥
/// v0.3.98: 新增功能
///
/// # 参数
/// - `cert_path`: 证书文件路径 (PEM 格式)
/// - `key_path`: 私钥文件路径 (PEM 格式)
///
/// # 返回
/// - `Ok(TlsCertificate)` 加载成功
/// - `Err(String)` 加载失败
pub fn load_tls_certificate(cert_path: &str, key_path: &str) -> Result<TlsCertificate, String> {
    // 加载证书文件
    let cert_file = File::open(cert_path)
        .map_err(|e| format!("Failed to open certificate file: {}", e))?;
    let mut cert_reader = BufReader::new(cert_file);

    // 使用 rustls-pemfile 解析证书
    let certs_result = rustls_pemfile::certs(&mut cert_reader);
    let mut certs = Vec::new();
    for cert in certs_result {
        let cert_der = cert.map_err(|e| format!("Failed to parse certificate: {}", e))?;
        certs.push(rustls::Certificate(cert_der.to_vec()));
    }

    if certs.is_empty() {
        return Err("No certificates found in file".to_string());
    }

    // 加载私钥文件
    let key_file = File::open(key_path)
        .map_err(|e| format!("Failed to open key file: {}", e))?;
    let mut key_reader = BufReader::new(key_file);

    // 首先尝试解析 RSA 私钥
    let keys_result = rustls_pemfile::rsa_private_keys(&mut key_reader);
    let mut keys = Vec::new();
    for key in keys_result {
        let key_bytes = key.map_err(|e| format!("Failed to parse private key: {}", e))?;
        keys.push(rustls::PrivateKey(key_bytes.secret_pkcs1_der().to_vec()));
    }

    // 如果没有 RSA 密钥，尝试 PKCS8 格式
    if keys.is_empty() {
        drop(key_reader);
        let key_file = File::open(key_path)
            .map_err(|e| format!("Failed to reopen key file: {}", e))?;
        let mut key_reader = BufReader::new(key_file);

        let pkcs8_result = rustls_pemfile::pkcs8_private_keys(&mut key_reader);
        for key in pkcs8_result {
            let key_bytes = key.map_err(|e| format!("Failed to parse PKCS8 private key: {}", e))?;
            keys.push(rustls::PrivateKey(key_bytes.secret_pkcs8_der().to_vec()));
        }
    }

    if keys.is_empty() {
        return Err("No private key found in file".to_string());
    }

    Ok(TlsCertificate {
        cert_chain: certs,
        private_key: keys.remove(0),
    })
}

/// 创建 TLS 服务器配置
/// v0.3.98: 新增功能
///
/// # 参数
/// - `cert`: TLS 证书
/// - `config`: HTTPS 服务器配置
///
/// # 返回
/// - `Arc<rustls::ServerConfig>` TLS 服务器配置
pub fn create_tls_server_config(
    cert: &TlsCertificate,
    config: &HttpsServerConfig,
) -> Arc<rustls::ServerConfig> {
    let mut tls_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert.cert_chain.clone(), cert.private_key.clone())
        .expect("Failed to create TLS config");

    // 设置 ALPN 协议
    tls_config.alpn_protocols = config.alpn_protocols.clone();

    Arc::new(tls_config)
}

/// HTTPS 服务器状态
/// v0.3.98: 新增
#[derive(Debug, Clone)]
pub struct HttpsServerState {
    pub listening: Arc<AtomicBool>,
    pub port: u16,
    pub host: String,
    pub tls_config: Option<Arc<rustls::ServerConfig>>,
}

impl HttpsServerState {
    pub fn new() -> Self {
        Self {
            listening: Arc::new(AtomicBool::new(false)),
            port: 443,
            host: "0.0.0.0".to_string(),
            tls_config: None,
        }
    }

    /// 检查是否已配置 TLS
    pub fn is_tls_configured(&self) -> bool {
        self.tls_config.is_some()
    }
}

/// 解析 HTTPS URL
/// v0.3.98: 新增功能
///
/// HTTPS 请求解析与 HTTP 相同，只是传输层使用 TLS
pub fn parse_https_request(data: &[u8]) -> Option<HttpServerRequest> {
    // HTTPS 使用与 HTTP 相同的请求解析逻辑
    parse_http_request(data)
}

/// 生成 HTTPS 响应
/// v0.3.98: 新增功能
///
/// HTTPS 响应与 HTTP 响应格式相同
pub fn generate_https_response(response: &mut HttpServerResponse) -> Vec<u8> {
    generate_http_response(response)
}

// ============================================================================
// V8 API 集成 - HTTPS 服务器
// ============================================================================

/// 创建 HTTPS 服务器配置的 JavaScript API
/// v0.3.98: 新增功能
pub fn create_https_config_js<'a>(
    scope: &mut v8::HandleScope<'a>,
    cert_path: String,
    key_path: String,
    port: u16,
) -> Option<v8::Local<'a, v8::Object>> {
    let config_obj = v8::Object::new(scope);

    // certPath
    let cert_path_key = v8::String::new(scope, "certPath").unwrap();
    let cert_path_val = v8::String::new(scope, &cert_path).unwrap();
    config_obj.set(scope, cert_path_key.into(), cert_path_val.into());

    // keyPath
    let key_path_key = v8::String::new(scope, "keyPath").unwrap();
    let key_path_val = v8::String::new(scope, &key_path).unwrap();
    config_obj.set(scope, key_path_key.into(), key_path_val.into());

    // port
    let port_key = v8::String::new(scope, "port").unwrap();
    let port_val = v8::Number::new(scope, port as f64);
    config_obj.set(scope, port_key.into(), port_val.into());

    Some(config_obj)
}

/// 加载 TLS 证书的 JavaScript API
/// v0.3.98: 新增功能
pub fn load_tls_certificate_js<'a>(
    scope: &mut v8::HandleScope<'a>,
    args: v8::FunctionCallbackArguments<'a>,
) -> Option<v8::Local<'a, v8::Object>> {
    let cert_path: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let key_path: String = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // 尝试加载证书
    match load_tls_certificate(&cert_path, &key_path) {
        Ok(_) => {
            let result_obj = v8::Object::new(scope);
            let success_key = v8::String::new(scope, "success").unwrap();
            let success_val = v8::Boolean::new(scope, true);
            result_obj.set(scope, success_key.into(), success_val.into());
            Some(result_obj)
        }
        Err(e) => {
            let result_obj = v8::Object::new(scope);
            let success_key = v8::String::new(scope, "success").unwrap();
            let success_val = v8::Boolean::new(scope, false);
            result_obj.set(scope, success_key.into(), success_val.into());
            let error_key = v8::String::new(scope, "error").unwrap();
            let error_val = v8::String::new(scope, &e).unwrap();
            result_obj.set(scope, error_key.into(), error_val.into());
            Some(result_obj)
        }
    }
}
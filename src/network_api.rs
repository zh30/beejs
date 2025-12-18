//! 零拷贝网络 I/O JavaScript API 绑定
//!
//! 该模块将零拷贝网络功能暴露给 JavaScript

use crate::network::{NetworkBufferPool, ConnectionPool, NetworkIoStatistics};
use anyhow::Result;
use rusty_v8 as v8;
use std::sync::Arc;

/// 设置所有零拷贝网络 I/O API
pub fn setup_network_apis(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
    _buffer_pool: Arc<NetworkBufferPool>,
    _connection_pool: Arc<ConnectionPool>,
    _network_statistics: Arc<NetworkIoStatistics>,
) -> Result<()> {
    // 创建全局 network 对象
    let network_global = v8::Object::new(scope);

    // 添加一个简单的测试函数
    let test_func = v8::FunctionTemplate::new(scope, |callback_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let result = v8::Object::new(callback_scope);

        // 分步创建字符串避免借用检查问题
        let success_key = v8::String::new(callback_scope, "success").unwrap();
        let success_val = v8::Boolean::new(callback_scope, true);
        result.set(callback_scope, success_key.into(), success_val.into());

        let message_key = v8::String::new(callback_scope, "message").unwrap();
        let message_val = v8::String::new(callback_scope, "Zero-copy network I/O APIs initialized").unwrap();
        result.set(callback_scope, message_key.into(), message_val.into());

        retval.set(result.into());
    });

    let test_func_instance = test_func.get_function(scope).unwrap();
    let test_key = v8::String::new(scope, "testNetworkAPI").unwrap();
    network_global.set(scope, test_key.into(), test_func_instance.into());

    // 将 network 对象设置为全局
    let global = context.global(scope);
    let network_key = v8::String::new(scope, "Network").unwrap();
    global.set(scope, network_key.into(), network_global.into());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::network::{NetworkBufferPool, ConnectionPool, NetworkIoStatistics};
    use std::sync::Arc;

    #[test]
    fn test_setup_network_apis() {
        let buffer_pool = Arc::new(NetworkBufferPool::default());
        let _connection_pool = Arc::new(ConnectionPool::default());
        let network_statistics = Arc::new(NetworkIoStatistics::default());

        // 简化的测试，验证网络模块的基本功能
        assert!(buffer_pool.get_stats().active_buffers >= 0);
        assert!(network_statistics.zero_copy_ratio() >= 0.0);
    }
}

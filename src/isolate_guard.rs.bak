//! Isolate 生命周期管理守卫
//! 使用 RAII 模式确保 V8 Isolate 的正确清理
use rusty_v8 as v8;
use std::collections::{HashMap, BTreeMap};
/// Isolate 生命周期守卫
/// 确保 Isolate 在正确的作用域结束时被清理
#[allow(dead_code)]
pub struct IsolateGuard {
    isolate: Option<v8::OwnedIsolate>,
    return_to_pool: bool,
}
#[allow(dead_code)]
impl IsolateGuard {
    /// 创建新的 Isolate 守卫
    pub fn new(isolate: v8::OwnedIsolate, return_to_pool: bool) -> Self {
        Self {
            isolate: Some(isolate),
            return_to_pool,
        }
    }
    /// 获取内部的 Isolate 引用
    pub fn get(&self) -> &v8::OwnedIsolate {
        self.isolate.as_ref().expect("Isolate already released")
    }
    /// 获取内部的可变 Isolate 引用
    pub fn get_mut(&mut self) -> &mut v8::OwnedIsolate {
        self.isolate.as_mut().expect("Isolate already released")
    }
}
impl Drop for IsolateGuard {
    fn drop(&mut self) {
        if let Some(mut isolate) = self.isolate.take() {
            // 在清理前执行状态重置
            isolate.low_memory_notification();
            // 如果需要归还给池，则执行归还操作
            if self.return_to_pool {
                #[cfg(not(test))]
                {
                    crate::isolate_pool::release_isolate(isolate);
                }
                #[cfg(test)]
                {
                    // 在测试环境中直接丢弃
                    drop(isolate);
                }
            } else {
                // 直接丢弃
                drop(isolate);
            }
        }
    }
}
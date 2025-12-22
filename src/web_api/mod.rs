// Web API modules - Stage 43.0 / Stage 74.0
/// Web标准API实现
use std::task::Context;
pub mod fetch;
pub mod websocket;
pub mod crypto;
pub mod url;
pub mod events;
pub mod form_data;
pub mod abort;
pub mod blob;        // Stage 74: Blob/File API
pub mod timers;      // Stage 74: Timer APIs
pub mod encoding;    // Stage 74: TextEncoder/TextDecoder
pub mod performance; // Stage 74: Performance API
use anyhow::Result;
use rusty_v8 as v8;
// 从各模块导入设置函数
use abort::setup_abort_api;
use blob::setup_blob_api;
use crypto::setup_crypto_api;
use encoding::setup_encoding_api;
use events::setup_events_api;
use fetch::setup_fetch_api;
use form_data::setup_form_data_api;
use performance::setup_performance_api;
use timers::setup_timer_api;
use url::setup_url_api;
use websocket::setup_websocket_api;
use std::collections::{HashMap, BTreeMap};
use std::fs::File;
/// 初始化所有 Web API 到 V8 上下文
pub fn init_web_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // 按照依赖顺序初始化各个 API
    // 1. 基础 API（无依赖）
    eprintln!("🔧 [STAGE74] Setting up crypto API...");
    setup_crypto_api(scope, context)?;
    eprintln!("✅ [STAGE74] crypto API done");
    eprintln!("🔧 [STAGE74] Setting up events API...");
    setup_events_api(scope, context)?;
    eprintln!("✅ [STAGE74] events API done");
    eprintln!("🔧 [STAGE74] Setting up abort API...");
    setup_abort_api(scope, context)?;
    eprintln!("✅ [STAGE74] abort API done");
    eprintln!("🔧 [STAGE74] Setting up blob API...");
    setup_blob_api(scope, context)?;     // Stage 74: Blob/File API
    eprintln!("✅ [STAGE74] blob API done");
    eprintln!("🔧 [STAGE74] Setting up timer API...");
    setup_timer_api(scope, context)?;
    eprintln!("✅ [STAGE74] timer API done");
    eprintln!("🔧 [STAGE74] Setting up encoding API...");
    setup_encoding_api(scope, context)?;     // Stage 74: Encoding APIs
    eprintln!("✅ [STAGE74] encoding API done");
    eprintln!("🔧 [STAGE74] Setting up performance API...");
    setup_performance_api(scope, context)?;  // Stage 74: Performance API
    eprintln!("✅ [STAGE74] performance API done");
    // 2. URL API（依赖 events）
    eprintln!("🔧 [STAGE74] Setting up URL API...");
    setup_url_api(scope, context)?;
    eprintln!("✅ [STAGE74] URL API done");
    // 3. FormData API（依赖 URL）
    eprintln!("🔧 [STAGE74] Setting up FormData API...");
    setup_form_data_api(scope, context)?;
    eprintln!("✅ [STAGE74] FormData API done");
    // 4. Fetch API（依赖 URL, Headers, Request, Response, Abort）
    eprintln!("🔧 [STAGE74] Setting up fetch API...");
    setup_fetch_api(scope, context)?;
    eprintln!("✅ [STAGE74] fetch API done");
    // 5. WebSocket API（依赖 Events）
    eprintln!("🔧 [STAGE74] Setting up WebSocket API...");
    setup_websocket_api(scope, context)?;
    eprintln!("✅ [STAGE74] WebSocket API done");
    eprintln!("🎉 [STAGE74] All Web APIs initialized!");
    Ok(())
}
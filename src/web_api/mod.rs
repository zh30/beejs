//! Web API modules - Stage 43.0 / Stage 74.0
//! Web标准API实现

pub mod fetch;
pub mod websocket;
pub mod crypto;
pub mod url;
pub mod events;
pub mod form_data;
pub mod abort;
pub mod timers;      // Stage 74: Timer APIs
pub mod encoding;    // Stage 74: TextEncoder/TextDecoder
pub mod performance; // Stage 74: Performance API

use anyhow::Result;
use rusty_v8 as v8;

// 从各模块导入设置函数
use abort::setup_abort_api;
use crypto::setup_crypto_api;
use encoding::setup_encoding_api;
use events::setup_events_api;
use fetch::setup_fetch_api;
use form_data::setup_form_data_api;
use performance::setup_performance_api;
use timers::setup_timer_api;
use url::setup_url_api;
use websocket::setup_websocket_api;

/// 初始化所有 Web API 到 V8 上下文
pub fn init_web_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // 按照依赖顺序初始化各个 API

    // 1. 基础 API（无依赖）
    setup_crypto_api(scope, context)?;
    setup_events_api(scope, context)?;
    setup_abort_api(scope, context)?;
    setup_timer_api(scope, context)?;
    setup_encoding_api(scope, context)?;     // Stage 74: Encoding APIs
    setup_performance_api(scope, context)?;  // Stage 74: Performance API

    // 2. URL API（依赖 events）
    setup_url_api(scope, context)?;

    // 3. FormData API（依赖 URL）
    setup_form_data_api(scope, context)?;

    // 4. Fetch API（依赖 URL, Headers, Request, Response, Abort）
    setup_fetch_api(scope, context)?;

    // 5. WebSocket API（依赖 Events）
    setup_websocket_api(scope, context)?;

    Ok(())
}

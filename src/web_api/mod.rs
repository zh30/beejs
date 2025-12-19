//! Web API modules - Stage 43.0
//! Web标准API实现

pub mod fetch;
pub mod websocket;
pub mod crypto;
pub mod url;
pub mod events;
pub mod form_data;
pub mod abort;

use anyhow::Result;
use rusty_v8 as v8;

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

// 重新导出各个模块的设置函数
pub use fetch::setup_fetch_api;
pub use websocket::setup_websocket_api;
pub use crypto::setup_crypto_api;
pub use url::setup_url_api;
pub use events::setup_events_api;
pub use form_data::setup_form_data_api;
pub use abort::setup_abort_api;

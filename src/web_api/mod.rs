// Web API modules - Stage 43.0 / Stage 74.0
/// Web标准API实现
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
pub mod streams;     // Stage 75: Web Streams API for AI workloads
pub mod compression; // v0.3.295: CompressionStream API (gzip/deflate)
pub mod structured_clone; // v0.3.299: structuredClone global function
pub mod array_buffer_transfer; // v0.3.311: ArrayBuffer transfer (zero-copy detach)
pub mod broadcast_channel; // v0.3.312: BroadcastChannel API (cross-tab communication)
pub mod message_channel; // v0.3.315: MessageChannel API (port-based communication)
pub mod worker; // v0.3.320: Worker API (Web Worker support for parallel execution)
pub mod shared_array_buffer; // v0.3.322: SharedArrayBuffer API (cross-Worker shared memory)
pub mod service_worker; // v0.3.324: ServiceWorker API (background tasks, push, offline)
pub mod background_sync; // v0.3.327: Background Sync API (SyncManager, SyncEvent)
pub mod notification; // v0.3.328: Notification API (system notifications)
pub mod payment_request; // v0.3.328: Payment Request API (payment processing)
pub mod error_event; // v0.3.333: ErrorEvent API (script error handling)
pub mod custom_event; // v0.3.337: CustomEvent API (custom event handling)
pub mod dom_parser; // v0.3.341: DOMParser API (HTML/XML document parsing for AI workloads)
use anyhow::Result;
use rusty_v8 as v8;
// 从各模块导入设置函数
use abort::setup_abort_api;
use background_sync::setup_background_sync_api;
use blob::setup_blob_api;
use broadcast_channel::setup_broadcast_channel_api;
use compression::setup_compression_api;
use crypto::setup_crypto_api;
use custom_event::setup_custom_event_api;
use dom_parser::setup_dom_parser_api;
use encoding::setup_encoding_api;
use events::setup_events_api;
use fetch::setup_fetch_api;
use form_data::setup_form_data_api;
use message_channel::setup_message_channel_api;
use notification::setup_notification_api;
use payment_request::setup_payment_request_api;
use error_event::setup_error_event_api;
use shared_array_buffer::setup_shared_array_buffer_api;
use service_worker::setup_service_worker_api;
use worker::setup_worker_api;
use performance::setup_performance_api;
use structured_clone::setup_structured_clone_api;
use array_buffer_transfer::setup_array_buffer_transfer_api;
use timers::setup_timer_api;
use url::setup_url_api;
use websocket::setup_websocket_api;
/// 初始化所有 Web API 到 V8 上下文
pub fn init_web_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);
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
    // v0.3.295: CompressionStream API（依赖 Web Streams）
    eprintln!("🔧 [v0.3.295] Setting up CompressionStream API...");
    setup_compression_api(scope, context)?;
    eprintln!("✅ [v0.3.295] CompressionStream API done");
    // v0.3.299: structuredClone global function
    eprintln!("🔧 [v0.3.299] Setting up structuredClone API...");
    setup_structured_clone_api(scope, context)?;
    eprintln!("✅ [v0.3.299] structuredClone API done");
    // v0.3.311: ArrayBuffer transfer API (zero-copy detach for AI workloads)
    eprintln!("🔧 [v0.3.311] Setting up ArrayBuffer transfer API...");
    setup_array_buffer_transfer_api(scope, context)?;
    eprintln!("✅ [v0.3.311] ArrayBuffer transfer API done");
    // v0.3.312: BroadcastChannel API (cross-tab communication)
    eprintln!("🔧 [v0.3.312] Setting up BroadcastChannel API...");
    setup_broadcast_channel_api(scope, context)?;
    eprintln!("✅ [v0.3.312] BroadcastChannel API done");
    // v0.3.315: MessageChannel API (port-based communication)
    eprintln!("🔧 [v0.3.315] Setting up MessageChannel API...");
    setup_message_channel_api(scope, context)?;
    eprintln!("✅ [v0.3.315] MessageChannel API done");
    // v0.3.320: Worker API (Web Worker support for parallel execution)
    eprintln!("🔧 [v0.3.320] Setting up Worker API...");
    setup_worker_api(scope, context)?;
    eprintln!("✅ [v0.3.320] Worker API done");
    // v0.3.322: SharedArrayBuffer API (cross-Worker shared memory)
    eprintln!("🔧 [v0.3.322] Setting up SharedArrayBuffer API...");
    setup_shared_array_buffer_api(scope, context)?;
    eprintln!("✅ [v0.3.322] SharedArrayBuffer API done");
    // v0.3.324: ServiceWorker API (background tasks, push notifications, offline caching)
    eprintln!("🔧 [v0.3.324] Setting up ServiceWorker API...");
    setup_service_worker_api(scope, context)?;
    eprintln!("✅ [v0.3.324] ServiceWorker API done");
    // v0.3.327: Background Sync API (SyncManager, SyncEvent)
    eprintln!("🔧 [v0.3.327] Setting up Background Sync API...");
    setup_background_sync_api(scope, context)?;
    eprintln!("✅ [v0.3.327] Background Sync API done");
    // v0.3.328: Notification API (system notifications)
    eprintln!("🔧 [v0.3.328] Setting up Notification API...");
    setup_notification_api(scope, context, global)?;
    eprintln!("✅ [v0.3.328] Notification API done");
    // v0.3.328: Payment Request API (payment processing)
    eprintln!("🔧 [v0.3.328] Setting up Payment Request API...");
    setup_payment_request_api(scope, context, global)?;
    eprintln!("✅ [v0.3.328] Payment Request API done");
    // v0.3.333: ErrorEvent API (script error handling for WebSocket, Worker, etc.)
    eprintln!("🔧 [v0.3.333] Setting up ErrorEvent API...");
    setup_error_event_api(scope, context);
    eprintln!("✅ [v0.3.333] ErrorEvent API done");
    // v0.3.337: CustomEvent API (custom event handling for AI agents and UI frameworks)
    eprintln!("🔧 [v0.3.337] Setting up CustomEvent API...");
    setup_custom_event_api(scope, context);
    eprintln!("✅ [v0.3.337] CustomEvent API done");
    // v0.3.341: DOMParser API (HTML/XML document parsing for AI workloads)
    eprintln!("🔧 [v0.3.341] Setting up DOMParser API...");
    setup_dom_parser_api(scope, context)?;
    eprintln!("✅ [v0.3.341] DOMParser API done");
    // Note: Streams API is initialized separately in runtime_minimal.rs
    // to avoid duplicate initialization
    eprintln!("🎉 [STAGE74/75] All Web APIs initialized (streams via runtime)!");
    Ok(())
}
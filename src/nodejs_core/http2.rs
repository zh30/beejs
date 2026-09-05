//! Node.js `http2` module compatibility.
use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_http2_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let script_code = r#"
    (function() {
        const EE = globalThis.EventEmitter || function() {};
        const proto = (EE.prototype || Object.prototype);

        function Http2Server() {
            if (typeof EE === 'function') EE.call(this);
        }
        Http2Server.prototype = Object.create(proto);

        function Http2SecureServer() {
            if (typeof EE === 'function') EE.call(this);
        }
        Http2SecureServer.prototype = Object.create(proto);

        function Http2Session() {
            if (typeof EE === 'function') EE.call(this);
        }
        Http2Session.prototype = Object.create(proto);

        const http2 = {
            constants: {
                NGHTTP2_SESSION_SERVER: 1,
                NGHTTP2_SESSION_CLIENT: 2,
                NGHTTP2_FLAG_NONE: 0,
                HTTP2_HEADER_STATUS: ':status',
                HTTP2_HEADER_METHOD: ':method',
                HTTP2_HEADER_AUTHORITY: ':authority',
                HTTP2_HEADER_SCHEME: ':scheme',
                HTTP2_HEADER_PATH: ':path',
            },
            getDefaultSettings() { return {}; },
            getPackedSettings() { return typeof Buffer !== 'undefined' && Buffer.alloc ? Buffer.alloc(0) : []; },
            getUnpackedSettings() { return {}; },
            createServer(options, onRequestHandler) {
                return new Http2Server();
            },
            createSecureServer(options, onRequestHandler) {
                return new Http2SecureServer();
            },
            connect(authority, options, listener) {
                return new Http2Session();
            },
            Http2Server,
            Http2SecureServer,
            Http2Session,
        };
        http2.default = http2;
        return http2;
    })();
    "#;

    let source = v8::String::new(scope, script_code)
        .ok_or_else(|| anyhow::anyhow!("Failed to create http2 bootstrap source"))?;
    let script = v8::Script::compile(scope, source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to compile http2 bootstrap"))?;
    let http2_obj = script
        .run(scope)
        .ok_or_else(|| anyhow::anyhow!("Failed to run http2 bootstrap"))?;

    let global = context.global(scope);
    let key = v8::String::new(scope, "http2").unwrap();
    global.set(scope, key.into(), http2_obj);

    Ok(())
}

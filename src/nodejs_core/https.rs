//! Node.js `https` module — thin wrapper over `http` with TLS intent.
//! Full certificate verification / SNI details continue to land with `tls`.

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_https_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);
    let https = v8::Object::new(scope);

    // Re-export http.request/get/createServer when present; mark module as TLS-capable.
    let http_key = v8::String::new(scope, "http").unwrap();
    if let Some(http_val) = global.get(scope, http_key.into()) {
        if let Ok(http_obj) = v8::Local::<v8::Object>::try_from(http_val) {
            for name in ["request", "get", "createServer", "Agent", "globalAgent"] {
                let key = v8::String::new(scope, name).unwrap();
                if let Some(val) = http_obj.get(scope, key.into()) {
                    https.set(scope, key.into(), val);
                }
            }
        }
    }

    let tls_flag = v8::Boolean::new(scope, true);
    let tls_key = v8::String::new(scope, "_beeUsesTls").unwrap();
    https.set(scope, tls_key.into(), tls_flag.into());

    let key = v8::String::new(scope, "https").unwrap();
    global.set(scope, key.into(), https.into());
    Ok(())
}

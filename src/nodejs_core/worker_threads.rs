//! Node.js `worker_threads` module.
//! Real multi-isolate Workers are provided by `crate::web_api::worker_host`.

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_worker_threads_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    crate::web_api::worker_host::setup_worker_host_api(scope, context)
}

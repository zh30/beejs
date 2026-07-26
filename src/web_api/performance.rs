use anyhow::Result;
use rusty_v8 as v8;

/// Install the shared Performance API implementation used by the default runtime.
pub fn setup_performance_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    crate::nodejs_core::performance::setup_performance_api(scope, context)
}

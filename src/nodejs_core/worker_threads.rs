//! Node.js `worker_threads` module.
//! Real multi-isolate Workers are provided by `crate::web_api::worker_host`.
//! Until a WorkerHost is available, `Worker` construction fails closed.

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_worker_threads_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);
    let mod_obj = v8::Object::new(scope);

    let is_main = v8::Boolean::new(scope, true);
    let is_main_key = v8::String::new(scope, "isMainThread").unwrap();
    mod_obj.set(scope, is_main_key.into(), is_main.into());

    let parent_port = v8::null(scope);
    let pp_key = v8::String::new(scope, "parentPort").unwrap();
    mod_obj.set(scope, pp_key.into(), parent_port.into());

    let worker_data = v8::null(scope);
    let wd_key = v8::String::new(scope, "workerData").unwrap();
    mod_obj.set(scope, wd_key.into(), worker_data.into());

    let thread_id = v8::Integer::new(scope, 0);
    let tid_key = v8::String::new(scope, "threadId").unwrap();
    mod_obj.set(scope, tid_key.into(), thread_id.into());

    let worker_ctor = v8::Function::new(
        scope,
        |scope: &mut v8::HandleScope,
         _args: v8::FunctionCallbackArguments,
         mut rv: v8::ReturnValue| {
            let msg = v8::String::new(
                scope,
                "worker_threads.Worker requires WorkerHost (multi-isolate) support",
            )
            .unwrap();
            let exc = v8::Exception::error(scope, msg);
            scope.throw_exception(exc);
            rv.set(v8::undefined(scope).into());
        },
    )
    .unwrap();
    let worker_key = v8::String::new(scope, "Worker").unwrap();
    mod_obj.set(scope, worker_key.into(), worker_ctor.into());

    let key = v8::String::new(scope, "worker_threads").unwrap();
    global.set(scope, key.into(), mod_obj.into());
    Ok(())
}

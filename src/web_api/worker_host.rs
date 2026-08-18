//! Multi-isolate WorkerHost scaffold.
//!
//! Each worker runs on its own OS thread with an independent V8 isolate.
//! Message passing uses JSON structured-clone subset until ValueSerializer lands.

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

static NEXT_WORKER_ID: AtomicU32 = AtomicU32::new(1);
static HOST: Lazy<Mutex<WorkerHost>> = Lazy::new(|| Mutex::new(WorkerHost::new()));

#[derive(Debug)]
pub enum WorkerMessage {
    PostMessage(String),
    Terminate,
}

pub struct WorkerHandle {
    pub id: u32,
    pub script_url: String,
    tx: Sender<WorkerMessage>,
    join: Option<JoinHandle<()>>,
}

pub struct WorkerHost {
    workers: HashMap<u32, WorkerHandle>,
}

impl WorkerHost {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub fn spawn(script_source: String, script_url: String) -> Result<u32> {
        let id = NEXT_WORKER_ID.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel::<WorkerMessage>();
        let worker_id = id;
        let url_clone = script_url.clone();
        let join = thread::Builder::new()
            .name(format!("bee-worker-{}", id))
            .spawn(move || {
                if let Err(err) = run_worker_thread(worker_id, script_source, rx) {
                    eprintln!("[WorkerHost {}] fatal: {}", worker_id, err);
                }
            })
            .map_err(|e| anyhow!("failed to spawn worker thread: {}", e))?;

        let mut host = HOST.lock().unwrap();
        host.workers.insert(
            id,
            WorkerHandle {
                id,
                script_url: url_clone,
                tx,
                join: Some(join),
            },
        );
        Ok(id)
    }

    pub fn post_message(id: u32, payload_json: String) -> Result<()> {
        let host = HOST.lock().unwrap();
        let worker = host
            .workers
            .get(&id)
            .ok_or_else(|| anyhow!("unknown worker {}", id))?;
        worker
            .tx
            .send(WorkerMessage::PostMessage(payload_json))
            .map_err(|e| anyhow!("postMessage failed: {}", e))
    }

    pub fn terminate(id: u32) -> Result<()> {
        let mut host = HOST.lock().unwrap();
        if let Some(mut worker) = host.workers.remove(&id) {
            let _ = worker.tx.send(WorkerMessage::Terminate);
            if let Some(join) = worker.join.take() {
                let _ = join.join();
            }
        }
        Ok(())
    }

    pub fn active_count() -> usize {
        HOST.lock().unwrap().workers.len()
    }
}

fn run_worker_thread(id: u32, script_source: String, rx: Receiver<WorkerMessage>) -> Result<()> {
    // Independent isolate lifetime for this OS thread.
    crate::initialize_v8()?;
    let mut isolate = v8::Isolate::new(Default::default());
    {
        let scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Minimal console for worker diagnostics
        let global = context.global(scope);
        let console = v8::Object::new(scope);
        let log_fn = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             _rv: v8::ReturnValue| {
                let mut parts = Vec::new();
                for i in 0..args.length() {
                    if let Some(s) = args.get(i).to_string(scope) {
                        parts.push(s.to_rust_string_lossy(scope));
                    }
                }
                println!("{}", parts.join(" "));
            },
        )
        .unwrap();
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), log_fn.into());
        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console.into());

        let source = v8::String::new(scope, &script_source).unwrap();
        if let Some(script) = v8::Script::compile(scope, source, None) {
            let _ = script.run(scope);
        } else {
            return Err(anyhow!("Worker {} failed to compile script", id));
        }

        // Drain control channel until terminate.
        loop {
            match rx.recv() {
                Ok(WorkerMessage::PostMessage(payload)) => {
                    // Best-effort: invoke global onmessage if present.
                    let onmessage_key = v8::String::new(scope, "onmessage").unwrap();
                    if let Some(handler) = global.get(scope, onmessage_key.into()) {
                        if let Ok(func) = v8::Local::<v8::Function>::try_from(handler) {
                            let event = v8::Object::new(scope);
                            let data_key = v8::String::new(scope, "data").unwrap();
                            let data_val = v8::String::new(scope, &payload).unwrap();
                            event.set(scope, data_key.into(), data_val.into());
                            let _ = func.call(scope, global.into(), &[event.into()]);
                        }
                    }
                }
                Ok(WorkerMessage::Terminate) | Err(_) => break,
            }
        }
    }
    Ok(())
}

/// Extract the script body from a `data:[<mediatype>][;base64],<data>` URL.
///
/// Passing the whole URL through as source makes the worker try to execute the
/// `data:,` prefix, which fails to compile.
fn decode_data_url(url: &str) -> Result<String> {
    let body = url
        .strip_prefix("data:")
        .ok_or_else(|| anyhow!("not a data URL: {}", url))?;
    let (metadata, payload) = body
        .split_once(',')
        .ok_or_else(|| anyhow!("data URL is missing the ',' separator: {}", url))?;

    if metadata
        .rsplit(';')
        .next()
        .is_some_and(|last| last.eq_ignore_ascii_case("base64"))
    {
        use base64::Engine as _;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(payload.trim())
            .map_err(|e| anyhow!("data URL has invalid base64 payload: {}", e))?;
        String::from_utf8(decoded).map_err(|e| anyhow!("data URL is not valid UTF-8: {}", e))
    } else {
        Ok(percent_encoding::percent_decode_str(payload)
            .decode_utf8()
            .map_err(|e| anyhow!("data URL is not valid UTF-8: {}", e))?
            .into_owned())
    }
}

/// Setup a progressive Worker constructor that uses WorkerHost when script text is inline.
pub fn setup_worker_host_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);
    let ctor = v8::Function::new(
        scope,
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut rv: v8::ReturnValue| {
            if args.length() == 0 {
                let msg = v8::String::new(scope, "Worker requires a script URL or source").unwrap();
                let exc = v8::Exception::type_error(scope, msg);
                scope.throw_exception(exc);
                return;
            }
            let script_ref = args
                .get(0)
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();

            // Support `new Worker({ source: "..." })` progressive path and file URL later.
            let source = if script_ref.starts_with("data:") {
                match decode_data_url(&script_ref) {
                    Ok(code) => code,
                    Err(err) => {
                        let msg = v8::String::new(scope, &format!("{}", err)).unwrap();
                        let exc = v8::Exception::error(scope, msg);
                        scope.throw_exception(exc);
                        return;
                    }
                }
            } else if script_ref.contains('\n') {
                script_ref.clone()
            } else if let Ok(code) = std::fs::read_to_string(&script_ref) {
                code
            } else {
                let msg = v8::String::new(
                    scope,
                    &format!(
                        "WorkerHost could not load script '{}'. Pass a readable file path.",
                        script_ref
                    ),
                )
                .unwrap();
                let exc = v8::Exception::error(scope, msg);
                scope.throw_exception(exc);
                return;
            };

            match WorkerHost::spawn(source, script_ref) {
                Ok(id) => {
                    let obj = v8::Object::new(scope);
                    let id_key = v8::String::new(scope, "_workerId").unwrap();
                    let id_val = v8::Integer::new(scope, id as i32);
                    obj.set(scope, id_key.into(), id_val.into());

                    let post = v8::Function::new(
                        scope,
                        |scope: &mut v8::HandleScope,
                         args: v8::FunctionCallbackArguments,
                         _rv: v8::ReturnValue| {
                            let this = args.this();
                            let id_key = v8::String::new(scope, "_workerId").unwrap();
                            let id = this
                                .get(scope, id_key.into())
                                .and_then(|v| v.to_uint32(scope))
                                .map(|v| v.value() as u32)
                                .unwrap_or(0);
                            let payload = args
                                .get(0)
                                .to_string(scope)
                                .map(|s| s.to_rust_string_lossy(scope))
                                .unwrap_or_else(|| "null".into());
                            let _ = WorkerHost::post_message(id, payload);
                        },
                    )
                    .unwrap();
                    let post_key = v8::String::new(scope, "postMessage").unwrap();
                    obj.set(scope, post_key.into(), post.into());

                    let terminate = v8::Function::new(
                        scope,
                        |scope: &mut v8::HandleScope,
                         args: v8::FunctionCallbackArguments,
                         _rv: v8::ReturnValue| {
                            let this = args.this();
                            let id_key = v8::String::new(scope, "_workerId").unwrap();
                            let id = this
                                .get(scope, id_key.into())
                                .and_then(|v| v.to_uint32(scope))
                                .map(|v| v.value() as u32)
                                .unwrap_or(0);
                            let _ = WorkerHost::terminate(id);
                        },
                    )
                    .unwrap();
                    let term_key = v8::String::new(scope, "terminate").unwrap();
                    obj.set(scope, term_key.into(), terminate.into());

                    rv.set(obj.into());
                }
                Err(err) => {
                    let msg = v8::String::new(scope, &format!("{}", err)).unwrap();
                    let exc = v8::Exception::error(scope, msg);
                    scope.throw_exception(exc);
                }
            }
        },
    )
    .unwrap();

    // Expose as BeeWorkerHost so existing fail-closed Worker can be swapped gradually.
    let key = v8::String::new(scope, "BeeWorkerHost").unwrap();
    global.set(scope, key.into(), ctor.into());
    // Progressive: also install as Worker when host is ready.
    let worker_key = v8::String::new(scope, "Worker").unwrap();
    global.set(scope, worker_key.into(), ctor.into());
    Ok(())
}

impl Default for WorkerHost {
    fn default() -> Self {
        Self::new()
    }
}

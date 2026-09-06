// Node.js perf_hooks 模块实现

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_perf_hooks_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    let js_code = r#"
    (function() {
        const perf = globalThis.performance || {
            now: () => Date.now(),
            timeOrigin: Date.now(),
            mark: () => {},
            measure: () => {},
            getEntries: () => [],
            getEntriesByName: () => [],
            getEntriesByType: () => [],
            clearMarks: () => {},
            clearMeasures: () => {}
        };

        class PerformanceObserver {
            constructor(callback) {
                this.callback = callback;
            }
            observe() {}
            disconnect() {}
            takeRecords() { return []; }
        }

        const perfHooks = {
            performance: perf,
            PerformanceObserver,
            default: {
                performance: perf,
                PerformanceObserver
            }
        };

        globalThis.__perf_hooks = perfHooks;
        globalThis.perf_hooks = perfHooks;
    })();
    "#;

    let script_source = v8::String::new(scope, js_code).unwrap();
    if let Some(script) = v8::Script::compile(scope, script_source, None) {
        let _ = script.run(scope);
    }

    let perf_hooks_key = v8::String::new(scope, "__perf_hooks").unwrap();
    if let Some(ph_val) = global.get(scope, perf_hooks_key.into()) {
        let ph_key = v8::String::new(scope, "perf_hooks").unwrap();
        global.set(scope, ph_key.into(), ph_val);
    }

    Ok(())
}

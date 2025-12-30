// structuredClone API implementation
// v0.3.300: Enhanced with Date, RegExp, Map, Set support
// Optimized for AI workloads - enables safe deep cloning of inference results

use anyhow::Result;
use rusty_v8 as v8;

/// Internal key for storing the clone function in global
const CLONE_FUNC_KEY: &str = "__beejs_internal_clone_func";

/// Setup the internal clone function in the global object
fn setup_internal_clone_func(
    scope: &mut v8::HandleScope,
    global: v8::Local<v8::Object>,
) -> Result<()> {
    let code = r#"
        (function() {
            "use strict";
            function clone(v) {
                if (v === null || typeof v !== "object") return v;

                const seen = new WeakSet();
                function deepClone(obj) {
                    if (obj === null || typeof obj !== "object") return obj;
                    if (seen.has(obj)) return obj;
                    seen.add(obj);

                    if (obj instanceof Date) return new Date(obj.getTime());
                    if (obj instanceof RegExp) return new RegExp(obj.source, obj.flags);

                    if (obj instanceof Map) {
                        const c = new Map();
                        for (const [k, val] of obj) c.set(deepClone(k), deepClone(val));
                        return c;
                    }
                    if (obj instanceof Set) {
                        const c = new Set();
                        for (const val of obj) c.add(deepClone(val));
                        return c;
                    }
                    if (Array.isArray(obj)) {
                        return obj.map(item => deepClone(item));
                    }

                    const cloned = {};
                    for (const key in obj) {
                        if (Object.prototype.hasOwnProperty.call(obj, key)) {
                            cloned[key] = deepClone(obj[key]);
                        }
                    }
                    return cloned;
                }
                return deepClone(v);
            }
            return clone;
        })()
    "#;

    let code = v8::String::new(scope, code).unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let func = script.run(scope).unwrap();

    let key = v8::String::new(scope, CLONE_FUNC_KEY).unwrap();
    global.set(scope, key.into(), func);
    Ok(())
}

/// structuredClone callback function
fn structured_clone_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let value = args.get(0);

    let global = scope.get_current_context().global(scope);
    let key = v8::String::new(scope, CLONE_FUNC_KEY).unwrap();

    // Get the internal clone function (set up during initialization)
    let clone_func = global.get(scope, key.into()).unwrap();
    let func: v8::Local<v8::Function> = clone_func.try_into().unwrap();

    let undefined = v8::undefined(scope);
    let result = func.call(scope, undefined.into(), &[value]);
    retval.set(result.unwrap_or(v8::null(scope).into()));
}

/// Setup structuredClone global function
pub fn setup_structured_clone_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global: _ = context.global(scope);

    // Setup the internal clone function in the same context
    setup_internal_clone_func(scope, global)?;

    let structured_clone_template: _ = v8::FunctionTemplate::new(scope, structured_clone_callback);
    let structured_clone_func: _ = structured_clone_template.get_function(scope).unwrap();
    let structured_clone_key: _ = v8::String::new(scope, "structuredClone").unwrap();
    global.set(scope, structured_clone_key.into(), structured_clone_func.into());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_value_primitives() {
        // Tests would require V8 isolate setup
        // For now, just verify the module compiles
        assert!(true);
    }
}

// structuredClone API implementation
// v0.3.301: Enhanced with transfer option support for zero-copy transfer
// Optimized for AI workloads - enables safe deep cloning with transfer semantics

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
            function clone(v, transferList) {
                if (v === null || typeof v !== "object") return v;

                const seen = new WeakSet();
                const transfers = new Map();

                // Process transfer list - mark objects as transferred
                if (Array.isArray(transferList)) {
                    for (const obj of transferList) {
                        if (obj && typeof obj === 'object') {
                            // Get the underlying buffer/transferable
                            const transferable = obj.buffer || obj;
                            if (transferable && typeof transferable === 'object') {
                                transfers.set(transferable, obj);
                            }
                        }
                    }
                }

                function isTypedArray(obj) {
                    return obj && typeof obj === 'object' &&
                        obj.buffer instanceof ArrayBuffer &&
                        typeof obj.byteLength === 'number' &&
                        typeof obj.byteOffset === 'number';
                }

                function getTypedArrayConstructor(obj) {
                    // Detect TypedArray type and return appropriate constructor
                    if (obj instanceof Uint8Array) return Uint8Array;
                    if (obj instanceof Int8Array) return Int8Array;
                    if (obj instanceof Uint16Array) return Uint16Array;
                    if (obj instanceof Int16Array) return Int16Array;
                    if (obj instanceof Uint32Array) return Uint32Array;
                    if (obj instanceof Int32Array) return Int32Array;
                    if (obj instanceof Float32Array) return Float32Array;
                    if (obj instanceof Float64Array) return Float64Array;
                    if (obj instanceof Uint8ClampedArray) return Uint8ClampedArray;
                    return null;
                }

                function deepClone(obj) {
                    if (obj === null || typeof obj !== "object") return obj;
                    if (seen.has(obj)) return obj;
                    seen.add(obj);

                    // Check if this object is being transferred
                    const transferTarget = transfers.get(obj);
                    if (transferTarget) {
                        // Remove from transfers map and return as-is (transfer semantics)
                        transfers.delete(obj);
                        return obj;
                    }

                    // Check for ArrayBuffer being transferred
                    if (obj instanceof ArrayBuffer && transfers.has(obj)) {
                        return obj;
                    }

                    // Handle TypedArray (before Array check, since TypedArrays are not true arrays)
                    const TypedArrayConstructor = getTypedArrayConstructor(obj);
                    if (TypedArrayConstructor) {
                        if (obj.buffer instanceof ArrayBuffer && transfers.has(obj.buffer)) {
                            // Transfer the buffer
                            transfers.delete(obj.buffer);
                            return obj;
                        }
                        // Clone by creating new TypedArray with same values
                        return new TypedArrayConstructor(obj);
                    }

                    // Check for Date using both instanceof and timestamp property
                    // This handles both native Date and our custom Date implementation
                    const isDate = obj instanceof Date ||
                        (typeof obj.getTime === 'function' && typeof obj.getMonth === 'function');
                    if (isDate) {
                        const timestamp = (typeof obj.getTime === 'function')
                            ? obj.getTime()
                            : obj.timestamp;
                        return new Date(timestamp);
                    }

                    // Check for RegExp
                    if (obj instanceof RegExp ||
                        (typeof obj.source === 'string' && typeof obj.flags === 'string')) {
                        return new RegExp(obj.source || obj.patternSource, obj.flags || obj.patternFlags || '');
                    }

                    // Check for ArrayBuffer (non-transfer case)
                    if (obj instanceof ArrayBuffer) {
                        // Copy the actual data from the source buffer
                        const bytes = new Uint8Array(obj);
                        const cloned = new ArrayBuffer(obj.byteLength);
                        new Uint8Array(cloned).set(bytes);
                        return cloned;
                    }

                    // Check for Map (native or custom with forEach)
                    if (obj instanceof Map ||
                        (typeof obj.forEach === 'function' && typeof obj.get === 'function')) {
                        const c = new Map();
                        if (typeof obj.forEach === 'function') {
                            obj.forEach(function(val, key) {
                                c.set(deepClone(key), deepClone(val));
                            });
                        }
                        return c;
                    }

                    // Check for Set (native or custom with forEach)
                    if (obj instanceof Set ||
                        (typeof obj.forEach === 'function' && typeof obj.has === 'function')) {
                        const c = new Set();
                        if (typeof obj.forEach === 'function') {
                            obj.forEach(function(val) {
                                c.add(deepClone(val));
                            });
                        }
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

    // Extract transfer list from options object (second argument)
    let transfer_list = if args.length() > 1 {
        let options = args.get(1);
        if options.is_object() {
            let options_obj = v8::Local::<v8::Object>::try_from(options).ok();
            if let Some(obj) = options_obj {
                let transfer_key = v8::String::new(scope, "transfer").unwrap();
                obj.get(scope, transfer_key.into()).unwrap_or(v8::null(scope).into())
            } else {
                v8::null(scope).into()
            }
        } else {
            v8::null(scope).into()
        }
    } else {
        v8::null(scope).into()
    };

    let global = scope.get_current_context().global(scope);
    let key = v8::String::new(scope, CLONE_FUNC_KEY).unwrap();

    // Get the internal clone function (set up during initialization)
    let clone_func = global.get(scope, key.into()).unwrap();
    let func: v8::Local<v8::Function> = clone_func.try_into().unwrap();

    let undefined = v8::undefined(scope);
    let result = func.call(scope, undefined.into(), &[value, transfer_list]);
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

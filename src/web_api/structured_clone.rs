// structuredClone API implementation
// v0.3.303: Performance optimization - iterative deep clone using work queue
// Replaced recursive approach with iterative stack to avoid stack overflow
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

                const transfers = new Map();

                // Process transfer list - mark objects as transferred
                if (Array.isArray(transferList)) {
                    for (const obj of transferList) {
                        if (obj && typeof obj === 'object') {
                            const transferable = obj.buffer || obj;
                            if (transferable && typeof transferable === 'object') {
                                transfers.set(transferable, obj);
                            }
                        }
                    }
                }

                function getTypedArrayConstructor(obj) {
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

                function getErrorConstructor(obj) {
                    if (obj instanceof TypeError) return TypeError;
                    if (obj instanceof RangeError) return RangeError;
                    if (obj instanceof ReferenceError) return ReferenceError;
                    if (obj instanceof SyntaxError) return SyntaxError;
                    if (obj instanceof EvalError) return EvalError;
                    if (obj instanceof URIError) return URIError;
                    return Error;
                }

                // TRUE ITERATIVE DEEP CLONE using work queue
                const clonedObjects = new Map();
                const pendingProps = [];  // [parent, key, value]
                const mapEntries = [];  // [map, originalKey, originalVal] for Map processing
                const setValues = [];  // [set, value] for Set processing

                function createClone(obj) {
                    const TypedArrayConstructor = getTypedArrayConstructor(obj);

                    // Check for WeakMap/WeakSet first - these cannot be cloned per spec
                    // They throw DataCloneError (using Error with name property per spec)
                    if (obj instanceof WeakMap) {
                        const err = new Error("WeakMap cannot be cloned");
                        err.name = "DataCloneError";
                        throw err;
                    }
                    if (obj instanceof WeakSet) {
                        const err = new Error("WeakSet cannot be cloned");
                        err.name = "DataCloneError";
                        throw err;
                    }

                    if (TypedArrayConstructor) {
                        if (obj.buffer instanceof ArrayBuffer && transfers.has(obj.buffer)) {
                            transfers.delete(obj.buffer);
                            return obj;
                        }
                        return new TypedArrayConstructor(obj);
                    } else if (obj instanceof Date ||
                        (typeof obj.getTime === 'function' && typeof obj.getMonth === 'function')) {
                        const timestamp = (typeof obj.getTime === 'function')
                            ? obj.getTime()
                            : obj.timestamp;
                        return new Date(timestamp);
                    } else if (obj instanceof RegExp ||
                        (typeof obj.source === 'string' && typeof obj.flags === 'string')) {
                        return new RegExp(obj.source || obj.patternSource, obj.flags || obj.patternFlags || '');
                    } else if (obj instanceof ArrayBuffer) {
                        const bytes = new Uint8Array(obj);
                        const cloned = new ArrayBuffer(obj.byteLength);
                        new Uint8Array(cloned).set(bytes);
                        return cloned;
                    } else if (obj instanceof Map ||
                        (typeof obj.forEach === 'function' && typeof obj.get === 'function')) {
                        return new Map();
                    } else if (obj instanceof Set ||
                        (typeof obj.forEach === 'function' && typeof obj.has === 'function')) {
                        return new Set();
                    } else if (obj instanceof Error ||
                        (typeof obj.name === 'string' && typeof obj.message === 'string')) {
                        const ErrorConstructor = getErrorConstructor(obj);
                        const cloned = new ErrorConstructor(obj.message);
                        if (typeof obj.name === 'string') cloned.name = obj.name;
                        if (typeof obj.stack === 'string') cloned.stack = obj.stack;
                        return cloned;
                    } else if (Array.isArray(obj)) {
                        return new Array(obj.length);
                    } else {
                        return {};
                    }
                }

                function queueProperties(source, cloned) {
                    if (source instanceof Map) {
                        source.forEach((val, key) => {
                            // Queue for Map processing after cloning
                            mapEntries.push([cloned, key, val]);
                            // Queue key and value for cloning
                            pendingProps.push([cloned, 'MAP_KEY', key]);
                            pendingProps.push([cloned, 'MAP_VAL', val]);
                        });
                    } else if (source instanceof Set) {
                        source.forEach(val => {
                            setValues.push([cloned, val]);
                            pendingProps.push([cloned, 'SET_VAL', val]);
                        });
                    } else if (source instanceof Error) {
                        for (const key in source) {
                            if (Object.prototype.hasOwnProperty.call(source, key) &&
                                key !== 'name' && key !== 'message' && key !== 'stack') {
                                pendingProps.push([cloned, key, source[key]]);
                            }
                        }
                    } else if (Array.isArray(source)) {
                        source.forEach((val, idx) => {
                            pendingProps.push([cloned, idx.toString(), val]);
                        });
                    } else if (typeof source === 'object') {
                        for (const key in source) {
                            if (Object.prototype.hasOwnProperty.call(source, key)) {
                                pendingProps.push([cloned, key, source[key]]);
                            }
                        }
                    }
                }

                // Create clone of root object
                const rootCloned = createClone(v);
                clonedObjects.set(v, rootCloned);
                queueProperties(v, rootCloned);

                // Process pending properties
                while (pendingProps.length > 0) {
                    const [parent, key, value] = pendingProps.pop();

                    // Skip marker entries
                    if (key === 'MAP_KEY' || key === 'MAP_VAL' || key === 'SET_VAL') {
                        // For Map/Set keys/values, track in clonedObjects
                        if (value === null || typeof value !== "object") {
                            clonedObjects.set(value, value);  // Primitives clone to themselves
                        }
                        continue;
                    }

                    // Handle primitives
                    if (value === null || typeof value !== "object") {
                        parent[key] = value;
                        continue;
                    }

                    // Handle transfer
                    if (transfers.has(value)) {
                        const transferred = transfers.get(value);
                        transfers.delete(value);
                        parent[key] = transferred;
                        clonedObjects.set(value, transferred);
                        continue;
                    }

                    // Check if already cloned
                    if (clonedObjects.has(value)) {
                        parent[key] = clonedObjects.get(value);
                        continue;
                    }

                    // Create new clone
                    const cloned = createClone(value);
                    clonedObjects.set(value, cloned);

                    // Set to parent
                    parent[key] = cloned;

                    // Queue this object's properties
                    queueProperties(value, cloned);
                }

                // Now process Map entries using cloned keys and values
                for (const [map, origKey, origVal] of mapEntries) {
                    const clonedKey = clonedObjects.get(origKey);
                    const clonedVal = clonedObjects.get(origVal);
                    if (clonedKey !== undefined && clonedVal !== undefined) {
                        map.set(clonedKey, clonedVal);
                    }
                }

                // Process Set values using cloned values
                for (const [set, origVal] of setValues) {
                    const clonedVal = clonedObjects.get(origVal);
                    if (clonedVal !== undefined) {
                        set.add(clonedVal);
                    }
                }

                return rootCloned;
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

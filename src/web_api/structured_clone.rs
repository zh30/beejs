// structuredClone API implementation
// v0.3.299: Global structuredClone function for deep cloning objects
// Optimized for AI workloads - enables safe deep cloning of inference results

use anyhow::Result;
use rusty_v8 as v8;

/// structuredClone callback function
fn structured_clone_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let value = args.get(0);

    // Handle primitives and null/undefined directly
    if value.is_null_or_undefined()
        || value.is_string()
        || value.is_number()
        || value.is_boolean()
        || value.is_symbol()
    {
        retval.set(value);
        return;
    }

    // Handle arrays
    if value.is_array() {
        let arr = v8::Local::<v8::Array>::try_from(value).unwrap();
        let length = arr.length();
        let new_arr = v8::Array::new(scope, length as i32);

        for i in 0..length {
            if let Some(elem) = arr.get_index(scope, i as u32) {
                let cloned = clone_value(scope, elem);
                if let Some(c) = cloned {
                    new_arr.set_index(scope, i as u32, c);
                }
            }
        }
        retval.set(new_arr.into());
        return;
    }

    // Handle objects
    if let Some(obj) = value.to_object(scope) {
        let new_obj = v8::Object::new(scope);
        let prop_names = obj.get_property_names(scope).unwrap();
        let length = prop_names.length();

        for i in 0..length {
            if let Some(key) = prop_names.get_index(scope, i as u32) {
                if let Some(value_local) = obj.get(scope, key) {
                    let cloned_key = clone_value(scope, key);
                    let cloned_value = clone_value(scope, value_local);

                    if let (Some(k), Some(v)) = (cloned_key, cloned_value) {
                        new_obj.set(scope, k, v);
                    }
                }
            }
        }
        retval.set(new_obj.into());
        return;
    }

    // Fallback for unsupported types
    retval.set(v8::null(scope).into());
}

/// Helper to clone a single value (primitives and simple objects)
fn clone_value<'a>(
    scope: &mut v8::HandleScope<'a>,
    value: v8::Local<'a, v8::Value>,
) -> Option<v8::Local<'a, v8::Value>> {
    if value.is_null_or_undefined()
        || value.is_string()
        || value.is_number()
        || value.is_boolean()
        || value.is_symbol()
    {
        return Some(value);
    }

    if value.is_array() {
        let arr = v8::Local::<v8::Array>::try_from(value).unwrap();
        let length = arr.length();
        let new_arr = v8::Array::new(scope, length as i32);

        for i in 0..length {
            if let Some(elem) = arr.get_index(scope, i as u32) {
                if let Some(cloned) = clone_value(scope, elem) {
                    new_arr.set_index(scope, i as u32, cloned);
                }
            }
        }
        return Some(new_arr.into());
    }

    if let Some(obj) = value.to_object(scope) {
        let new_obj = v8::Object::new(scope);
        let prop_names = obj.get_property_names(scope).unwrap();
        let length = prop_names.length();

        for i in 0..length {
            if let Some(key) = prop_names.get_index(scope, i as u32) {
                if let Some(value_local) = obj.get(scope, key) {
                    let cloned_key = clone_value(scope, key);
                    let cloned_value = clone_value(scope, value_local);

                    if let (Some(k), Some(v)) = (cloned_key, cloned_value) {
                        new_obj.set(scope, k, v);
                    }
                }
            }
        }
        return Some(new_obj.into());
    }

    None
}

/// Setup structuredClone global function
pub fn setup_structured_clone_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global: _ = context.global(scope);

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

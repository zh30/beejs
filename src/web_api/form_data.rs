// FormData API implementation per Web standard
//
// The FormData interface provides a way to construct a set of key/value pairs
/// representing form fields and their values, which can be sent using fetch().
use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Thread-safe FormData storage
static FORMDATA_CACHE: OnceLock<Mutex<HashMap<usize, Vec<FormDataEntry>>>> = OnceLock::new();

/// Get the FormData cache mutex
fn get_formdata_cache() -> &'static Mutex<HashMap<usize, Vec<FormDataEntry>>> {
    FORMDATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// FormData entry - can be simple value or file
#[derive(Debug, Clone)]
pub struct FormDataEntry {
    pub name: String,
    pub value: String,
    pub filename: Option<String>,
    pub content_type: String,
}

/// Setup FormData API in V8 context
pub fn setup_form_data_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    eprintln!("🔧 [STAGE74] Setting up FormData API...");
    // FormData constructor
    let form_data_template: _ = v8::FunctionTemplate::new(scope, form_data_constructor);
    let form_data_constructor: _ = form_data_template.get_function(scope).unwrap();
    // Register FormData constructor
    let global: _ = context.global(scope);
    let form_data_key: _ = v8::String::new(scope, "FormData").unwrap();
    global.set(scope, form_data_key.into(), form_data_constructor.into());
    eprintln!("✅ [STAGE74] FormData API complete");
    Ok(())
}
/// FormData constructor callback
fn form_data_constructor(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Create ObjectTemplate with internal field for storing FormData index
    let form_data_template = v8::ObjectTemplate::new(scope);
    form_data_template.set_internal_field_count(1);

    let form_data_obj: v8::Local<v8::Object> = match form_data_template.new_instance(scope) {
        Some(obj) => obj,
        None => {
            retval.set(v8::null(scope).into());
            return;
        }
    };

    // Get next available index for this FormData instance
    static FORMDATA_INDEX_COUNTER: OnceLock<Mutex<usize>> = OnceLock::new();
    let index_counter = FORMDATA_INDEX_COUNTER.get_or_init(|| Mutex::new(0));
    let mut counter = index_counter.lock().unwrap();
    let index = *counter;
    *counter += 1;
    drop(counter);

    // Store index in internal field 0
    let index_val: v8::Local<v8::Value> = v8::Integer::new(scope, index as i32).into();
    form_data_obj.set_internal_field(0, index_val);

    // Initialize FormData entries for this index
    let mut cache = get_formdata_cache().lock().unwrap();
    cache.insert(index, Vec::new());
    drop(cache);

    // Add append method
    let append_key = v8::String::new(scope, "append").unwrap().into();
    let append_template = v8::FunctionTemplate::new(scope, form_data_append);
    let append_func = append_template.get_function(scope).unwrap();
    form_data_obj.set(scope, append_key, append_func.into());

    // Add delete method
    let delete_key = v8::String::new(scope, "delete").unwrap().into();
    let delete_template = v8::FunctionTemplate::new(scope, form_data_delete);
    let delete_func = delete_template.get_function(scope).unwrap();
    form_data_obj.set(scope, delete_key, delete_func.into());

    // Add get method
    let get_key = v8::String::new(scope, "get").unwrap().into();
    let get_template = v8::FunctionTemplate::new(scope, form_data_get);
    let get_func = get_template.get_function(scope).unwrap();
    form_data_obj.set(scope, get_key, get_func.into());

    // Add getAll method
    let get_all_key = v8::String::new(scope, "getAll").unwrap().into();
    let get_all_template = v8::FunctionTemplate::new(scope, form_data_get_all);
    let get_all_func = get_all_template.get_function(scope).unwrap();
    form_data_obj.set(scope, get_all_key, get_all_func.into());

    // Add has method
    let has_key = v8::String::new(scope, "has").unwrap().into();
    let has_template = v8::FunctionTemplate::new(scope, form_data_has);
    let has_func = has_template.get_function(scope).unwrap();
    form_data_obj.set(scope, has_key, has_func.into());

    // Add set method
    let set_key = v8::String::new(scope, "set").unwrap().into();
    let set_template = v8::FunctionTemplate::new(scope, form_data_set);
    let set_func = set_template.get_function(scope).unwrap();
    form_data_obj.set(scope, set_key, set_func.into());

    // Add entries method
    let entries_key = v8::String::new(scope, "entries").unwrap().into();
    let entries_template = v8::FunctionTemplate::new(scope, form_data_entries);
    let entries_func = entries_template.get_function(scope).unwrap();
    form_data_obj.set(scope, entries_key, entries_func.into());

    // Add keys method
    let keys_key = v8::String::new(scope, "keys").unwrap().into();
    let keys_template = v8::FunctionTemplate::new(scope, form_data_keys);
    let keys_func = keys_template.get_function(scope).unwrap();
    form_data_obj.set(scope, keys_key, keys_func.into());

    // Add values method
    let values_key = v8::String::new(scope, "values").unwrap().into();
    let values_template = v8::FunctionTemplate::new(scope, form_data_values);
    let values_func = values_template.get_function(scope).unwrap();
    form_data_obj.set(scope, values_key, values_func.into());

    // Add forEach method
    let for_each_key = v8::String::new(scope, "forEach").unwrap().into();
    let for_each_template = v8::FunctionTemplate::new(scope, form_data_for_each);
    let for_each_func = for_each_template.get_function(scope).unwrap();
    form_data_obj.set(scope, for_each_key, for_each_func.into());

    retval.set(form_data_obj.into());
}

/// FormData.append() method - adds a new value to an existing key
/// or adds the key if it doesn't exist
fn form_data_append(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    // Get index from internal field
    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    // Get name
    let name = if let Some(name_val) = args.get(0).to_string(scope) {
        name_val.to_rust_string_lossy(scope)
    } else {
        return;
    };

    // Get value (can be string or Blob/File)
    let (value, _filename, content_type) = if let Some(value_val) = args.get(1).to_string(scope) {
        (value_val.to_rust_string_lossy(scope), None::<String>, "text/plain".to_string())
    } else if args.get(1).is_object() {
        // Handle Blob-like objects
        if let Some(obj) = args.get(1).to_object(scope) {
            // Check if it has a type property
            let type_key = v8::String::new(scope, "type").unwrap().into();
            let content_type = if let Some(t) = obj.get(scope, type_key) {
                t.to_string(scope)
                    .map(|s| s.to_rust_string_lossy(scope))
                    .unwrap_or_else(|| "application/octet-stream".to_string())
            } else {
                "application/octet-stream".to_string()
            };

            // Check if it has a size property for arrayBuffer extraction
            let size_key = v8::String::new(scope, "size").unwrap().into();
            let _size = obj.get(scope, size_key)
                .and_then(|v| v.to_integer(scope))
                .map(|i| i.value() as usize)
                .unwrap_or(0);

            // Check for arrayBuffer method
            let array_buffer_key = v8::String::new(scope, "arrayBuffer").unwrap().into();
            let has_array_buffer = obj.has(scope, array_buffer_key).unwrap_or(false);
            if has_array_buffer {
                // For now, return a placeholder - full blob support requires async handling
                ("[Blob data]".to_string(), None, content_type)
            } else {
                ("[Object]".to_string(), None, content_type)
            }
        } else {
            return;
        }
    } else {
        return;
    };

    // Get optional filename (third argument)
    let filename = if let Some(filename_val) = args.get(2).to_string(scope) {
        Some(filename_val.to_rust_string_lossy(scope))
    } else {
        None
    };

    // Store in cache
    let mut cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get_mut(&index) {
        entries.push(FormDataEntry {
            name,
            value,
            filename,
            content_type,
        });
    }
}

/// FormData.delete() method - removes all values associated with a key
fn form_data_delete(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let name = if let Some(name_val) = args.get(0).to_string(scope) {
        name_val.to_rust_string_lossy(scope)
    } else {
        return;
    };

    let mut cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get_mut(&index) {
        entries.retain(|entry| entry.name != name);
    }
}

/// FormData.get() method - returns the first value associated with a key
fn form_data_get(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let name = if let Some(name_val) = args.get(0).to_string(scope) {
        name_val.to_rust_string_lossy(scope)
    } else {
        retval.set(v8::null(scope).into());
        return;
    };

    let cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get(&index) {
        for entry in entries {
            if entry.name == name {
                retval.set(v8::String::new(scope, &entry.value).unwrap().into());
                return;
            }
        }
    }
    retval.set(v8::null(scope).into());
}

/// FormData.getAll() method - returns all values associated with a key
fn form_data_get_all(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let name = if let Some(name_val) = args.get(0).to_string(scope) {
        name_val.to_rust_string_lossy(scope)
    } else {
        retval.set(v8::Array::new(scope, 0).into());
        return;
    };

    let cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get(&index) {
        let values: Vec<_> = entries.iter()
            .filter(|entry| entry.name == name)
            .map(|entry| {
                v8::String::new(scope, &entry.value).unwrap().into()
            })
            .collect();
        let array = v8::Array::new_with_elements(scope, &values);
        retval.set(array.into());
    } else {
        retval.set(v8::Array::new(scope, 0).into());
    }
}

/// FormData.has() method - returns whether a key exists
fn form_data_has(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let name = if let Some(name_val) = args.get(0).to_string(scope) {
        name_val.to_rust_string_lossy(scope)
    } else {
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    };

    let cache = get_formdata_cache().lock().unwrap();
    let has_key = cache.get(&index)
        .map(|entries| entries.iter().any(|e| e.name == name))
        .unwrap_or(false);

    retval.set(v8::Boolean::new(scope, has_key).into());
}

/// FormData.set() method - sets a new value for a key, replacing existing values
fn form_data_set(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let name = if let Some(name_val) = args.get(0).to_string(scope) {
        name_val.to_rust_string_lossy(scope)
    } else {
        return;
    };

    let value = if let Some(value_val) = args.get(1).to_string(scope) {
        value_val.to_rust_string_lossy(scope)
    } else if args.get(1).is_object() {
        "[Object]".to_string()
    } else {
        return;
    };

    let filename = if let Some(filename_val) = args.get(2).to_string(scope) {
        Some(filename_val.to_rust_string_lossy(scope))
    } else {
        None
    };

    let mut cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get_mut(&index) {
        // Remove all existing entries with this name
        entries.retain(|entry| entry.name != name);
        // Add new entry
        entries.push(FormDataEntry {
            name,
            value,
            filename,
            content_type: "text/plain".to_string(),
        });
    }
}

/// FormData.entries() method - returns an iterator of key/value pairs
fn form_data_entries(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get(&index) {
        // Create an array of [name, value] pairs
        let pairs: Vec<v8::Local<v8::Value>> = entries.iter()
            .map(|entry| {
                let pair = v8::Array::new(scope, 2);
                let name_str = v8::String::new(scope, &entry.name).unwrap();
                let value_str = v8::String::new(scope, &entry.value).unwrap();
                pair.set_index(scope, 0, name_str.into());
                pair.set_index(scope, 1, value_str.into());
                pair.into()
            })
            .collect();
        let array = v8::Array::new_with_elements(scope, &pairs);
        retval.set(array.into());
    } else {
        retval.set(v8::Array::new(scope, 0).into());
    }
}

/// FormData.keys() method - returns an iterator of keys
fn form_data_keys(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get(&index) {
        let unique_keys: std::collections::HashSet<_> = entries.iter()
            .map(|e| e.name.clone())
            .collect();
        let keys: Vec<_> = unique_keys.iter()
            .map(|k| v8::String::new(scope, k).unwrap().into())
            .collect();
        let array = v8::Array::new_with_elements(scope, &keys);
        retval.set(array.into());
    } else {
        retval.set(v8::Array::new(scope, 0).into());
    }
}

/// FormData.values() method - returns an iterator of values
fn form_data_values(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj: v8::Local<v8::Object> = args.this();

    let index = this_obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
        .unwrap_or(usize::MAX);

    let cache = get_formdata_cache().lock().unwrap();
    if let Some(entries) = cache.get(&index) {
        let values: Vec<_> = entries.iter()
            .map(|e| v8::String::new(scope, &e.value).unwrap().into())
            .collect();
        let array = v8::Array::new_with_elements(scope, &values);
        retval.set(array.into());
    } else {
        retval.set(v8::Array::new(scope, 0).into());
    }
}

/// FormData.forEach() method - iterates over all key/value pairs
fn form_data_for_each(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    // ForEach requires more complex V8 callback handling
    // For now, this is a stub - the core functionality (append, get, etc.) is implemented
    // Full forEach support would require proper callback scope management
}

/// Export FormData entries for use with fetch
/// Returns the entries as a Vec for serialization
pub fn get_formdata_entries(index: usize) -> Option<Vec<FormDataEntry>> {
    let cache = FORMDATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = cache.lock().unwrap();
    guard.get(&index).cloned()
}

/// Check if a V8 value is a FormData object and return its internal index
pub fn get_formdata_index(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Option<usize> {
    if !value.is_object() {
        return None;
    }

    let obj = value.to_object(scope)?;

    // Check if object has internal fields (our FormData uses ObjectTemplate with internal field count)
    if obj.internal_field_count() < 1 {
        return None;
    }

    // Check if the object has FormData methods
    let append_key = v8::String::new(scope, "append").unwrap().into();
    if !obj.has(scope, append_key).unwrap_or(false) {
        return None;
    }

    // Get the internal field which stores the FormData index
    obj.get_internal_field(scope, 0)
        .and_then(|v| v.to_integer(scope))
        .map(|i| i.value() as usize)
}

/// Serialize FormData entries to multipart/form-data format
pub fn serialize_formdata_multipart(entries: &[FormDataEntry], boundary: &str) -> Vec<u8> {
    let mut result = Vec::new();

    for entry in entries {
        // Write boundary
        result.extend_from_slice(b"--");
        result.extend_from_slice(boundary.as_bytes());
        result.extend_from_slice(b"\r\n");

        // Write Content-Disposition header
        result.extend_from_slice(b"Content-Disposition: form-data; name=\"");
        result.extend_from_slice(entry.name.as_bytes());
        result.extend_from_slice(b"\"");

        // Add filename if present
        if let Some(filename) = &entry.filename {
            result.extend_from_slice(b"; filename=\"");
            result.extend_from_slice(filename.as_bytes());
            result.extend_from_slice(b"\"");
        }
        result.extend_from_slice(b"\r\n");

        // Write Content-Type header
        if !entry.content_type.is_empty() {
            result.extend_from_slice(b"Content-Type: ");
            result.extend_from_slice(entry.content_type.as_bytes());
            result.extend_from_slice(b"\r\n");
        }

        // Empty line before body
        result.extend_from_slice(b"\r\n");

        // Write body
        result.extend_from_slice(entry.value.as_bytes());
        result.extend_from_slice(b"\r\n");
    }

    // Write final boundary
    result.extend_from_slice(b"--");
    result.extend_from_slice(boundary.as_bytes());
    result.extend_from_slice(b"--\r\n");

    result
}

/// Generate a random boundary string for multipart/form-data
pub fn generate_boundary() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random: u128 = rng.gen();
    format!("----BeejsFormBoundary{}", random)
}
// Blob API implementation per Web standard
// Provides binary data container with File API support


use anyhow::Result;
use rusty_v8 as v8;
/// Setup Blob and File APIs in V8 context
pub fn setup_blob_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    eprintln!("🔧 [STAGE74] Setting up blob API...");
    let global: _ = context.global(scope);
    // Setup Blob constructor
    let blob_template: _ = v8::FunctionTemplate::new(scope, blob_constructor);
    let blob_constructor: _ = blob_template.get_function(scope).unwrap();
    let blob_key: _ = v8::String::new(scope, "Blob").unwrap();
    global.set(scope, blob_key.into(), blob_constructor.into());
    eprintln!("✅ [STAGE74] Blob constructor registered");
    // Setup File constructor
    let file_template: _ = v8::FunctionTemplate::new(scope, file_constructor);
    let file_constructor: _ = file_template.get_function(scope).unwrap();
    let file_key: _ = v8::String::new(scope, "File").unwrap();
    global.set(scope, file_key.into(), file_constructor.into());
    eprintln!("✅ [STAGE74] File constructor registered");
    eprintln!("✅ [STAGE74] blob API setup complete");
    Ok(())
}
/// Blob constructor callback
fn blob_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    // Parse blob parts array
    let mut data = Vec::new();
    let mut mime_type = String::new();
    if args.length() > 0 {
        let blob_parts: _ = args.get(0);
        if !blob_parts.is_undefined() && !blob_parts.is_null() {
            if blob_parts.is_array() {
                let parts_array: _ = v8::Local::<v8::Array>::try_from(blob_parts).unwrap();
                let len: _ = parts_array.length();
                for i in 0..len {
                    let part: _ = parts_array.get_index(scope, i).unwrap();
                    if part.is_string() {
                        let part_str: _ = part.to_string(scope).unwrap().to_rust_string_lossy(scope);
                        data.extend_from_slice(part_str.as_bytes());
                    }
                }
            } else if blob_parts.is_string() {
                // Direct string argument
                let part_str: _ = blob_parts.to_string(scope).unwrap().to_rust_string_lossy(scope);
                data.extend_from_slice(part_str.as_bytes());
            }
        }
    }
    // Parse options for MIME type
    if args.length() > 1 {
        let options: _ = args.get(1);
        if !options.is_undefined() && !options.is_null() && options.is_object() {
            let options_obj: _ = v8::Local::<v8::Object>::try_from(options).unwrap();
            let type_key: _ = v8::String::new(scope, "type").unwrap();
            if let Some(type_val) = options_obj.get(scope, type_key.into()) {
                if type_val.is_string() {
                    mime_type = type_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
                }
            }
        }
    }
    // Get the default instance created by V8
    let this: v8::Local<v8::Object> = args.this();
    // Set size property
    let size_key: _ = v8::String::new(scope, "size").unwrap();
    let size_val: _ = v8::Number::new(scope, data.len() as f64);
    this.set(scope, size_key.into(), size_val.into());
    // Set type property
    let type_key: _ = v8::String::new(scope, "type").unwrap();
    let type_val: _ = v8::String::new(scope, &mime_type).unwrap();
    this.set(scope, type_key.into(), type_val.into());
    // Add methods to Blob FIRST
    let text_key: _ = v8::String::new(scope, "text").unwrap();
    let text_template: _ = v8::FunctionTemplate::new(scope, blob_text);
    let text_func: _ = text_template.get_function(scope).unwrap();
    this.set(scope, text_key.into(), text_func.into());
    let slice_key: _ = v8::String::new(scope, "slice").unwrap();
    let slice_template: _ = v8::FunctionTemplate::new(scope, blob_slice);
    let slice_func: _ = slice_template.get_function(scope).unwrap();
    this.set(scope, slice_key.into(), slice_func.into());
    let array_buffer_key: _ = v8::String::new(scope, "arrayBuffer").unwrap();
    let array_buffer_template: _ = v8::FunctionTemplate::new(scope, blob_array_buffer);
    let array_buffer_func: _ = array_buffer_template.get_function(scope).unwrap();
    this.set(scope, array_buffer_key.into(), array_buffer_func.into());
    let stream_key: _ = v8::String::new(scope, "stream").unwrap();
    let stream_template: _ = v8::FunctionTemplate::new(scope, blob_stream);
    let stream_func: _ = stream_template.get_function(scope).unwrap();
    this.set(scope, stream_key.into(), stream_func.into());
    // Store data internally AFTER adding methods
    if !data.is_empty() {
        let data_str: _ = String::from_utf8_lossy(&data);
        // Use a less common property name
        let data_key: _ = v8::String::new(scope, "blobData").unwrap();
        let data_v8: _ = v8::String::new(scope, &data_str).unwrap();
        this.set(scope, data_key.into(), data_v8.into());
    }
    // Don't return anything - use the default instance
}
/// File constructor callback (extends Blob)
fn file_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    // Parse file parts (same as Blob)
    let mut data = Vec::new();
    let mut mime_type = String::new();
    let mut file_name = String::new();
    if args.length() > 0 {
        let blob_parts: _ = args.get(0);
        if !blob_parts.is_undefined() && !blob_parts.is_null() {
            if blob_parts.is_array() {
                let parts_array: _ = v8::Local::<v8::Array>::try_from(blob_parts).unwrap();
                let len: _ = parts_array.length();
                for i in 0..len {
                    let part: _ = parts_array.get_index(scope, i).unwrap();
                    if part.is_string() {
                        let part_str: _ = part.to_string(scope).unwrap().to_rust_string_lossy(scope);
                        data.extend_from_slice(part_str.as_bytes());
                    }
                }
            }
        }
    }
    // Parse file name (first argument after parts)
    if args.length() > 1 {
        let name_arg: _ = args.get(1);
        if name_arg.is_string() {
            file_name = name_arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
        }
    }
    // Parse options for MIME type and other metadata
    if args.length() > 2 {
        let options: _ = args.get(2);
        if !options.is_undefined() && !options.is_null() && options.is_object() {
            let options_obj: _ = v8::Local::<v8::Object>::try_from(options).unwrap();
            let type_key: _ = v8::String::new(scope, "type").unwrap();
            if let Some(type_val) = options_obj.get(scope, type_key.into()) {
                if type_val.is_string() {
                    mime_type = type_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
                }
            }
        }
    }
    // Get the default instance created by V8
    let this: v8::Local<v8::Object> = args.this();
    // Set Blob properties
    let size_key: _ = v8::String::new(scope, "size").unwrap();
    let size_val: _ = v8::Number::new(scope, data.len() as f64);
    this.set(scope, size_key.into(), size_val.into());
    let type_key: _ = v8::String::new(scope, "type").unwrap();
    let type_val: _ = v8::String::new(scope, &mime_type).unwrap();
    this.set(scope, type_key.into(), type_val.into());
    // Set File-specific properties
    let name_key: _ = v8::String::new(scope, "name").unwrap();
    let name_val: _ = v8::String::new(scope, &file_name).unwrap();
    this.set(scope, name_key.into(), name_val.into());
    // Add lastModified property (current timestamp)
    let last_modified_key: _ = v8::String::new(scope, "lastModified").unwrap();
    let last_modified_val: _ = v8::Number::new(scope, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64);
    this.set(scope, last_modified_key.into(), last_modified_val.into());
    // Store data internally
    if !data.is_empty() {
        let data_key: _ = v8::String::new(scope, "blobData").unwrap();
        let data_str: _ = v8::String::new(scope, &String::from_utf8_lossy(&data)).unwrap();
        this.set(scope, data_key.into(), data_str.into());
    }
    // Add methods to File (inherits from Blob)
    let text_key: _ = v8::String::new(scope, "text").unwrap();
    let text_template: _ = v8::FunctionTemplate::new(scope, blob_text);
    let text_func: _ = text_template.get_function(scope).unwrap();
    this.set(scope, text_key.into(), text_func.into());
    let slice_key: _ = v8::String::new(scope, "slice").unwrap();
    let slice_template: _ = v8::FunctionTemplate::new(scope, blob_slice);
    let slice_func: _ = slice_template.get_function(scope).unwrap();
    this.set(scope, slice_key.into(), slice_func.into());
    let array_buffer_key: _ = v8::String::new(scope, "arrayBuffer").unwrap();
    let array_buffer_template: _ = v8::FunctionTemplate::new(scope, blob_array_buffer);
    let array_buffer_func: _ = array_buffer_template.get_function(scope).unwrap();
    this.set(scope, array_buffer_key.into(), array_buffer_func.into());
    let stream_key: _ = v8::String::new(scope, "stream").unwrap();
    let stream_template: _ = v8::FunctionTemplate::new(scope, blob_stream);
    let stream_func: _ = stream_template.get_function(scope).unwrap();
    this.set(scope, stream_key.into(), stream_func.into());
    // Don't return anything - use the default instance
}
/// Blob.arrayBuffer() method
fn blob_array_buffer(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the Blob object (this)
    let this: v8::Local<v8::Object> = args.this();
    // Get the internal data stored in the Blob
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_val: _ = this.get(scope, data_key.into());
    if let Some(data) = data_val {
        if data.is_array_buffer() {
            retval.set(data);
            return;
        }
    }
    // If no data stored, return empty ArrayBuffer
    let array_buffer: _ = v8::ArrayBuffer::new(scope, 0);
    retval.set(array_buffer.into());
}
/// Blob.text() method
fn blob_text(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the Blob object (this)
    let this: v8::Local<v8::Object> = args.this();
    // Get the internal data stored in the Blob
    let data_key: _ = v8::String::new(scope, "blobData").unwrap();
    let data_val: _ = this.get(scope, data_key.into());
    if let Some(data) = data_val {
        if data.is_string() {
            // Return the stored string directly
            let text_str: _ = data.to_string(scope).unwrap();
            retval.set(text_str.into());
            return;
        }
    }
    // Return empty string if no valid data
    let text: _ = v8::String::new(scope, "").unwrap();
    retval.set(text.into());
}
/// Blob.slice() method
fn blob_slice(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the source Blob object (this)
    let this: v8::Local<v8::Object> = args.this();
    // Parse arguments with defaults
    let start: _ = if args.length() > 0 && !args.get(0).is_undefined() {
        args.get(0).to_number(scope).unwrap().value() as i64
    } else {
        0
    };
    let end: _ = if args.length() > 1 && !args.get(1).is_undefined() {
        args.get(1).to_number(scope).unwrap().value() as i64
    } else {
        i64::MAX
    };
    let content_type: _ = if args.length() > 2 && !args.get(2).is_undefined() {
        let type_val: _ = args.get(2);
        if type_val.is_string() {
            type_val.to_string(scope).unwrap().to_rust_string_lossy(scope)
        } else {
            String::new()
        }
    } else {
        // Get the original type from this Blob
        let type_key: _ = v8::String::new(scope, "type").unwrap();
        if let Some(type_val) = this.get(scope, type_key.into()) {
            if type_val.is_string() {
                type_val.to_string(scope).unwrap().to_rust_string_lossy(scope)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };
    // Get the source data
    let data_key: _ = v8::String::new(scope, "blobData").unwrap();
    let data_val: _ = this.get(scope, data_key.into());
    let sliced_string: _ = if let Some(data) = data_val {
        if data.is_string() {
            let data_str: _ = data.to_string(scope).unwrap().to_rust_string_lossy(scope);
            let len: _ = data_str.len() as i64;
            let start_usize: _ = if start < 0 {
                ((len + start) as usize).min(len as usize)
            } else {
                start as usize
            }.min(len as usize);
            let end_usize: _ = if end == i64::MAX {
                len as usize
            } else if end < 0 {
                ((len + end) as usize).min(len as usize)
            } else {
                end as usize
            }.min(len as usize);
            if end_usize > start_usize {
                Some(data_str[start_usize..end_usize].to_string())
            } else {
                Some(String::new())
            }
        } else {
            Some(String::new())
        }
    } else {
        Some(String::new())
    };
    // Create new Blob object with sliced data
    let blob_obj: _ = v8::Object::new(scope);
    let size_key: _ = v8::String::new(scope, "size").unwrap();
    let size_val: _ = v8::Number::new(scope, sliced_string.as_ref().map(|d| d.len()).unwrap_or(0) as f64);
    blob_obj.set(scope, size_key.into(), size_val.into());
    let type_key: _ = v8::String::new(scope, "type").unwrap();
    let type_val: _ = v8::String::new(scope, &content_type).unwrap();
    blob_obj.set(scope, type_key.into(), type_val.into());
    // Store the sliced string data
    if let Some(data) = sliced_string {
        if !data.is_empty() {
            let data_key: _ = v8::String::new(scope, "blobData").unwrap();
            let data_str: _ = v8::String::new(scope, &data).unwrap();
            blob_obj.set(scope, data_key.into(), data_str.into());
        }
    }
    // Add methods to sliced Blob
    let text_key: _ = v8::String::new(scope, "text").unwrap();
    let text_template: _ = v8::FunctionTemplate::new(scope, blob_text);
    let text_func: _ = text_template.get_function(scope).unwrap();
    blob_obj.set(scope, text_key.into(), text_func.into());
    let slice_key: _ = v8::String::new(scope, "slice").unwrap();
    let slice_template: _ = v8::FunctionTemplate::new(scope, blob_slice);
    let slice_func: _ = slice_template.get_function(scope).unwrap();
    blob_obj.set(scope, slice_key.into(), slice_func.into());
    let array_buffer_key: _ = v8::String::new(scope, "arrayBuffer").unwrap();
    let array_buffer_template: _ = v8::FunctionTemplate::new(scope, blob_array_buffer);
    let array_buffer_func: _ = array_buffer_template.get_function(scope).unwrap();
    blob_obj.set(scope, array_buffer_key.into(), array_buffer_func.into());
    let stream_key: _ = v8::String::new(scope, "stream").unwrap();
    let stream_template: _ = v8::FunctionTemplate::new(scope, blob_stream);
    let stream_func: _ = stream_template.get_function(scope).unwrap();
    blob_obj.set(scope, stream_key.into(), stream_func.into());
    retval.set(blob_obj.into());
}
/// Blob.stream() method
fn blob_stream(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Return null for now - ReadableStream not yet implemented
    // In full implementation, would return a ReadableStream
    let stream: _ = v8::null(scope);
    retval.set(stream.into());
}
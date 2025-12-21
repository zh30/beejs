//! Blob API implementation per Web standard
//! Provides binary data container with File API support

use anyhow::Result;
use rusty_v8 as v8;

/// Setup Blob and File APIs in V8 context
pub fn setup_blob_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    eprintln!("🔧 [STAGE74] Setting up blob API...");

    // Write to file to confirm this is called
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/blob_api_init.log")
        .and_then(|mut file| {
            use std::io::Write;
            writeln!(file, "Blob API init called at {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
        });

    let global = context.global(scope);

    // Setup Blob constructor
    let blob_template = v8::FunctionTemplate::new(scope, blob_constructor);
    let blob_constructor = blob_template.get_function(scope).unwrap();
    let blob_key = v8::String::new(scope, "Blob").unwrap();
    global.set(scope, blob_key.into(), blob_constructor.into());
    eprintln!("✅ [STAGE74] Blob constructor registered");

    // Setup File constructor (extends Blob)
    let file_template = v8::FunctionTemplate::new(scope, file_constructor);
    let file_constructor = file_template.get_function(scope).unwrap();
    let file_key = v8::String::new(scope, "File").unwrap();
    global.set(scope, file_key.into(), file_constructor.into());
    eprintln!("✅ [STAGE74] File constructor registered");

    eprintln!("✅ [STAGE74] blob API setup complete");

    Ok(())
}

/// Blob constructor callback
fn blob_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Parse blob parts array
    let mut data = Vec::new();
    let mut mime_type = String::new();

    if args.length() > 0 {
        let blob_parts = args.get(0);

        if !blob_parts.is_undefined() && !blob_parts.is_null() {
            if blob_parts.is_array() {
                let parts_array = v8::Local::<v8::Array>::try_from(blob_parts).unwrap();
                let len = parts_array.length();

                for i in 0..len {
                    let part = parts_array.get_index(scope, i).unwrap();

                    if part.is_string() {
                        let part_str = part.to_string(scope).unwrap().to_rust_string_lossy(scope);
                        data.extend_from_slice(part_str.as_bytes());
                    }
                    // Note: ArrayBuffer support can be added later
                }
            }
        }
    }

    // Parse options for MIME type
    if args.length() > 1 {
        let options = args.get(1);
        if !options.is_undefined() && !options.is_null() && options.is_object() {
            let options_obj = v8::Local::<v8::Object>::try_from(options).unwrap();
            let type_key = v8::String::new(scope, "type").unwrap();
            if let Some(type_val) = options_obj.get(scope, type_key.into()) {
                if type_val.is_string() {
                    mime_type = type_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
                }
            }
        }
    }

    // Create Blob object
    let blob_obj = v8::Object::new(scope);

    // Set size property
    let size_key = v8::String::new(scope, "size").unwrap();
    let size_val = v8::Number::new(scope, data.len() as f64);
    blob_obj.set(scope, size_key.into(), size_val.into());

    // Set type property
    let type_key = v8::String::new(scope, "type").unwrap();
    let type_val = v8::String::new(scope, &mime_type).unwrap();
    blob_obj.set(scope, type_key.into(), type_val.into());

    // Add arrayBuffer method
    let array_buffer_key = v8::String::new(scope, "arrayBuffer").unwrap();
    let array_buffer_template = v8::FunctionTemplate::new(scope, blob_array_buffer);
    let array_buffer_func = array_buffer_template.get_function(scope).unwrap();
    blob_obj.set(scope, array_buffer_key.into(), array_buffer_func.into());

    // Add text method
    let text_key = v8::String::new(scope, "text").unwrap();
    let text_template = v8::FunctionTemplate::new(scope, blob_text);
    let text_func = text_template.get_function(scope).unwrap();
    blob_obj.set(scope, text_key.into(), text_func.into());

    // Add slice method
    let slice_key = v8::String::new(scope, "slice").unwrap();
    let slice_template = v8::FunctionTemplate::new(scope, blob_slice);
    let slice_func = slice_template.get_function(scope).unwrap();
    blob_obj.set(scope, slice_key.into(), slice_func.into());

    // Add stream method
    let stream_key = v8::String::new(scope, "stream").unwrap();
    let stream_template = v8::FunctionTemplate::new(scope, blob_stream);
    let stream_func = stream_template.get_function(scope).unwrap();
    blob_obj.set(scope, stream_key.into(), stream_func.into());

    retval.set(blob_obj.into());
}

/// File constructor callback (extends Blob)
fn file_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Parse file parts (same as Blob)
    let mut data = Vec::new();
    let mut mime_type = String::new();
    let mut file_name = String::new();

    if args.length() > 0 {
        let blob_parts = args.get(0);
        if !blob_parts.is_undefined() && !blob_parts.is_null() {
            if blob_parts.is_array() {
                let parts_array = v8::Local::<v8::Array>::try_from(blob_parts).unwrap();
                let len = parts_array.length();

                for i in 0..len {
                    let part = parts_array.get_index(scope, i).unwrap();
                    if part.is_string() {
                        let part_str = part.to_string(scope).unwrap().to_rust_string_lossy(scope);
                        data.extend_from_slice(part_str.as_bytes());
                    }
                }
            }
        }
    }

    // Parse file name (first argument after parts)
    if args.length() > 1 {
        let name_arg = args.get(1);
        if name_arg.is_string() {
            file_name = name_arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
        }
    }

    // Parse options for MIME type and other metadata
    if args.length() > 2 {
        let options = args.get(2);
        if !options.is_undefined() && !options.is_null() && options.is_object() {
            let options_obj = v8::Local::<v8::Object>::try_from(options).unwrap();
            let type_key = v8::String::new(scope, "type").unwrap();
            if let Some(type_val) = options_obj.get(scope, type_key.into()) {
                if type_val.is_string() {
                    mime_type = type_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
                }
            }
        }
    }

    // Create File object (same as Blob)
    let file_obj = v8::Object::new(scope);

    // Set Blob properties
    let size_key = v8::String::new(scope, "size").unwrap();
    let size_val = v8::Number::new(scope, data.len() as f64);
    file_obj.set(scope, size_key.into(), size_val.into());

    let type_key = v8::String::new(scope, "type").unwrap();
    let type_val = v8::String::new(scope, &mime_type).unwrap();
    file_obj.set(scope, type_key.into(), type_val.into());

    // Set File-specific properties
    let name_key = v8::String::new(scope, "name").unwrap();
    let name_val = v8::String::new(scope, &file_name).unwrap();
    file_obj.set(scope, name_key.into(), name_val.into());

    // Add lastModified property (current timestamp)
    let last_modified_key = v8::String::new(scope, "lastModified").unwrap();
    let last_modified_val = v8::Number::new(scope, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64);
    file_obj.set(scope, last_modified_key.into(), last_modified_val.into());

    // Add Blob methods
    let array_buffer_key = v8::String::new(scope, "arrayBuffer").unwrap();
    let array_buffer_template = v8::FunctionTemplate::new(scope, blob_array_buffer);
    let array_buffer_func = array_buffer_template.get_function(scope).unwrap();
    file_obj.set(scope, array_buffer_key.into(), array_buffer_func.into());

    let text_key = v8::String::new(scope, "text").unwrap();
    let text_template = v8::FunctionTemplate::new(scope, blob_text);
    let text_func = text_template.get_function(scope).unwrap();
    file_obj.set(scope, text_key.into(), text_func.into());

    let slice_key = v8::String::new(scope, "slice").unwrap();
    let slice_template = v8::FunctionTemplate::new(scope, blob_slice);
    let slice_func = slice_template.get_function(scope).unwrap();
    file_obj.set(scope, slice_key.into(), slice_func.into());

    let stream_key = v8::String::new(scope, "stream").unwrap();
    let stream_template = v8::FunctionTemplate::new(scope, blob_stream);
    let stream_func = stream_template.get_function(scope).unwrap();
    file_obj.set(scope, stream_key.into(), stream_func.into());

    retval.set(file_obj.into());
}

/// Blob.arrayBuffer() method
fn blob_array_buffer(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // For simplicity, return empty ArrayBuffer
    // In full implementation, would extract data from internal storage
    let array_buffer = v8::ArrayBuffer::new(scope, 0);
    retval.set(array_buffer.into());
}

/// Blob.text() method
fn blob_text(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // For simplicity, return empty string
    // In full implementation, would decode the internal data
    let text = v8::String::new(scope, "").unwrap();
    retval.set(text.into());
}

/// Blob.slice() method
fn blob_slice(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Parse arguments
    let start = if args.length() > 0 {
        args.get(0).to_number(scope).unwrap().value() as i64
    } else {
        0
    };

    let end = if args.length() > 1 {
        args.get(1).to_number(scope).unwrap().value() as i64
    } else {
        0
    };

    // Create a new Blob with the sliced data
    // For simplicity, return empty Blob
    let blob_obj = v8::Object::new(scope);

    let size_key = v8::String::new(scope, "size").unwrap();
    let size_val = v8::Number::new(scope, 0.0);
    blob_obj.set(scope, size_key.into(), size_val.into());

    let type_key = v8::String::new(scope, "type").unwrap();
    let type_val = v8::String::new(scope, "").unwrap();
    blob_obj.set(scope, type_key.into(), type_val.into());

    retval.set(blob_obj.into());
}

/// Blob.stream() method
fn blob_stream(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // For simplicity, return null
    // In full implementation, would return a ReadableStream
    let stream = v8::null(scope);
    retval.set(stream.into());
}

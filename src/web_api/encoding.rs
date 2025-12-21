//! TextEncoder/TextDecoder API implementation
//! Provides text encoding and decoding functionality per Web standards

use anyhow::Result;
use rusty_v8 as v8;

/// Setup TextEncoder and TextDecoder APIs in V8 context
pub fn setup_encoding_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Setup TextEncoder constructor
    let encoder_template = v8::FunctionTemplate::new(scope, text_encoder_constructor);
    let encoder_constructor = encoder_template.get_function(scope).unwrap();
    let encoder_key = v8::String::new(scope, "TextEncoder").unwrap();
    global.set(scope, encoder_key.into(), encoder_constructor.into());

    // Setup TextDecoder constructor
    let decoder_template = v8::FunctionTemplate::new(scope, text_decoder_constructor);
    let decoder_constructor = decoder_template.get_function(scope).unwrap();
    let decoder_key = v8::String::new(scope, "TextDecoder").unwrap();
    global.set(scope, decoder_key.into(), decoder_constructor.into());

    // Setup atob (base64 decode)
    let atob_template = v8::FunctionTemplate::new(scope, atob_callback);
    let atob_func = atob_template.get_function(scope).unwrap();
    let atob_key = v8::String::new(scope, "atob").unwrap();
    global.set(scope, atob_key.into(), atob_func.into());

    // Setup btoa (base64 encode)
    let btoa_template = v8::FunctionTemplate::new(scope, btoa_callback);
    let btoa_func = btoa_template.get_function(scope).unwrap();
    let btoa_key = v8::String::new(scope, "btoa").unwrap();
    global.set(scope, btoa_key.into(), btoa_func.into());

    Ok(())
}

/// TextEncoder constructor callback
fn text_encoder_constructor(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let encoder_obj = v8::Object::new(scope);

    // Set encoding property (always "utf-8")
    let encoding_key = v8::String::new(scope, "encoding").unwrap();
    let encoding_val = v8::String::new(scope, "utf-8").unwrap();
    encoder_obj.set(scope, encoding_key.into(), encoding_val.into());

    // Add encode method
    let encode_key = v8::String::new(scope, "encode").unwrap();
    let encode_template = v8::FunctionTemplate::new(scope, text_encoder_encode);
    let encode_func = encode_template.get_function(scope).unwrap();
    encoder_obj.set(scope, encode_key.into(), encode_func.into());

    // Add encodeInto method
    let encode_into_key = v8::String::new(scope, "encodeInto").unwrap();
    let encode_into_template = v8::FunctionTemplate::new(scope, text_encoder_encode_into);
    let encode_into_func = encode_into_template.get_function(scope).unwrap();
    encoder_obj.set(scope, encode_into_key.into(), encode_into_func.into());

    retval.set(encoder_obj.into());
}

/// TextEncoder.encode() method
fn text_encoder_encode(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let input = args.get(0);
    let input_str = if input.is_undefined() || input.is_null() {
        String::new()
    } else {
        input.to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default()
    };

    // Convert string to UTF-8 bytes
    let bytes = input_str.as_bytes();

    // Create Uint8Array
    let array_buffer = v8::ArrayBuffer::new(scope, bytes.len());
    let backing_store = array_buffer.get_backing_store();

    // Copy bytes to backing store
    unsafe {
        let data = backing_store.data();
        if !data.is_null() {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                data as *mut u8,
                bytes.len(),
            );
        }
    }

    let uint8_array = v8::Uint8Array::new(scope, array_buffer, 0, bytes.len()).unwrap();
    retval.set(uint8_array.into());
}

/// TextEncoder.encodeInto() method
fn text_encoder_encode_into(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let input = args.get(0);
    let destination = args.get(1);

    // Validate destination is Uint8Array
    if !destination.is_uint8_array() {
        let error = v8::String::new(scope, "encodeInto: destination must be Uint8Array").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let input_str = input.to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let bytes = input_str.as_bytes();
    let dest_array = v8::Local::<v8::Uint8Array>::try_from(destination).unwrap();
    let dest_len = dest_array.byte_length();

    // Calculate how many bytes we can write
    let written = std::cmp::min(bytes.len(), dest_len);

    // Copy bytes (simplified - in real impl we'd use proper backing store access)
    // For now, return the result object
    let result = v8::Object::new(scope);

    let read_key = v8::String::new(scope, "read").unwrap();
    let read_val = v8::Number::new(scope, input_str.chars().count() as f64);
    result.set(scope, read_key.into(), read_val.into());

    let written_key = v8::String::new(scope, "written").unwrap();
    let written_val = v8::Number::new(scope, written as f64);
    result.set(scope, written_key.into(), written_val.into());

    retval.set(result.into());
}

/// TextDecoder constructor callback
fn text_decoder_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get encoding (default "utf-8")
    let encoding = args.get(0);
    let encoding_str = if encoding.is_undefined() || encoding.is_null() {
        "utf-8".to_string()
    } else {
        encoding.to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope).to_lowercase())
            .unwrap_or_else(|| "utf-8".to_string())
    };

    // Validate encoding
    let valid_encodings = ["utf-8", "utf8", "unicode-1-1-utf-8"];
    let normalized_encoding = if valid_encodings.contains(&encoding_str.as_str()) {
        "utf-8"
    } else {
        // For now, only support UTF-8
        let error = v8::String::new(scope, &format!("TextDecoder: unsupported encoding '{}'", encoding_str)).unwrap();
        let error_obj = v8::Exception::range_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    };

    let decoder_obj = v8::Object::new(scope);

    // Set encoding property
    let encoding_key = v8::String::new(scope, "encoding").unwrap();
    let encoding_val = v8::String::new(scope, normalized_encoding).unwrap();
    decoder_obj.set(scope, encoding_key.into(), encoding_val.into());

    // Set fatal property (from options)
    let fatal_key = v8::String::new(scope, "fatal").unwrap();
    let fatal_val = v8::Boolean::new(scope, false);
    decoder_obj.set(scope, fatal_key.into(), fatal_val.into());

    // Set ignoreBOM property
    let ignore_bom_key = v8::String::new(scope, "ignoreBOM").unwrap();
    let ignore_bom_val = v8::Boolean::new(scope, false);
    decoder_obj.set(scope, ignore_bom_key.into(), ignore_bom_val.into());

    // Add decode method
    let decode_key = v8::String::new(scope, "decode").unwrap();
    let decode_template = v8::FunctionTemplate::new(scope, text_decoder_decode);
    let decode_func = decode_template.get_function(scope).unwrap();
    decoder_obj.set(scope, decode_key.into(), decode_func.into());

    retval.set(decoder_obj.into());
}

/// TextDecoder.decode() method
fn text_decoder_decode(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let input = args.get(0);

    // Handle undefined/null input
    if input.is_undefined() || input.is_null() {
        let empty = v8::String::new(scope, "").unwrap();
        retval.set(empty.into());
        return;
    }

    // Get bytes from input (Uint8Array, ArrayBuffer, etc.)
    let bytes: Vec<u8> = if input.is_uint8_array() {
        let array = v8::Local::<v8::Uint8Array>::try_from(input).unwrap();
        let len = array.byte_length();
        let mut buffer = vec![0u8; len];
        array.copy_contents(&mut buffer);
        buffer
    } else if input.is_array_buffer() {
        let array_buffer = v8::Local::<v8::ArrayBuffer>::try_from(input).unwrap();
        let backing_store = array_buffer.get_backing_store();
        let len = backing_store.byte_length();
        let mut buffer = vec![0u8; len];
        unsafe {
            let ptr = backing_store.data();
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(
                    ptr as *const u8,
                    buffer.as_mut_ptr(),
                    len,
                );
            }
        }
        buffer
    } else if input.is_array_buffer_view() {
        let view = v8::Local::<v8::ArrayBufferView>::try_from(input).unwrap();
        let len = view.byte_length();
        let mut buffer = vec![0u8; len];
        view.copy_contents(&mut buffer);
        buffer
    } else {
        let error = v8::String::new(scope, "decode: input must be ArrayBuffer or TypedArray").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    };

    // Decode UTF-8 bytes to string
    match String::from_utf8(bytes) {
        Ok(s) => {
            let result = v8::String::new(scope, &s).unwrap();
            retval.set(result.into());
        }
        Err(e) => {
            // Handle invalid UTF-8 - use replacement character
            let s = String::from_utf8_lossy(e.as_bytes());
            let result = v8::String::new(scope, &s).unwrap();
            retval.set(result.into());
        }
    }
}

/// atob - decode base64 string
fn atob_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let input = args.get(0);

    if input.is_undefined() {
        let error = v8::String::new(scope, "atob: input is required").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let encoded = input.to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // Use base64 decoding
    use base64::{Engine, engine::general_purpose::STANDARD};
    match STANDARD.decode(&encoded) {
        Ok(bytes) => {
            // Convert bytes to string (treating as Latin-1)
            let decoded: String = bytes.iter().map(|&b| b as char).collect();
            let result = v8::String::new(scope, &decoded).unwrap();
            retval.set(result.into());
        }
        Err(_) => {
            let error = v8::String::new(scope, "atob: invalid base64 string").unwrap();
            let error_obj = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
        }
    }
}

/// btoa - encode to base64 string
fn btoa_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let input = args.get(0);

    if input.is_undefined() {
        let error = v8::String::new(scope, "btoa: input is required").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let to_encode = input.to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // Check for non-Latin1 characters
    for c in to_encode.chars() {
        if c as u32 > 255 {
            let error = v8::String::new(scope, "btoa: string contains characters outside Latin-1 range").unwrap();
            let error_obj = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    }

    // Convert to bytes (Latin-1 encoding)
    let bytes: Vec<u8> = to_encode.chars().map(|c| c as u8).collect();

    // Encode to base64
    use base64::{Engine, engine::general_purpose::STANDARD};
    let encoded = STANDARD.encode(&bytes);
    let result = v8::String::new(scope, &encoded).unwrap();
    retval.set(result.into());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_base64_encode_decode() {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let original = "Hello, World!";
        let encoded = STANDARD.encode(original);
        let decoded_bytes = STANDARD.decode(&encoded).unwrap();
        let decoded = String::from_utf8(decoded_bytes).unwrap();

        assert_eq!(original, decoded);
    }
}

#![allow(clippy::all)]
// Node.js Crypto模块实现
/// 支持哈希、HMAC、加密、解密等常用功能
use anyhow::Result;
use base64::{
    engine::general_purpose::{
        STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD as BASE64_URL_SAFE_NO_PAD,
    },
    Engine,
};
use blake3::Hasher;
use openssl::symm::{Cipher, Crypter, Mode};
use ring::digest;
use ring::hmac;
use rusty_v8 as v8;
use sha1::{Digest, Sha1};

/// 根据输出编码返回结果的辅助函数
fn return_output(
    scope: &mut v8::HandleScope,
    output: &[u8],
    output_encoding: &str,
    mut retval: v8::ReturnValue,
) {
    if output_encoding == "utf8" || output_encoding == "utf-8" {
        let result_str = String::from_utf8_lossy(output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "hex" {
        let result_str = hex::encode(output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "base64" {
        let result_str = BASE64_STANDARD.encode(output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "base64url" {
        let result_str = BASE64_URL_SAFE_NO_PAD.encode(output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "latin1" || output_encoding == "binary" {
        let result_str = bytes_to_latin1_string(output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else {
        let buffer_obj: _ = v8::ArrayBuffer::new(scope, output.len());
        if output.len() > 0 {
            let store = buffer_obj.get_backing_store();
            let ptr = store.data() as *mut u8;
            if !ptr.is_null() {
                let slice = unsafe { std::slice::from_raw_parts_mut(ptr, output.len()) };
                slice.copy_from_slice(output);
            }
        }
        if let Some(uint8_array) = v8::Uint8Array::new(scope, buffer_obj, 0, output.len()) {
            retval.set(uint8_array.into());
        }
    }
}

fn bytes_to_latin1_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| byte as char).collect()
}

fn decode_base64url(text: &str) -> std::result::Result<Vec<u8>, base64::DecodeError> {
    BASE64_URL_SAFE_NO_PAD.decode(text.trim_end_matches('=').as_bytes())
}

fn normalize_hash_algorithm(algorithm: &str) -> String {
    algorithm.to_ascii_lowercase().replace('-', "")
}

fn is_supported_hash_algorithm(algorithm: &str) -> bool {
    matches!(
        algorithm,
        "sha256" | "sha384" | "sha1" | "sha512" | "md5" | "blake3"
    )
}

fn array_buffer_bytes(
    value: v8::Local<v8::ArrayBuffer>,
    offset: usize,
    length: usize,
) -> Result<Vec<u8>, String> {
    if length == 0 {
        return Ok(Vec::new());
    }

    let backing_store = value.get_backing_store();
    let ptr = backing_store.data() as *const u8;
    if ptr.is_null() {
        return Err("buffer data is unavailable".to_string());
    }

    Ok(unsafe { std::slice::from_raw_parts(ptr.add(offset), length).to_vec() })
}

fn encode_digest_bytes(bytes: &[u8], encoding: &str) -> String {
    let encoding = encoding.to_ascii_lowercase();
    match encoding.as_str() {
        "hex" => hex::encode(bytes),
        "base64" => BASE64_STANDARD.encode(bytes),
        "base64url" => BASE64_URL_SAFE_NO_PAD.encode(bytes),
        "latin1" | "binary" => bytes_to_latin1_string(bytes),
        _ => hex::encode(bytes),
    }
}

fn optional_string_arg(
    scope: &mut v8::HandleScope,
    args: &v8::FunctionCallbackArguments,
    index: i32,
) -> Option<String> {
    let value = args.get(index);
    if args.length() <= index || value.is_undefined() || value.is_null() {
        None
    } else {
        value
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
    }
}

fn object_string_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    obj.get(scope, key.into())
        .and_then(|value| value.to_string(scope))
        .map(|value| value.to_rust_string_lossy(scope))
}

fn digest_encoding_arg(
    scope: &mut v8::HandleScope,
    args: &v8::FunctionCallbackArguments,
) -> Option<String> {
    optional_string_arg(scope, args, 0)
}

fn hmac_key_encoding_arg(
    scope: &mut v8::HandleScope,
    args: &v8::FunctionCallbackArguments,
) -> Option<String> {
    let value = args.get(2);
    if args.length() <= 2 || value.is_undefined() || value.is_null() || !value.is_object() {
        return None;
    }

    let options = v8::Local::<v8::Object>::try_from(value).ok()?;
    object_string_property(scope, options, "encoding")
}

fn bytes_from_update_value(
    scope: &mut v8::HandleScope,
    value: v8::Local<v8::Value>,
    string_encoding: Option<&str>,
) -> Result<Vec<u8>, String> {
    if value.is_string() {
        let text = value
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        let encoding = string_encoding.unwrap_or("utf8").to_ascii_lowercase();
        return match encoding.as_str() {
            "hex" => hex::decode(&text).map_err(|error| format!("invalid hex data: {}", error)),
            "base64" => BASE64_STANDARD
                .decode(text.as_bytes())
                .map_err(|error| format!("invalid base64 data: {}", error)),
            "base64url" => decode_base64url(&text)
                .map_err(|error| format!("invalid base64url data: {}", error)),
            "latin1" | "binary" => Ok(text.chars().map(|ch| ch as u8).collect()),
            _ => Ok(text.into_bytes()),
        };
    }

    if value.is_array_buffer() {
        let buffer = v8::Local::<v8::ArrayBuffer>::try_from(value)
            .map_err(|_| "data must be an ArrayBuffer".to_string())?;
        return array_buffer_bytes(buffer, 0, buffer.byte_length());
    }

    if value.is_typed_array() {
        let typed_array = v8::Local::<v8::TypedArray>::try_from(value)
            .map_err(|_| "data must be a TypedArray".to_string())?;
        let buffer = typed_array
            .buffer(scope)
            .ok_or_else(|| "typed array buffer is unavailable".to_string())?;
        return array_buffer_bytes(buffer, typed_array.byte_offset(), typed_array.byte_length());
    }

    Err("data must be a string, ArrayBuffer, or TypedArray".to_string())
}

fn append_data_chunk(
    scope: &mut v8::HandleScope,
    this: v8::Local<v8::Object>,
    bytes: &[u8],
) -> Result<(), String> {
    let data_key =
        v8::String::new(scope, "_data").ok_or_else(|| "allocation failed".to_string())?;
    let data_array = this
        .get(scope, data_key.into())
        .ok_or_else(|| "data buffer is unavailable".to_string())?;
    let arr = v8::Local::<v8::Array>::try_from(data_array)
        .map_err(|_| "data buffer is invalid".to_string())?;
    let length = arr.length();
    let chunk = v8::String::new(scope, &hex::encode(bytes))
        .ok_or_else(|| "allocation failed".to_string())?;
    arr.set_index(scope, length, chunk.into());
    Ok(())
}

fn data_chunks_bytes(
    scope: &mut v8::HandleScope,
    this: v8::Local<v8::Object>,
) -> Result<Vec<u8>, String> {
    let data_key =
        v8::String::new(scope, "_data").ok_or_else(|| "allocation failed".to_string())?;
    let data_array = this
        .get(scope, data_key.into())
        .ok_or_else(|| "data buffer is unavailable".to_string())?;
    let arr = v8::Local::<v8::Array>::try_from(data_array)
        .map_err(|_| "data buffer is invalid".to_string())?;
    let mut combined_data = Vec::new();
    for i in 0..arr.length() {
        if let Some(data_str) = arr.get_index(scope, i).and_then(|v| v.to_string(scope)) {
            let chunk_hex = data_str.to_rust_string_lossy(scope);
            let mut chunk = hex::decode(&chunk_hex)
                .map_err(|error| format!("invalid internal update chunk: {}", error))?;
            combined_data.append(&mut chunk);
        }
    }
    Ok(combined_data)
}

fn hmac_key_bytes(
    scope: &mut v8::HandleScope,
    this: v8::Local<v8::Object>,
) -> Result<Vec<u8>, String> {
    if let Some(key_hex) = object_string_property(scope, this, "_key_bytes") {
        return hex::decode(&key_hex)
            .map_err(|error| format!("invalid internal HMAC key: {}", error));
    }

    Ok(object_string_property(scope, this, "_key")
        .unwrap_or_default()
        .into_bytes())
}

fn object_bool_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
) -> bool {
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    obj.get(scope, key.into())
        .map(|value| value.boolean_value(scope))
        .unwrap_or(false)
}

fn set_object_bool_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
    value: bool,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let value = v8::Boolean::new(scope, value);
        obj.set(scope, key.into(), value.into());
    }
}

fn object_usize_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
) -> usize {
    let Some(key) = v8::String::new(scope, name) else {
        return 0;
    };
    obj.get(scope, key.into())
        .and_then(|value| value.number_value(scope))
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map(|value| value as usize)
        .unwrap_or(0)
}

fn set_object_usize_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
    value: usize,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let value = v8::Number::new(scope, value as f64);
        obj.set(scope, key.into(), value.into());
    }
}

fn object_array_buffer_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
) -> Vec<u8> {
    let Some(key) = v8::String::new(scope, name) else {
        return Vec::new();
    };
    let Some(value) = obj.get(scope, key.into()) else {
        return Vec::new();
    };
    let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) else {
        return Vec::new();
    };
    let store = buffer.get_backing_store();
    let len = store.byte_length();
    if len == 0 {
        return Vec::new();
    }
    let ptr = store.data() as *const u8;
    if ptr.is_null() {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
    }
}

fn set_object_array_buffer_property(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
    bytes: &[u8],
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };

    let buffer = v8::ArrayBuffer::new(scope, bytes.len());
    if !bytes.is_empty() {
        let store = buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, bytes.len()) };
            slice.copy_from_slice(bytes);
        }
    }
    obj.set(scope, key.into(), buffer.into());
}

fn cipher_auto_padding(scope: &mut v8::HandleScope, obj: v8::Local<v8::Object>) -> bool {
    let Some(key) = v8::String::new(scope, "_autoPadding") else {
        return true;
    };

    obj.get(scope, key.into())
        .map(|value| {
            if value.is_undefined() || value.is_null() {
                true
            } else {
                value.boolean_value(scope)
            }
        })
        .unwrap_or(true)
}

fn throw_crypto_error(scope: &mut v8::HandleScope, message: &str) {
    let message = v8::String::new(scope, message).unwrap();
    let error = v8::Exception::error(scope, message);
    scope.throw_exception(error);
}

fn throw_crypto_error_with_code(scope: &mut v8::HandleScope, message: &str, code: &str) {
    let message = v8::String::new(scope, message).unwrap();
    let error = v8::Exception::error(scope, message);
    if let Some(error_object) = error.to_object(scope) {
        if let (Some(code_key), Some(code_value)) =
            (v8::String::new(scope, "code"), v8::String::new(scope, code))
        {
            error_object.set(scope, code_key.into(), code_value.into());
        }
    }
    scope.throw_exception(error);
}

fn throw_digest_already_called(scope: &mut v8::HandleScope) {
    let message = v8::String::new(scope, "Digest already called").unwrap();
    let error = v8::Exception::error(scope, message);
    scope.throw_exception(error);
}

fn compute_hmac_with_hash<F>(
    key_bytes: &[u8],
    data_bytes: &[u8],
    block_size: usize,
    hash: F,
) -> Vec<u8>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let mut key_block = key_bytes.to_vec();
    if key_block.len() > block_size {
        key_block = hash(&key_block);
    }
    key_block.resize(block_size, 0);

    let mut inner_input = Vec::with_capacity(block_size + data_bytes.len());
    inner_input.extend(key_block.iter().map(|b| b ^ 0x36));
    inner_input.extend(data_bytes);
    let inner_hash = hash(&inner_input);

    let mut outer_input = Vec::with_capacity(block_size + inner_hash.len());
    outer_input.extend(key_block.iter().map(|b| b ^ 0x5c));
    outer_input.extend(&inner_hash);
    hash(&outer_input)
}

fn sha1_digest_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// 设置Crypto API
pub fn setup_crypto_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global: _ = context.global(scope);
    let crypto_key: _ = v8::String::new(scope, "crypto").unwrap();
    let crypto_obj: _ = global
        .get(scope, crypto_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .unwrap_or_else(|| v8::Object::new(scope));
    // createHash
    let create_hash_func: _ = v8::FunctionTemplate::new(scope, create_hash_callback);
    let create_hash_instance: _ = create_hash_func.get_function(scope).unwrap();
    let create_hash_key: _ = v8::String::new(scope, "createHash").unwrap();
    crypto_obj.set(scope, create_hash_key.into(), create_hash_instance.into());
    // createHmac
    let create_hmac_func: _ = v8::FunctionTemplate::new(scope, create_hmac_callback);
    let create_hmac_instance: _ = create_hmac_func.get_function(scope).unwrap();
    let create_hmac_key: _ = v8::String::new(scope, "createHmac").unwrap();
    crypto_obj.set(scope, create_hmac_key.into(), create_hmac_instance.into());
    // randomBytes
    let random_bytes_func: _ = v8::FunctionTemplate::new(scope, random_bytes_callback);
    let random_bytes_instance: _ = random_bytes_func.get_function(scope).unwrap();
    let random_bytes_key: _ = v8::String::new(scope, "randomBytes").unwrap();
    crypto_obj.set(scope, random_bytes_key.into(), random_bytes_instance.into());
    // randomBytesSync
    let random_bytes_sync_func: _ = v8::FunctionTemplate::new(scope, random_bytes_sync_callback);
    let random_bytes_sync_instance: _ = random_bytes_sync_func.get_function(scope).unwrap();
    let random_bytes_sync_key: _ = v8::String::new(scope, "randomBytesSync").unwrap();
    crypto_obj.set(
        scope,
        random_bytes_sync_key.into(),
        random_bytes_sync_instance.into(),
    );
    // createCipher - v0.3.61: 添加对称加密支持
    let create_cipher_func: _ = v8::FunctionTemplate::new(scope, create_cipher_callback);
    let create_cipher_instance: _ = create_cipher_func.get_function(scope).unwrap();
    let create_cipher_key: _ = v8::String::new(scope, "createCipher").unwrap();
    crypto_obj.set(
        scope,
        create_cipher_key.into(),
        create_cipher_instance.into(),
    );
    // createDecipher - v0.3.61: 添加对称解密支持
    let create_decipher_func: _ = v8::FunctionTemplate::new(scope, create_decipher_callback);
    let create_decipher_instance: _ = create_decipher_func.get_function(scope).unwrap();
    let create_decipher_key: _ = v8::String::new(scope, "createDecipher").unwrap();
    crypto_obj.set(
        scope,
        create_decipher_key.into(),
        create_decipher_instance.into(),
    );
    // createCipheriv - v0.3.63: 添加显式 IV 加密支持
    let create_cipheriv_func: _ = v8::FunctionTemplate::new(scope, create_cipheriv_callback);
    let create_cipheriv_instance: _ = create_cipheriv_func.get_function(scope).unwrap();
    let create_cipheriv_key: _ = v8::String::new(scope, "createCipheriv").unwrap();
    crypto_obj.set(
        scope,
        create_cipheriv_key.into(),
        create_cipheriv_instance.into(),
    );
    // createDecipheriv - v0.3.63: 添加显式 IV 解密支持
    let create_decipheriv_func: _ = v8::FunctionTemplate::new(scope, create_decipheriv_callback);
    let create_decipheriv_instance: _ = create_decipheriv_func.get_function(scope).unwrap();
    let create_decipheriv_key: _ = v8::String::new(scope, "createDecipheriv").unwrap();
    crypto_obj.set(
        scope,
        create_decipheriv_key.into(),
        create_decipheriv_instance.into(),
    );
    // 设置crypto对象到全局
    global.set(scope, crypto_key.into(), crypto_obj.into());
    Ok(())
}

fn set_hash_methods(scope: &mut v8::HandleScope, hash_obj: v8::Local<v8::Object>) {
    let update_func: _ = v8::FunctionTemplate::new(scope, hash_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    hash_obj.set(scope, update_key.into(), update_instance.into());

    let digest_func: _ = v8::FunctionTemplate::new(scope, hash_digest_callback);
    let digest_instance: _ = digest_func.get_function(scope).unwrap();
    let digest_key: _ = v8::String::new(scope, "digest").unwrap();
    hash_obj.set(scope, digest_key.into(), digest_instance.into());

    let copy_func: _ = v8::FunctionTemplate::new(scope, hash_copy_callback);
    let copy_instance: _ = copy_func.get_function(scope).unwrap();
    let copy_key: _ = v8::String::new(scope, "copy").unwrap();
    hash_obj.set(scope, copy_key.into(), copy_instance.into());
}

fn create_hash_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let algorithm: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let algorithm = normalize_hash_algorithm(&algorithm);
    if !is_supported_hash_algorithm(&algorithm) {
        let error_msg =
            v8::String::new(scope, &format!("Unsupported hash algorithm: {}", algorithm)).unwrap();
        let error = v8::Exception::type_error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }

    // 创建hash对象
    let hash_obj: _ = v8::Object::new(scope);
    set_hash_methods(scope, hash_obj);
    // 保存算法到对象内部
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    hash_obj.set(scope, algo_key.into(), algo_val.into());
    // 保存数据缓冲区
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_val: _ = v8::Array::new(scope, 0);
    hash_obj.set(scope, data_key.into(), data_val.into());
    set_object_bool_property(scope, hash_obj, "_digest_called", false);
    retval.set(hash_obj.into());
}
fn hash_update_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    if object_bool_property(scope, this, "_digest_called") {
        throw_digest_already_called(scope);
        return;
    }

    let string_encoding = optional_string_arg(scope, &args, 1);
    let bytes = match bytes_from_update_value(scope, args.get(0), string_encoding.as_deref()) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = v8::String::new(scope, &error).unwrap();
            let error = v8::Exception::type_error(scope, message);
            scope.throw_exception(error);
            return;
        }
    };
    if let Err(error) = append_data_chunk(scope, this, &bytes) {
        let message = v8::String::new(scope, &error).unwrap();
        let error = v8::Exception::type_error(scope, message);
        scope.throw_exception(error);
        return;
    }
    retval.set(this.into());
}

fn hash_copy_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    if object_bool_property(scope, this, "_digest_called") {
        throw_digest_already_called(scope);
        return;
    }

    let algorithm = object_string_property(scope, this, "_algorithm").unwrap_or_default();
    let copied_hash = v8::Object::new(scope);
    set_hash_methods(scope, copied_hash);

    let algo_key = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val = v8::String::new(scope, &algorithm).unwrap();
    copied_hash.set(scope, algo_key.into(), algo_val.into());

    let data_key = v8::String::new(scope, "_data").unwrap();
    let copied_data = v8::Array::new(scope, 0);
    if let Some(data_value) = this.get(scope, data_key.into()) {
        if let Ok(source_data) = v8::Local::<v8::Array>::try_from(data_value) {
            for i in 0..source_data.length() {
                if let Some(value) = source_data.get_index(scope, i) {
                    copied_data.set_index(scope, i, value);
                }
            }
        }
    }
    copied_hash.set(scope, data_key.into(), copied_data.into());
    set_object_bool_property(scope, copied_hash, "_digest_called", false);
    retval.set(copied_hash.into());
}

fn hash_digest_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    if object_bool_property(scope, this, "_digest_called") {
        throw_digest_already_called(scope);
        return;
    }

    let encoding = digest_encoding_arg(scope, &args);
    // 获取算法
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm: _ = this
        .get(scope, algo_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();
    let combined_data = match data_chunks_bytes(scope, this) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = v8::String::new(scope, &error).unwrap();
            let error = v8::Exception::type_error(scope, message);
            scope.throw_exception(error);
            return;
        }
    };
    let digest_bytes: Vec<u8> = match algorithm.as_str() {
        "sha256" => {
            let digest: _ = digest::digest(&digest::SHA256, &combined_data);
            digest.as_ref().to_vec()
        }
        "sha384" => {
            let digest: _ = digest::digest(&digest::SHA384, &combined_data);
            digest.as_ref().to_vec()
        }
        "sha512" => {
            let digest: _ = digest::digest(&digest::SHA512, &combined_data);
            digest.as_ref().to_vec()
        }
        "sha1" => {
            // 使用 sha1 crate 正确计算 SHA1 哈希
            let mut hasher = Sha1::new();
            hasher.update(&combined_data);
            let digest = hasher.finalize();
            digest.to_vec()
        }
        "blake3" => {
            let mut hasher = Hasher::new();
            hasher.update(&combined_data);
            let hash = hasher.finalize();
            hash.as_bytes().to_vec()
        }
        "md5" => {
            let digest: _ = md5::compute(&combined_data);
            digest.0.to_vec()
        }
        _ => {
            // 抛出错误：不支持的算法
            let error_msg =
                v8::String::new(scope, &format!("Unsupported hash algorithm: {}", algorithm))
                    .unwrap();
            let error = v8::Exception::type_error(scope, error_msg);
            scope.throw_exception(error);
            return;
        }
    };

    set_object_bool_property(scope, this, "_digest_called", true);

    if let Some(encoding) = encoding {
        let digest_result = encode_digest_bytes(&digest_bytes, &encoding);
        let result_str: _ = v8::String::new(scope, &digest_result).unwrap();
        retval.set(result_str.into());
    } else {
        return_output(scope, &digest_bytes, "buffer", retval);
    }
}
fn create_hmac_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let algorithm: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let algorithm = normalize_hash_algorithm(&algorithm);
    let key: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let key_encoding = hmac_key_encoding_arg(scope, &args);
    let key_bytes = if args.length() <= 1 || args.get(1).is_undefined() {
        Vec::new()
    } else {
        match bytes_from_update_value(scope, args.get(1), key_encoding.as_deref()) {
            Ok(bytes) => bytes,
            Err(error) => {
                let message = v8::String::new(scope, &error).unwrap();
                let error = v8::Exception::type_error(scope, message);
                scope.throw_exception(error);
                return;
            }
        }
    };
    // 验证算法是否支持
    if !is_supported_hash_algorithm(&algorithm) {
        let error_msg =
            v8::String::new(scope, &format!("Unsupported HMAC algorithm: {}", algorithm)).unwrap();
        let error = v8::Exception::type_error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }
    // 创建hmac对象
    let hmac_obj: _ = v8::Object::new(scope);
    // update方法
    let update_func: _ = v8::FunctionTemplate::new(scope, hmac_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    hmac_obj.set(scope, update_key.into(), update_instance.into());
    // digest方法
    let digest_func: _ = v8::FunctionTemplate::new(scope, hmac_digest_callback);
    let digest_instance: _ = digest_func.get_function(scope).unwrap();
    let digest_key: _ = v8::String::new(scope, "digest").unwrap();
    hmac_obj.set(scope, digest_key.into(), digest_instance.into());
    // 保存数据
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    hmac_obj.set(scope, algo_key.into(), algo_val.into());
    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key_val: _ = v8::String::new(scope, &key).unwrap();
    hmac_obj.set(scope, key_key.into(), key_val.into());
    let key_bytes_key: _ = v8::String::new(scope, "_key_bytes").unwrap();
    let key_bytes_val: _ = v8::String::new(scope, &hex::encode(&key_bytes)).unwrap();
    hmac_obj.set(scope, key_bytes_key.into(), key_bytes_val.into());
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_val: _ = v8::Array::new(scope, 0);
    hmac_obj.set(scope, data_key.into(), data_val.into());
    set_object_bool_property(scope, hmac_obj, "_digest_called", false);
    retval.set(hmac_obj.into());
}
fn hmac_update_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    if object_bool_property(scope, this, "_digest_called") {
        throw_digest_already_called(scope);
        return;
    }

    let string_encoding = optional_string_arg(scope, &args, 1);
    let bytes = match bytes_from_update_value(scope, args.get(0), string_encoding.as_deref()) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = v8::String::new(scope, &error).unwrap();
            let error = v8::Exception::type_error(scope, message);
            scope.throw_exception(error);
            return;
        }
    };
    if let Err(error) = append_data_chunk(scope, this, &bytes) {
        let message = v8::String::new(scope, &error).unwrap();
        let error = v8::Exception::type_error(scope, message);
        scope.throw_exception(error);
        return;
    }
    retval.set(this.into());
}
fn hmac_digest_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let encoding = digest_encoding_arg(scope, &args);
    if object_bool_property(scope, this, "_digest_called") {
        if encoding.is_some() {
            let result_str: _ = v8::String::new(scope, "").unwrap();
            retval.set(result_str.into());
        } else {
            return_output(scope, &[], "buffer", retval);
        }
        return;
    }

    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm: _ = this
        .get(scope, algo_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();
    let key_bytes = match hmac_key_bytes(scope, this) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = v8::String::new(scope, &error).unwrap();
            let error = v8::Exception::type_error(scope, message);
            scope.throw_exception(error);
            return;
        }
    };
    let combined_data = match data_chunks_bytes(scope, this) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = v8::String::new(scope, &error).unwrap();
            let error = v8::Exception::type_error(scope, message);
            scope.throw_exception(error);
            return;
        }
    };
    let digest_bytes: Vec<u8> = match algorithm.as_str() {
        "sha256" => {
            let signing_key: _ = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
            let hmac_result: _ = hmac::sign(&signing_key, &combined_data);
            hmac_result.as_ref().to_vec()
        }
        "sha1" => compute_hmac_with_hash(&key_bytes, &combined_data, 64, sha1_digest_bytes),
        "sha384" => compute_hmac_with_hash(&key_bytes, &combined_data, 128, |data| {
            digest::digest(&digest::SHA384, data).as_ref().to_vec()
        }),
        "sha512" => compute_hmac_with_hash(&key_bytes, &combined_data, 128, |data| {
            digest::digest(&digest::SHA512, data).as_ref().to_vec()
        }),
        "md5" => compute_hmac_with_hash(&key_bytes, &combined_data, 64, |data| {
            md5::compute(data).0.to_vec()
        }),
        "blake3" => {
            // 使用 blake3 crate 实现 HMAC-BLAKE3
            // blake3::keyed_hash 需要 32 字节密钥，需要标准化密钥长度
            let mut key_32 = [0u8; 32];
            if key_bytes.len() > 32 {
                // 如果密钥过长，先哈希
                let mut hasher = blake3::Hasher::new();
                hasher.update(&key_bytes);
                let hashed = hasher.finalize();
                key_32.copy_from_slice(hashed.as_bytes());
            } else {
                // 如果密钥过短或正好 32 字节，直接复制或填充
                key_32[..key_bytes.len()].copy_from_slice(&key_bytes);
            }
            // 使用 blake3::keyed_hash 进行带密钥的哈希
            let result = blake3::keyed_hash(&key_32, &combined_data);
            result.as_bytes().to_vec()
        }
        _ => {
            // 抛出错误：不支持的算法
            let error_msg =
                v8::String::new(scope, &format!("Unsupported HMAC algorithm: {}", algorithm))
                    .unwrap();
            let error = v8::Exception::type_error(scope, error_msg);
            scope.throw_exception(error);
            return;
        }
    };

    set_object_bool_property(scope, this, "_digest_called", true);

    if let Some(encoding) = encoding {
        let digest_result = encode_digest_bytes(&digest_bytes, &encoding);
        let result_str: _ = v8::String::new(scope, &digest_result).unwrap();
        retval.set(result_str.into());
    } else {
        return_output(scope, &digest_bytes, "buffer", retval);
    }
}
fn random_bytes_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let size: _ = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let size = size.max(0);

    // Generate random bytes (only if size > 0)
    let random_data = if size > 0 {
        let mut data = vec![0u8; size];
        if let Some(seed) = crate::permissions::get_deterministic_seed() {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(size as u64, std::sync::atomic::Ordering::SeqCst);
            let mut state = seed.wrapping_add(count);
            for byte in data.iter_mut() {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                *byte = (state >> 33) as u8;
            }
        } else {
            let rand: _ = ring::rand::SystemRandom::new();
            ring::rand::SecureRandom::fill(&rand, &mut data).unwrap_or(());
        }
        data
    } else {
        vec![]
    };

    // Create ArrayBuffer and copy random data
    let buffer_obj: _ = v8::ArrayBuffer::new(scope, size);

    // Copy random data to ArrayBuffer's backing store (only if size > 0)
    if size > 0 {
        let store = buffer_obj.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
            slice.copy_from_slice(&random_data);
        }
    }

    // Check if callback is provided as second argument
    let callback = args.get(1);
    if callback.is_function() {
        // Callback API: randomBytes(size, callback)
        if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
            // Create null error (no error occurred)
            let null_val = v8::null(scope).into();
            let undefined_val = v8::undefined(scope).into();

            // Create the buffer as Uint8Array for better compatibility
            if let Some(uint8_array) = v8::Uint8Array::new(scope, buffer_obj, 0, size) {
                let cb_args: &[v8::Local<v8::Value>] = &[null_val, uint8_array.into()];
                cb_func.call(scope, undefined_val, cb_args);
            }
        }
    }

    // Return Uint8Array for consistency with Node.js
    if let Some(uint8_array) = v8::Uint8Array::new(scope, buffer_obj, 0, size) {
        retval.set(uint8_array.into());
    }
}

fn random_bytes_sync_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let size: _ = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .value() as usize;

    let size = size.max(0);

    // Generate random bytes (only if size > 0)
    let random_data = if size > 0 {
        let mut data = vec![0u8; size];
        if let Some(seed) = crate::permissions::get_deterministic_seed() {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(size as u64, std::sync::atomic::Ordering::SeqCst);
            let mut state = seed.wrapping_add(count);
            for byte in data.iter_mut() {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                *byte = (state >> 33) as u8;
            }
        } else {
            let rand: _ = ring::rand::SystemRandom::new();
            ring::rand::SecureRandom::fill(&rand, &mut data).unwrap_or(());
        }
        data
    } else {
        vec![]
    };

    // Create ArrayBuffer and copy random data
    let buffer_obj: _ = v8::ArrayBuffer::new(scope, size);

    // Copy random data to ArrayBuffer's backing store (only if size > 0)
    if size > 0 {
        let store = buffer_obj.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
            slice.copy_from_slice(&random_data);
        }
    }

    // Return Uint8Array for consistency with Node.js
    if let Some(uint8_array) = v8::Uint8Array::new(scope, buffer_obj, 0, size) {
        retval.set(uint8_array.into());
    }
}

/// 获取 OpenSSL Cipher 对象
fn normalize_cipher_algorithm(algorithm: &str) -> String {
    match algorithm.to_ascii_lowercase().as_str() {
        "aes128" => "aes-128-cbc".to_string(),
        "aes192" => "aes-192-cbc".to_string(),
        "aes256" => "aes-256-cbc".to_string(),
        normalized => normalized.to_string(),
    }
}

fn get_cipher(algorithm: &str) -> Option<Cipher> {
    let algorithm = normalize_cipher_algorithm(algorithm);
    match algorithm.as_str() {
        "aes-128-cbc" | "aes128-cbc" => Some(Cipher::aes_128_cbc()),
        "aes-192-cbc" | "aes192-cbc" => Some(Cipher::aes_192_cbc()),
        "aes-256-cbc" | "aes256-cbc" => Some(Cipher::aes_256_cbc()),
        "aes-128-cfb" | "aes128-cfb" => Some(Cipher::aes_128_cfb128()),
        "aes-192-cfb" | "aes192-cfb" => Some(Cipher::aes_192_cfb128()),
        "aes-256-cfb" | "aes256-cfb" => Some(Cipher::aes_256_cfb128()),
        "aes-128-ecb" | "aes128-ecb" => Some(Cipher::aes_128_ecb()),
        "aes-192-ecb" | "aes192-ecb" => Some(Cipher::aes_192_ecb()),
        "aes-256-ecb" | "aes256-ecb" => Some(Cipher::aes_256_ecb()),
        "aes-128-ofb" | "aes128-ofb" => Some(Cipher::aes_128_ofb()),
        "aes-192-ofb" | "aes192-ofb" => Some(Cipher::aes_192_ofb()),
        "aes-256-ofb" | "aes256-ofb" => Some(Cipher::aes_256_ofb()),
        "aes-128-ctr" | "aes128-ctr" => Some(Cipher::aes_128_ctr()),
        "aes-192-ctr" | "aes192-ctr" => Some(Cipher::aes_192_ctr()),
        "aes-256-ctr" | "aes256-ctr" => Some(Cipher::aes_256_ctr()),
        "aes-128-gcm" | "aes128-gcm" => Some(Cipher::aes_128_gcm()),
        "aes-192-gcm" | "aes192-gcm" => Some(Cipher::aes_192_gcm()),
        "aes-256-gcm" | "aes256-gcm" => Some(Cipher::aes_256_gcm()),
        _ => None,
    }
}

fn expected_cipher_key_len(algorithm: &str) -> Option<usize> {
    let algorithm = normalize_cipher_algorithm(algorithm);
    match algorithm.as_str() {
        value if value.starts_with("aes-128-") || value.starts_with("aes128-") => Some(16),
        value if value.starts_with("aes-192-") || value.starts_with("aes192-") => Some(24),
        value if value.starts_with("aes-256-") || value.starts_with("aes256-") => Some(32),
        _ => None,
    }
}

fn expected_cipher_iv_len(algorithm: &str) -> Option<usize> {
    let algorithm = normalize_cipher_algorithm(algorithm);
    match algorithm.as_str() {
        value
            if value.contains("-cbc")
                || value.contains("-ctr")
                || value.contains("-cfb")
                || value.contains("-ofb") =>
        {
            Some(16)
        }
        value if value.contains("-gcm") => Some(12),
        value if value.contains("-ecb") => Some(0),
        _ => None,
    }
}

fn is_ctr_cipher(algorithm: &str) -> bool {
    normalize_cipher_algorithm(algorithm).contains("ctr")
}

fn is_gcm_cipher(algorithm: &str) -> bool {
    normalize_cipher_algorithm(algorithm).contains("-gcm")
}

/// 创建加密/解密器辅助函数
fn create_crypter(
    algorithm: &str,
    key: &[u8],
    iv: Option<&[u8]>,
    is_encrypt: bool,
    auto_padding: bool,
) -> Result<Crypter, String> {
    let cipher = get_cipher(algorithm).ok_or_else(|| "unsupported algorithm".to_string())?;
    let mode = if is_encrypt {
        Mode::Encrypt
    } else {
        Mode::Decrypt
    };

    let mut crypter = Crypter::new(cipher, mode, key, iv)
        .map_err(|error| format!("failed to create cipher: {}", error))?;
    crypter.pad(auto_padding);
    Ok(crypter)
}

fn apply_gcm_aad(
    scope: &mut v8::HandleScope,
    crypter: &mut Crypter,
    this: v8::Local<v8::Object>,
) -> bool {
    let aad = object_array_buffer_property(scope, this, "_aad");
    if aad.is_empty() {
        return true;
    }

    match crypter.aad_update(&aad) {
        Ok(()) => true,
        Err(error) => {
            throw_crypto_error(scope, &format!("cipher AAD update failed: {}", error));
            false
        }
    }
}

fn cipher_set_aad_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    if object_bool_property(scope, this, "_finalCalled") {
        throw_crypto_error(scope, "cipher setAAD after final");
        return;
    }

    let encoding = optional_string_arg(scope, &args, 1);
    let bytes = match bytes_from_update_value(scope, args.get(0), encoding.as_deref()) {
        Ok(bytes) => bytes,
        Err(error) => {
            throw_crypto_error(scope, &format!("setAAD: {}", error));
            return;
        }
    };

    let mut aad = object_array_buffer_property(scope, this, "_aad");
    aad.extend_from_slice(&bytes);
    set_object_array_buffer_property(scope, this, "_aad", &aad);
    retval.set(this.into());
}

fn cipher_get_auth_tag_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    retval: v8::ReturnValue,
) {
    let this = args.this();
    if !object_bool_property(scope, this, "_finalCalled") {
        throw_crypto_error(scope, "auth tag is not available before final");
        return;
    }

    let tag = object_array_buffer_property(scope, this, "_authTag");
    if tag.is_empty() {
        throw_crypto_error(scope, "auth tag is not available");
        return;
    }

    return_output(scope, &tag, "buffer", retval);
}

fn cipher_set_auth_tag_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    if object_bool_property(scope, this, "_finalCalled") {
        throw_crypto_error(scope, "cipher setAuthTag after final");
        return;
    }

    let tag = match bytes_from_update_value(scope, args.get(0), None) {
        Ok(tag) => tag,
        Err(error) => {
            throw_crypto_error(scope, &format!("setAuthTag: {}", error));
            return;
        }
    };

    if tag.is_empty() || tag.len() > 16 {
        throw_crypto_error(scope, "invalid auth tag length");
        return;
    }

    set_object_array_buffer_property(scope, this, "_authTag", &tag);
    retval.set(this.into());
}

fn set_gcm_methods(scope: &mut v8::HandleScope, obj: v8::Local<v8::Object>, is_encrypt: bool) {
    let set_aad_func: _ = v8::FunctionTemplate::new(scope, cipher_set_aad_callback);
    let set_aad_instance: _ = set_aad_func.get_function(scope).unwrap();
    let set_aad_key: _ = v8::String::new(scope, "setAAD").unwrap();
    obj.set(scope, set_aad_key.into(), set_aad_instance.into());

    if is_encrypt {
        let get_auth_tag_func: _ = v8::FunctionTemplate::new(scope, cipher_get_auth_tag_callback);
        let get_auth_tag_instance: _ = get_auth_tag_func.get_function(scope).unwrap();
        let get_auth_tag_key: _ = v8::String::new(scope, "getAuthTag").unwrap();
        obj.set(scope, get_auth_tag_key.into(), get_auth_tag_instance.into());
    } else {
        let set_auth_tag_func: _ = v8::FunctionTemplate::new(scope, cipher_set_auth_tag_callback);
        let set_auth_tag_instance: _ = set_auth_tag_func.get_function(scope).unwrap();
        let set_auth_tag_key: _ = v8::String::new(scope, "setAuthTag").unwrap();
        obj.set(scope, set_auth_tag_key.into(), set_auth_tag_instance.into());
    }
}

/// createCipher 回调函数 - v0.3.61
/// 创建对称加密 Cipher 对象
fn create_cipher_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let algorithm: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let algorithm = normalize_cipher_algorithm(&algorithm);

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        throw_crypto_error_with_code(scope, "unsupported algorithm", "ERR_CRYPTO_UNKNOWN_CIPHER");
        return;
    }

    // 支持密码作为 Buffer 或字符串
    let password_data: Vec<u8> = if args.get(1).is_string() {
        let password: String = args
            .get(1)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        password.into_bytes()
    } else if args.get(1).is_array_buffer_view() {
        // 处理 Buffer 输入
        if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(1)) {
            let len = uint8_arr.byte_length();
            let mut data = vec![0u8; len];
            uint8_arr.copy_contents(&mut data);
            data
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 可选的 IV 参数
    let iv_data: Option<Vec<u8>> = if !args.get(2).is_undefined() {
        Some(if args.get(2).is_string() {
            let iv_str: String = args
                .get(2)
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
            iv_str.into_bytes()
        } else if args.get(2).is_array_buffer_view() {
            if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(2)) {
                let len = uint8_arr.byte_length();
                let mut data = vec![0u8; len];
                uint8_arr.copy_contents(&mut data);
                data
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        })
    } else {
        None
    };

    // 派生密钥和 IV
    let (key, derived_iv) = derive_key_and_iv(&algorithm, &password_data);

    // 如果没有提供 IV，使用派生的 IV（确保加密和解密使用相同的 IV）
    let iv_data = iv_data.or(Some(derived_iv));

    // 创建 cipher 对象
    let cipher_obj: _ = v8::Object::new(scope);

    // 保存加密状态
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    cipher_obj.set(scope, algo_key.into(), algo_val.into());

    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key_buffer = v8::ArrayBuffer::new(scope, key.len());
    // 复制密钥数据到缓冲区
    if key.len() > 0 {
        let store = key_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, key.len()) };
            slice.copy_from_slice(&key);
        }
    }
    cipher_obj.set(scope, key_key.into(), key_buffer.into());

    let iv_key: _ = v8::String::new(scope, "_iv").unwrap();
    // 只在有 IV 数据时才创建和设置 IV 缓冲区
    if let Some(iv_data_ref) = iv_data.as_ref() {
        let iv_len = iv_data_ref.len();
        let iv_buffer = v8::ArrayBuffer::new(scope, iv_len);
        if iv_len > 0 {
            let store = iv_buffer.get_backing_store();
            let ptr = store.data() as *mut u8;
            if !ptr.is_null() {
                let slice = unsafe { std::slice::from_raw_parts_mut(ptr, iv_len) };
                slice.copy_from_slice(iv_data_ref);
            }
        }
        cipher_obj.set(scope, iv_key.into(), iv_buffer.into());
    } else {
        // 没有 IV，设置空值
        let undefined_val = v8::undefined(scope);
        cipher_obj.set(scope, iv_key.into(), undefined_val.into());
    }

    let encrypt_key: _ = v8::String::new(scope, "_encrypt").unwrap();
    let encrypt_val = v8::Boolean::new(scope, true);
    cipher_obj.set(scope, encrypt_key.into(), encrypt_val.into());
    set_object_bool_property(scope, cipher_obj, "_autoPadding", true);
    set_object_bool_property(scope, cipher_obj, "_finalCalled", false);

    // 内部状态 - 待处理的数据
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    cipher_obj.set(scope, pending_key.into(), pending_val.into());
    set_object_usize_property(scope, cipher_obj, "_processedBytes", 0);
    set_object_usize_property(scope, cipher_obj, "_emittedBytes", 0);

    // update 方法
    let update_func: _ = v8::FunctionTemplate::new(scope, cipher_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    cipher_obj.set(scope, update_key.into(), update_instance.into());

    // final 方法
    let final_func: _ = v8::FunctionTemplate::new(scope, cipher_final_callback);
    let final_instance: _ = final_func.get_function(scope).unwrap();
    let final_key: _ = v8::String::new(scope, "final").unwrap();
    cipher_obj.set(scope, final_key.into(), final_instance.into());

    // setAutoPadding 方法 (noop - padding is always enabled)
    let set_auto_padding_func: _ = v8::FunctionTemplate::new(scope, set_auto_padding_callback);
    let set_auto_padding_instance = set_auto_padding_func.get_function(scope).unwrap();
    let set_auto_padding_key: _ = v8::String::new(scope, "setAutoPadding").unwrap();
    cipher_obj.set(
        scope,
        set_auto_padding_key.into(),
        set_auto_padding_instance.into(),
    );

    if is_gcm_cipher(&algorithm) {
        set_gcm_methods(scope, cipher_obj, true);
    }

    retval.set(cipher_obj.into());
}

/// createDecipher 回调函数 - v0.3.61
/// 创建对称解密 Decipher 对象
fn create_decipher_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let algorithm: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let algorithm = normalize_cipher_algorithm(&algorithm);

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        throw_crypto_error_with_code(scope, "unsupported algorithm", "ERR_CRYPTO_UNKNOWN_CIPHER");
        return;
    }

    // 支持密码作为 Buffer 或字符串
    let password_data: Vec<u8> = if args.get(1).is_string() {
        let password: String = args
            .get(1)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        password.into_bytes()
    } else if args.get(1).is_array_buffer_view() {
        if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(1)) {
            let len = uint8_arr.byte_length();
            let mut data = vec![0u8; len];
            uint8_arr.copy_contents(&mut data);
            data
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 可选的 IV 参数
    let iv_data: Option<Vec<u8>> = if !args.get(2).is_undefined() {
        Some(if args.get(2).is_string() {
            let iv_str: String = args
                .get(2)
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
            iv_str.into_bytes()
        } else if args.get(2).is_array_buffer_view() {
            if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(2)) {
                let len = uint8_arr.byte_length();
                let mut data = vec![0u8; len];
                uint8_arr.copy_contents(&mut data);
                data
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        })
    } else {
        None
    };

    // 派生密钥和 IV
    let (key, derived_iv) = derive_key_and_iv(&algorithm, &password_data);

    // 如果没有提供 IV，使用派生的 IV（确保加密和解密使用相同的 IV）
    let iv_data = iv_data.or(Some(derived_iv));

    // 创建 decipher 对象
    let decipher_obj: _ = v8::Object::new(scope);

    // 保存解密状态
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    decipher_obj.set(scope, algo_key.into(), algo_val.into());

    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key_buffer = v8::ArrayBuffer::new(scope, key.len());
    // 复制密钥数据到缓冲区
    if key.len() > 0 {
        let store = key_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, key.len()) };
            slice.copy_from_slice(&key);
        }
    }
    decipher_obj.set(scope, key_key.into(), key_buffer.into());

    let iv_key: _ = v8::String::new(scope, "_iv").unwrap();
    // 只在有 IV 数据时才创建和设置 IV 缓冲区
    if let Some(iv_data_ref) = iv_data.as_ref() {
        let iv_len = iv_data_ref.len();
        let iv_buffer = v8::ArrayBuffer::new(scope, iv_len);
        if iv_len > 0 {
            let store = iv_buffer.get_backing_store();
            let ptr = store.data() as *mut u8;
            if !ptr.is_null() {
                let slice = unsafe { std::slice::from_raw_parts_mut(ptr, iv_len) };
                slice.copy_from_slice(iv_data_ref);
            }
        }
        decipher_obj.set(scope, iv_key.into(), iv_buffer.into());
    } else {
        // 没有 IV，设置空值
        let undefined_val = v8::undefined(scope);
        decipher_obj.set(scope, iv_key.into(), undefined_val.into());
    }

    let encrypt_key: _ = v8::String::new(scope, "_encrypt").unwrap();
    let encrypt_val = v8::Boolean::new(scope, false);
    decipher_obj.set(scope, encrypt_key.into(), encrypt_val.into());
    set_object_bool_property(scope, decipher_obj, "_autoPadding", true);
    set_object_bool_property(scope, decipher_obj, "_finalCalled", false);

    // 内部状态
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    decipher_obj.set(scope, pending_key.into(), pending_val.into());
    set_object_usize_property(scope, decipher_obj, "_processedBytes", 0);
    set_object_usize_property(scope, decipher_obj, "_emittedBytes", 0);

    // update 方法
    let update_func: _ = v8::FunctionTemplate::new(scope, cipher_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    decipher_obj.set(scope, update_key.into(), update_instance.into());

    // final 方法
    let final_func: _ = v8::FunctionTemplate::new(scope, cipher_final_callback);
    let final_instance: _ = final_func.get_function(scope).unwrap();
    let final_key: _ = v8::String::new(scope, "final").unwrap();
    decipher_obj.set(scope, final_key.into(), final_instance.into());

    // setAutoPadding 方法 (noop - padding is always enabled)
    let set_auto_padding_func: _ = v8::FunctionTemplate::new(scope, set_auto_padding_callback);
    let set_auto_padding_instance = set_auto_padding_func.get_function(scope).unwrap();
    let set_auto_padding_key: _ = v8::String::new(scope, "setAutoPadding").unwrap();
    decipher_obj.set(
        scope,
        set_auto_padding_key.into(),
        set_auto_padding_instance.into(),
    );

    if is_gcm_cipher(&algorithm) {
        set_gcm_methods(scope, decipher_obj, false);
    }

    retval.set(decipher_obj.into());
}

/// createCipheriv 回调函数 - v0.3.63
/// 创建带显式 IV 的对称加密 Cipher 对象
/// 参数: algorithm, key, iv[, options]
fn create_cipheriv_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let algorithm: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let algorithm = normalize_cipher_algorithm(&algorithm);

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        throw_crypto_error_with_code(scope, "unsupported algorithm", "ERR_CRYPTO_UNKNOWN_CIPHER");
        return;
    }

    // 获取 key (必需) - 支持 hex 字符串或 Buffer
    let key_data: Vec<u8> = if args.get(1).is_string() {
        let key_str: String = args
            .get(1)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        // 检查是否是 hex 字符串 (偶数长度，只包含 hex 字符)
        if key_str.len() % 2 == 0 && key_str.chars().all(|c| c.is_ascii_hexdigit()) {
            // 作为 hex 字符串解码
            hex::decode(key_str).unwrap_or_default()
        } else {
            // 作为原始字符串
            key_str.into_bytes()
        }
    } else if args.get(1).is_array_buffer_view() {
        if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(1)) {
            let len = uint8_arr.byte_length();
            let mut data = vec![0u8; len];
            uint8_arr.copy_contents(&mut data);
            data
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 获取 iv (必需) - 支持 hex 字符串或 Buffer
    let iv_data: Vec<u8> = if !args.get(2).is_undefined() && !args.get(2).is_null() {
        if args.get(2).is_string() {
            let iv_str: String = args
                .get(2)
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
            // 检查是否是 hex 字符串 (偶数长度，只包含 hex 字符)
            if iv_str.len() % 2 == 0 && iv_str.chars().all(|c| c.is_ascii_hexdigit()) {
                // 作为 hex 字符串解码
                hex::decode(iv_str).unwrap_or_default()
            } else {
                // 作为原始字符串
                iv_str.into_bytes()
            }
        } else if args.get(2).is_array_buffer_view() {
            if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(2)) {
                let len = uint8_arr.byte_length();
                let mut data = vec![0u8; len];
                uint8_arr.copy_contents(&mut data);
                data
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if let Some(expected_key_len) = expected_cipher_key_len(&algorithm) {
        if key_data.len() != expected_key_len {
            throw_crypto_error_with_code(scope, "invalid key length", "ERR_CRYPTO_INVALID_KEYLEN");
            return;
        }
    }

    if let Some(expected_iv_len) = expected_cipher_iv_len(&algorithm) {
        if iv_data.len() != expected_iv_len {
            throw_crypto_error_with_code(scope, "invalid iv length", "ERR_CRYPTO_INVALID_IV");
            return;
        }
    }

    // 创建 cipher 对象
    let cipher_obj: _ = v8::Object::new(scope);

    // 保存加密状态
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    cipher_obj.set(scope, algo_key.into(), algo_val.into());

    // 直接保存 key (不派生)
    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key_buffer = v8::ArrayBuffer::new(scope, key_data.len());
    if key_data.len() > 0 {
        let store = key_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, key_data.len()) };
            slice.copy_from_slice(&key_data);
        }
    }
    cipher_obj.set(scope, key_key.into(), key_buffer.into());

    // 保存 IV
    let iv_key: _ = v8::String::new(scope, "_iv").unwrap();
    let iv_buffer = v8::ArrayBuffer::new(scope, iv_data.len());
    if iv_data.len() > 0 {
        let store = iv_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, iv_data.len()) };
            slice.copy_from_slice(&iv_data);
        }
    }
    cipher_obj.set(scope, iv_key.into(), iv_buffer.into());

    let encrypt_key: _ = v8::String::new(scope, "_encrypt").unwrap();
    let encrypt_val = v8::Boolean::new(scope, true);
    cipher_obj.set(scope, encrypt_key.into(), encrypt_val.into());
    set_object_bool_property(scope, cipher_obj, "_autoPadding", true);
    set_object_bool_property(scope, cipher_obj, "_finalCalled", false);

    // 内部状态 - 待处理的数据
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    cipher_obj.set(scope, pending_key.into(), pending_val.into());
    set_object_usize_property(scope, cipher_obj, "_processedBytes", 0);
    set_object_usize_property(scope, cipher_obj, "_emittedBytes", 0);

    // update 方法
    let update_func: _ = v8::FunctionTemplate::new(scope, cipher_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    cipher_obj.set(scope, update_key.into(), update_instance.into());

    // final 方法
    let final_func: _ = v8::FunctionTemplate::new(scope, cipher_final_callback);
    let final_instance: _ = final_func.get_function(scope).unwrap();
    let final_key: _ = v8::String::new(scope, "final").unwrap();
    cipher_obj.set(scope, final_key.into(), final_instance.into());

    // setAutoPadding 方法 (noop - padding is always enabled)
    let set_auto_padding_func: _ = v8::FunctionTemplate::new(scope, set_auto_padding_callback);
    let set_auto_padding_instance = set_auto_padding_func.get_function(scope).unwrap();
    let set_auto_padding_key: _ = v8::String::new(scope, "setAutoPadding").unwrap();
    cipher_obj.set(
        scope,
        set_auto_padding_key.into(),
        set_auto_padding_instance.into(),
    );

    if is_gcm_cipher(&algorithm) {
        set_gcm_methods(scope, cipher_obj, true);
    }

    retval.set(cipher_obj.into());
}

/// createDecipheriv 回调函数 - v0.3.63
/// 创建带显式 IV 的对称解密 Decipher 对象
/// 参数: algorithm, key, iv[, options]
fn create_decipheriv_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let algorithm: String = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let algorithm = normalize_cipher_algorithm(&algorithm);

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        throw_crypto_error_with_code(scope, "unsupported algorithm", "ERR_CRYPTO_UNKNOWN_CIPHER");
        return;
    }

    // 获取 key (必需) - 支持 hex 字符串或 Buffer
    let key_data: Vec<u8> = if args.get(1).is_string() {
        let key_str: String = args
            .get(1)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        // 检查是否是 hex 字符串 (偶数长度，只包含 hex 字符)
        if key_str.len() % 2 == 0 && key_str.chars().all(|c| c.is_ascii_hexdigit()) {
            // 作为 hex 字符串解码
            hex::decode(key_str).unwrap_or_default()
        } else {
            // 作为原始字符串
            key_str.into_bytes()
        }
    } else if args.get(1).is_array_buffer_view() {
        if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(1)) {
            let len = uint8_arr.byte_length();
            let mut data = vec![0u8; len];
            uint8_arr.copy_contents(&mut data);
            data
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 获取 iv (必需) - 支持 hex 字符串或 Buffer
    let iv_data: Vec<u8> = if !args.get(2).is_undefined() && !args.get(2).is_null() {
        if args.get(2).is_string() {
            let iv_str: String = args
                .get(2)
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
            // 检查是否是 hex 字符串 (偶数长度，只包含 hex 字符)
            if iv_str.len() % 2 == 0 && iv_str.chars().all(|c| c.is_ascii_hexdigit()) {
                // 作为 hex 字符串解码
                hex::decode(iv_str).unwrap_or_default()
            } else {
                // 作为原始字符串
                iv_str.into_bytes()
            }
        } else if args.get(2).is_array_buffer_view() {
            if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(args.get(2)) {
                let len = uint8_arr.byte_length();
                let mut data = vec![0u8; len];
                uint8_arr.copy_contents(&mut data);
                data
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if let Some(expected_key_len) = expected_cipher_key_len(&algorithm) {
        if key_data.len() != expected_key_len {
            throw_crypto_error_with_code(scope, "invalid key length", "ERR_CRYPTO_INVALID_KEYLEN");
            return;
        }
    }

    if let Some(expected_iv_len) = expected_cipher_iv_len(&algorithm) {
        if iv_data.len() != expected_iv_len {
            throw_crypto_error_with_code(scope, "invalid iv length", "ERR_CRYPTO_INVALID_IV");
            return;
        }
    }

    // 创建 decipher 对象
    let decipher_obj: _ = v8::Object::new(scope);

    // 保存解密状态
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    decipher_obj.set(scope, algo_key.into(), algo_val.into());

    // 直接保存 key (不派生)
    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key_buffer = v8::ArrayBuffer::new(scope, key_data.len());
    if key_data.len() > 0 {
        let store = key_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, key_data.len()) };
            slice.copy_from_slice(&key_data);
        }
    }
    decipher_obj.set(scope, key_key.into(), key_buffer.into());

    // 保存 IV
    let iv_key: _ = v8::String::new(scope, "_iv").unwrap();
    let iv_buffer = v8::ArrayBuffer::new(scope, iv_data.len());
    if iv_data.len() > 0 {
        let store = iv_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, iv_data.len()) };
            slice.copy_from_slice(&iv_data);
        }
    }
    decipher_obj.set(scope, iv_key.into(), iv_buffer.into());

    let encrypt_key: _ = v8::String::new(scope, "_encrypt").unwrap();
    let encrypt_val = v8::Boolean::new(scope, false);
    decipher_obj.set(scope, encrypt_key.into(), encrypt_val.into());
    set_object_bool_property(scope, decipher_obj, "_autoPadding", true);
    set_object_bool_property(scope, decipher_obj, "_finalCalled", false);

    // 内部状态
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    decipher_obj.set(scope, pending_key.into(), pending_val.into());
    set_object_usize_property(scope, decipher_obj, "_processedBytes", 0);
    set_object_usize_property(scope, decipher_obj, "_emittedBytes", 0);

    // 保存已解密的输出数据（用于解密时累积）
    let decrypted_output_key: _ = v8::String::new(scope, "_decryptedOutput").unwrap();
    let decrypted_output_val: _ = v8::ArrayBuffer::new(scope, 0);
    decipher_obj.set(
        scope,
        decrypted_output_key.into(),
        decrypted_output_val.into(),
    );

    // update 方法
    let update_func: _ = v8::FunctionTemplate::new(scope, cipher_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    decipher_obj.set(scope, update_key.into(), update_instance.into());

    // final 方法
    let final_func: _ = v8::FunctionTemplate::new(scope, cipher_final_callback);
    let final_instance: _ = final_func.get_function(scope).unwrap();
    let final_key: _ = v8::String::new(scope, "final").unwrap();
    decipher_obj.set(scope, final_key.into(), final_instance.into());

    // setAutoPadding 方法 (noop - padding is always enabled)
    let set_auto_padding_func: _ = v8::FunctionTemplate::new(scope, set_auto_padding_callback);
    let set_auto_padding_instance = set_auto_padding_func.get_function(scope).unwrap();
    let set_auto_padding_key: _ = v8::String::new(scope, "setAutoPadding").unwrap();
    decipher_obj.set(
        scope,
        set_auto_padding_key.into(),
        set_auto_padding_instance.into(),
    );

    if is_gcm_cipher(&algorithm) {
        set_gcm_methods(scope, decipher_obj, false);
    }

    retval.set(decipher_obj.into());
}

/// 根据算法和密码派生密钥
fn derive_key_and_iv(algorithm: &str, password: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // 根据算法确定密钥长度
    let key_len = if algorithm.to_lowercase().contains("128") {
        16
    } else if algorithm.to_lowercase().contains("192") {
        24
    } else {
        32 // 默认 256 位
    };

    // 对于 AES-CBC，需要 IV (16字节)
    let iv_len = 16;
    let total_len = key_len + iv_len;

    if password.len() >= total_len {
        // 如果密码足够长，前面的作为 key，后面的作为 IV
        let key = password[..key_len].to_vec();
        let iv = password[key_len..total_len].to_vec();
        return (key, iv);
    }

    // 使用 EVP_BytesToKey 风格派生密钥和 IV
    let mut derived = vec![0u8; total_len];
    let mut hash = blake3::Hasher::new();

    // 简单的密码派生：循环哈希（最多 3 次迭代以避免无限循环）
    for counter in 1i32..=3 {
        let counter_bytes = counter.to_le_bytes();
        let mut input = password.to_vec();
        input.extend_from_slice(&counter_bytes);

        hash.update(&input);
        let output = hash.finalize();
        let output_bytes = output.as_bytes();

        for (i, byte) in output_bytes.iter().enumerate() {
            if i < total_len {
                derived[i] = *byte;
            }
        }
        // 重置哈希器用于下一次迭代
        hash = blake3::Hasher::new();
    }

    let key = derived[..key_len].to_vec();
    let iv = derived[key_len..].to_vec();
    (key, iv)
}

#[allow(dead_code)]
fn derive_key(algorithm: &str, password: &[u8]) -> Vec<u8> {
    derive_key_and_iv(algorithm, password).0
}

/// cipher.update() 回调函数
fn cipher_update_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    retval: v8::ReturnValue,
) {
    let this: _ = args.this();

    if object_bool_property(scope, this, "_finalCalled") {
        throw_crypto_error(scope, "cipher update after final");
        return;
    }

    let input_encoding: String = match args.get(1).to_string(scope) {
        Some(s) => s.to_rust_string_lossy(scope),
        None => "utf8".to_string(),
    };

    // 获取输出编码 (默认 'buffer')
    let output_encoding: String = match args.get(2).to_string(scope) {
        Some(s) => s.to_rust_string_lossy(scope),
        None => "buffer".to_string(),
    };

    // 获取输入数据
    let input_data: Vec<u8> = if input_encoding == "buffer" || input_encoding == "binary" {
        let arg0 = args.get(0);

        // Try Uint8Array first
        if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(arg0) {
            let len = uint8_arr.byte_length();
            let mut data = vec![0u8; len];
            uint8_arr.copy_contents(&mut data);
            data
        } else if arg0.is_array_buffer() {
            // Handle as ArrayBuffer
            if let Ok(buf) = v8::Local::<v8::ArrayBuffer>::try_from(arg0) {
                let store = buf.get_backing_store();
                let len = store.byte_length();
                if len > 0 {
                    let ptr = store.data() as *const u8;
                    if !ptr.is_null() {
                        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else if arg0.is_object() {
            // Handle beejs Buffer object (has 'buffer' property)
            let buffer_key = v8::String::new(scope, "buffer").unwrap();
            let obj = arg0.to_object(scope);
            if let Some(obj) = obj {
                let buffer_prop = obj.get(scope, buffer_key.into());

                if let Some(buf_val) = buffer_prop {
                    if buf_val.is_array_buffer() {
                        if let Ok(buf) = v8::Local::<v8::ArrayBuffer>::try_from(buf_val) {
                            let store = buf.get_backing_store();
                            let len = store.byte_length();
                            if len > 0 {
                                let ptr = store.data() as *const u8;
                                if !ptr.is_null() {
                                    unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
                                } else {
                                    Vec::new()
                                }
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else if input_encoding == "hex" {
        // hex 编码输入
        let input: String = args
            .get(0)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        let decoded = hex::decode(&input);
        decoded.unwrap_or_default()
    } else {
        // utf8 或其他字符串编码
        let input: String = args
            .get(0)
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();
        input.into_bytes()
    };

    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm: String = this
        .get(scope, algo_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();

    if is_ctr_cipher(&algorithm) {
        let key = object_array_buffer_property(scope, this, "_key");
        let iv = object_array_buffer_property(scope, this, "_iv");
        let is_encrypt = object_bool_property(scope, this, "_encrypt");
        let processed = object_usize_property(scope, this, "_processedBytes");

        let mut crypter = match create_crypter(&algorithm, &key, Some(&iv), is_encrypt, false) {
            Ok(crypter) => crypter,
            Err(error) => {
                throw_crypto_error(scope, &error);
                return;
            }
        };

        if processed > 0 {
            let advance = vec![0u8; processed];
            let mut discard = vec![0u8; processed + 64];
            if let Err(error) = crypter.update(&advance, &mut discard) {
                throw_crypto_error(scope, &format!("cipher update failed: {}", error));
                return;
            }
        }

        let mut output = vec![0u8; input_data.len() + 64];
        let count = match crypter.update(&input_data, &mut output) {
            Ok(count) => count,
            Err(error) => {
                throw_crypto_error(scope, &format!("cipher update failed: {}", error));
                return;
            }
        };
        output.truncate(count);
        set_object_usize_property(
            scope,
            this,
            "_processedBytes",
            processed.saturating_add(input_data.len()),
        );
        return_output(scope, &output, &output_encoding, retval);
        return;
    }

    // 获取之前累积的输入数据
    let pending_data_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_data_val = this.get(scope, pending_data_key.into());
    let mut pending_data: Vec<u8> =
        if let Ok(buf) = v8::Local::<v8::ArrayBuffer>::try_from(pending_data_val.unwrap()) {
            let store = buf.get_backing_store();
            let len = store.byte_length();
            if len > 0 {
                let ptr = store.data() as *const u8;
                if !ptr.is_null() {
                    unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

    // 追加新数据到累积缓冲区
    pending_data.extend_from_slice(&input_data);

    // Beejs 当前把 Cipher/Decipher 的真实 OpenSSL 流处理集中在 final()。
    // 这样 split update 拼接时不会在每次 update 重置 CBC/CTR 状态。
    let new_pending_buffer = v8::ArrayBuffer::new(scope, pending_data.len());
    if !pending_data.is_empty() {
        let store = new_pending_buffer.get_backing_store();
        let ptr = store.data() as *mut u8;
        if !ptr.is_null() {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, pending_data.len()) };
            slice.copy_from_slice(&pending_data);
        }
    }
    this.set(scope, pending_data_key.into(), new_pending_buffer.into());

    let key = object_array_buffer_property(scope, this, "_key");
    let iv = object_array_buffer_property(scope, this, "_iv");
    let iv_ref = if iv.is_empty() {
        None
    } else {
        Some(iv.as_slice())
    };
    let is_encrypt = object_bool_property(scope, this, "_encrypt");
    let auto_padding = cipher_auto_padding(scope, this);
    let is_gcm = is_gcm_cipher(&algorithm);

    let mut output = Vec::new();
    if !pending_data.is_empty() {
        let mut crypter = match create_crypter(
            &algorithm,
            &key,
            iv_ref,
            is_encrypt,
            auto_padding && !is_gcm,
        ) {
            Ok(crypter) => crypter,
            Err(error) => {
                throw_crypto_error(scope, &error);
                return;
            }
        };
        if is_gcm && !apply_gcm_aad(scope, &mut crypter, this) {
            return;
        }
        let mut replay_output = vec![0u8; pending_data.len() + 64];
        let count = match crypter.update(&pending_data, &mut replay_output) {
            Ok(count) => count,
            Err(error) => {
                throw_crypto_error(scope, &format!("cipher update failed: {}", error));
                return;
            }
        };
        replay_output.truncate(count);

        let emitted = object_usize_property(scope, this, "_emittedBytes");
        if emitted <= replay_output.len() {
            output.extend_from_slice(&replay_output[emitted..]);
            set_object_usize_property(scope, this, "_emittedBytes", replay_output.len());
        }
    }

    return_output(scope, &output, &output_encoding, retval);
}

/// cipher.final() 回调函数 - 处理最后的数据块并添加/移除填充
fn cipher_final_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();

    if object_bool_property(scope, this, "_finalCalled") {
        throw_crypto_error(scope, "cipher final already called");
        return;
    }
    set_object_bool_property(scope, this, "_finalCalled", true);

    // 获取输出编码参数 (可选)
    let output_encoding: String = match args.get(0).to_string(scope) {
        Some(s) => s.to_rust_string_lossy(scope),
        None => "buffer".to_string(),
    };

    // 获取加密状态
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm: String = this
        .get(scope, algo_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();

    let encrypt_key: _ = v8::String::new(scope, "_encrypt").unwrap();
    let is_encrypt: bool = this
        .get(scope, encrypt_key.into())
        .and_then(|v| v.to_boolean(scope).boolean_value(scope).into())
        .unwrap_or(false);
    let auto_padding = cipher_auto_padding(scope, this);
    let is_gcm = is_gcm_cipher(&algorithm);

    // 获取密钥
    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key_val = this.get(scope, key_key.into()).unwrap();
    let key: Vec<u8> = if let Ok(buf) = v8::Local::<v8::ArrayBuffer>::try_from(key_val) {
        let store = buf.get_backing_store();
        let len = store.byte_length();
        if len > 0 {
            let ptr = store.data() as *const u8;
            if !ptr.is_null() {
                unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 获取 IV
    let iv_key: _ = v8::String::new(scope, "_iv").unwrap();
    let iv_val = this.get(scope, iv_key.into());
    let iv: Option<Vec<u8>> = match iv_val {
        None => None,
        Some(v) if v.is_undefined() || v.is_null() => None,
        Some(v) => {
            // 尝试作为 ArrayBuffer 提取
            if let Ok(iv_buf) = v8::Local::<v8::ArrayBuffer>::try_from(v) {
                let store = iv_buf.get_backing_store();
                let len = store.byte_length();
                if len > 0 {
                    let ptr = store.data() as *const u8;
                    if !ptr.is_null() {
                        Some(unsafe { std::slice::from_raw_parts(ptr, len).to_vec() })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    // 获取之前累积的 pending 数据
    let pending_data_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_data_val = this.get(scope, pending_data_key.into());
    let pending_data: Vec<u8> =
        if let Ok(buf) = v8::Local::<v8::ArrayBuffer>::try_from(pending_data_val.unwrap()) {
            let store = buf.get_backing_store();
            let len = store.byte_length();
            if len > 0 {
                let ptr = store.data() as *const u8;
                if !ptr.is_null() {
                    unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

    // 清空 pending data
    let new_pending_buffer = v8::ArrayBuffer::new(scope, 0);
    this.set(scope, pending_data_key.into(), new_pending_buffer.into());

    // final() 只处理剩余的数据（可能为空或只有一个不完整的块）
    // 对于 CBC 模式，final() 会添加填充（加密）或移除填充（解密）
    let mut output: Vec<u8> = Vec::new();

    if !pending_data.is_empty() || is_encrypt || is_gcm {
        // 加密时，即使没有剩余数据也需要调用 finalize 来添加填充
        let mut crypter = match create_crypter(
            &algorithm,
            &key,
            iv.as_deref(),
            is_encrypt,
            auto_padding && !is_gcm,
        ) {
            Ok(crypter) => crypter,
            Err(error) => {
                throw_crypto_error(scope, &error);
                return;
            }
        };

        if is_gcm {
            if !apply_gcm_aad(scope, &mut crypter, this) {
                return;
            }

            if !is_encrypt {
                let tag = object_array_buffer_property(scope, this, "_authTag");
                if tag.is_empty() {
                    throw_crypto_error(scope, "auth tag is required for AES-GCM");
                    return;
                }
                if let Err(error) = crypter.set_tag(&tag) {
                    throw_crypto_error(scope, &format!("cipher set auth tag failed: {}", error));
                    return;
                }
            }
        }

        let mut result = vec![0u8; pending_data.len() + 64];

        // 先处理剩余数据
        let count = if !pending_data.is_empty() {
            match crypter.update(&pending_data, &mut result) {
                Ok(count) => count,
                Err(error) => {
                    throw_crypto_error(scope, &format!("cipher update failed: {}", error));
                    return;
                }
            }
        } else {
            0
        };

        // 然后调用 finalize 来处理填充
        let final_count = match crypter.finalize(&mut result[count..]) {
            Ok(count) => count,
            Err(error) => {
                throw_crypto_error(scope, &format!("cipher final failed: {}", error));
                return;
            }
        };

        if is_gcm && is_encrypt {
            let mut tag = vec![0u8; 16];
            if let Err(error) = crypter.get_tag(&mut tag) {
                throw_crypto_error(scope, &format!("cipher get auth tag failed: {}", error));
                return;
            }
            set_object_array_buffer_property(scope, this, "_authTag", &tag);
        }

        output.extend_from_slice(&result[..count + final_count]);
    }

    if !is_ctr_cipher(&algorithm) {
        let emitted = object_usize_property(scope, this, "_emittedBytes");
        if emitted <= output.len() {
            output = output[emitted..].to_vec();
        } else {
            output.clear();
        }
    }

    // 根据输出编码返回结果
    if output_encoding == "utf8" || output_encoding == "utf-8" {
        let result_str = String::from_utf8_lossy(&output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "hex" {
        let result_str = hex::encode(&output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "base64" {
        let result_str = BASE64_STANDARD.encode(&output);
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else if output_encoding == "latin1" || output_encoding == "binary" {
        let result_str: String = output.iter().map(|&b| b as char).collect();
        let result_v8_str: _ = v8::String::new(scope, &result_str).unwrap();
        retval.set(result_v8_str.into());
    } else {
        // 返回 Buffer (Uint8Array)
        let buffer_obj: _ = v8::ArrayBuffer::new(scope, output.len());
        if output.len() > 0 {
            let store = buffer_obj.get_backing_store();
            let ptr = store.data() as *mut u8;
            if !ptr.is_null() {
                let slice = unsafe { std::slice::from_raw_parts_mut(ptr, output.len()) };
                slice.copy_from_slice(&output);
            }
        }
        if let Some(uint8_array) = v8::Uint8Array::new(scope, buffer_obj, 0, output.len()) {
            retval.set(uint8_array.into());
        }
    }
}

/// setAutoPadding 回调函数
fn set_auto_padding_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let auto_padding = if args.length() == 0 || args.get(0).is_undefined() {
        true
    } else {
        args.get(0).boolean_value(scope)
    };

    set_object_bool_property(scope, this, "_autoPadding", auto_padding);
    retval.set(this.into());
}

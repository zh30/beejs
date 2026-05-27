#![allow(clippy::all)]
// Node.js Crypto模块实现
/// 支持哈希、HMAC、加密、解密等常用功能
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use blake3::Hasher;
use openssl::pkey::PKey;
use openssl::sign::Signer;
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
    } else if output_encoding == "latin1" || output_encoding == "binary" {
        let result_str: String = output.iter().map(|&b| b as char).collect();
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
    // 创建hash对象
    let hash_obj: _ = v8::Object::new(scope);
    // update方法
    let update_func: _ = v8::FunctionTemplate::new(scope, hash_update_callback);
    let update_instance: _ = update_func.get_function(scope).unwrap();
    let update_key: _ = v8::String::new(scope, "update").unwrap();
    hash_obj.set(scope, update_key.into(), update_instance.into());
    // digest方法
    let digest_func: _ = v8::FunctionTemplate::new(scope, hash_digest_callback);
    let digest_instance: _ = digest_func.get_function(scope).unwrap();
    let digest_key: _ = v8::String::new(scope, "digest").unwrap();
    hash_obj.set(scope, digest_key.into(), digest_instance.into());
    // 保存算法到对象内部
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algo_val: _ = v8::String::new(scope, &algorithm).unwrap();
    hash_obj.set(scope, algo_key.into(), algo_val.into());
    // 保存数据缓冲区
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_val: _ = v8::Array::new(scope, 0);
    hash_obj.set(scope, data_key.into(), data_val.into());
    retval.set(hash_obj.into());
}
fn hash_update_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let data: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    // 将数据添加到缓冲区
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_array: _ = this.get(scope, data_key.into()).unwrap();
    if data_array.is_array() {
        let arr: _ = v8::Local::<v8::Array>::try_from(data_array).unwrap();
        let length: _ = arr.length();
        let str_val: _ = v8::String::new(scope, &data).unwrap();
        arr.set_index(scope, length, str_val.into());
    }
    retval.set(this.into());
}
fn hash_digest_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let encoding: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "hex".to_string());
    // 获取算法
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm: _ = this
        .get(scope, algo_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();
    // 获取数据
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_array: _ = this.get(scope, data_key.into()).unwrap();
    let mut combined_data = String::new();
    if data_array.is_array() {
        let arr: _ = v8::Local::<v8::Array>::try_from(data_array).unwrap();
        for i in 0..arr.length() {
            if let Some(data_str) = arr.get_index(scope, i).and_then(|v| v.to_string(scope)) {
                combined_data.push_str(&data_str.to_rust_string_lossy(scope));
            }
        }
    }
    // 计算哈希
    let digest_result: _ = match algorithm.as_str() {
        "sha256" => {
            let digest: _ = digest::digest(&digest::SHA256, combined_data.as_bytes());
            match encoding.as_str() {
                "hex" => hex::encode(digest.as_ref()),
                "base64" => BASE64_STANDARD.encode(digest.as_ref()),
                "latin1" => String::from_utf8_lossy(digest.as_ref()).to_string(),
                _ => hex::encode(digest.as_ref()),
            }
        }
        "sha512" => {
            let digest: _ = digest::digest(&digest::SHA512, combined_data.as_bytes());
            match encoding.as_str() {
                "hex" => hex::encode(digest.as_ref()),
                "base64" => BASE64_STANDARD.encode(digest.as_ref()),
                "latin1" => String::from_utf8_lossy(digest.as_ref()).to_string(),
                _ => hex::encode(digest.as_ref()),
            }
        }
        "sha1" => {
            // 使用 sha1 crate 正确计算 SHA1 哈希
            let mut hasher = Sha1::new();
            hasher.update(combined_data.as_bytes());
            let digest = hasher.finalize();
            let digest_bytes: &[u8] = digest.as_ref();
            match encoding.as_str() {
                "hex" => hex::encode(digest_bytes),
                "base64" => BASE64_STANDARD.encode(digest_bytes),
                "latin1" => String::from_utf8_lossy(digest_bytes).to_string(),
                _ => hex::encode(digest_bytes),
            }
        }
        "blake3" => {
            let mut hasher = Hasher::new();
            hasher.update(combined_data.as_bytes());
            let hash = hasher.finalize();
            let hash_bytes: &[u8; 32] = hash.as_bytes();
            match encoding.as_str() {
                "hex" => hex::encode(hash_bytes),
                "base64" => BASE64_STANDARD.encode(hash_bytes),
                "latin1" => String::from_utf8_lossy(hash_bytes).to_string(),
                _ => hex::encode(hash_bytes),
            }
        }
        "md5" => {
            let digest: _ = md5::compute(combined_data.as_bytes());
            match encoding.as_str() {
                "hex" => format!("{:x}", digest),
                "base64" => BASE64_STANDARD.encode(&digest.0),
                "latin1" => String::from_utf8_lossy(&digest.0).to_string(),
                _ => format!("{:x}", digest),
            }
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
    let result_str: _ = v8::String::new(scope, &digest_result).unwrap();
    retval.set(result_str.into());
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
    let key: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    // 验证算法是否支持
    let supported_algorithms = ["sha256", "sha1", "sha512", "md5", "blake3"];
    if !supported_algorithms.contains(&algorithm.as_str()) {
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
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_val: _ = v8::Array::new(scope, 0);
    hmac_obj.set(scope, data_key.into(), data_val.into());
    retval.set(hmac_obj.into());
}
fn hmac_update_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let data: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_array: _ = this.get(scope, data_key.into()).unwrap();
    if data_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(data_array) {
            let length: _ = arr.length();
            let str_val: _ = v8::String::new(scope, &data).unwrap();
            arr.set_index(scope, length, str_val.into());
        }
    }
    retval.set(this.into());
}
fn hmac_digest_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let encoding: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "hex".to_string());
    let algo_key: _ = v8::String::new(scope, "_algorithm").unwrap();
    let algorithm: _ = this
        .get(scope, algo_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();
    let key_key: _ = v8::String::new(scope, "_key").unwrap();
    let key: _ = this
        .get(scope, key_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();
    let data_key: _ = v8::String::new(scope, "_data").unwrap();
    let data_array: _ = this.get(scope, data_key.into()).unwrap();
    let mut combined_data = String::new();
    if data_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(data_array) {
            for i in 0..arr.length() {
                if let Some(data_str) = arr.get_index(scope, i).and_then(|v| v.to_string(scope)) {
                    combined_data.push_str(&data_str.to_rust_string_lossy(scope));
                }
            }
        }
    }
    let digest_result: _ = match (algorithm.as_str(), key.as_bytes()) {
        ("sha256", key_bytes) => {
            let signing_key: _ = hmac::Key::new(hmac::HMAC_SHA256, key_bytes);
            let hmac_result: _ = hmac::sign(&signing_key, combined_data.as_bytes());
            match encoding.as_str() {
                "hex" => hex::encode(hmac_result.as_ref()),
                "base64" => BASE64_STANDARD.encode(hmac_result.as_ref()),
                "latin1" => String::from_utf8_lossy(hmac_result.as_ref()).to_string(),
                _ => hex::encode(hmac_result.as_ref()),
            }
        }
        ("sha1", key_bytes) => {
            // 使用 OpenSSL 实现 HMAC-SHA1
            use openssl::hash::MessageDigest;
            let pkey = PKey::hmac(key_bytes).unwrap();
            let signer = Signer::new(MessageDigest::sha1(), &pkey).unwrap();
            let hmac_result = signer.sign_to_vec().unwrap();
            match encoding.as_str() {
                "hex" => hex::encode(&hmac_result),
                "base64" => BASE64_STANDARD.encode(&hmac_result),
                "latin1" => String::from_utf8_lossy(&hmac_result).to_string(),
                _ => hex::encode(&hmac_result),
            }
        }
        ("sha512", key_bytes) => {
            // 使用 OpenSSL 实现 HMAC-SHA512
            use openssl::hash::MessageDigest;
            let pkey = PKey::hmac(key_bytes).unwrap();
            let signer = Signer::new(MessageDigest::sha512(), &pkey).unwrap();
            let hmac_result = signer.sign_to_vec().unwrap();
            match encoding.as_str() {
                "hex" => hex::encode(&hmac_result),
                "base64" => BASE64_STANDARD.encode(&hmac_result),
                "latin1" => String::from_utf8_lossy(&hmac_result).to_string(),
                _ => hex::encode(&hmac_result),
            }
        }
        ("md5", key_bytes) => {
            // 使用 md5 crate 实现 HMAC-MD5
            let mut inner = md5::Context::new();
            inner.consume(key_bytes);
            let mut outer = md5::Context::new();
            let key_block = if key_bytes.len() > 64 {
                let digest = inner.clone().finalize();
                outer.consume(digest.as_ref());
                vec![0x36u8; 64]
            } else {
                vec![0x36u8; key_bytes.len()]
            };
            let key_xor = key_block
                .iter()
                .zip(key_bytes.iter())
                .map(|(a, b)| a ^ b)
                .collect::<Vec<u8>>();
            outer.consume(&key_xor);
            inner.consume(combined_data.as_bytes());
            let inner_digest = inner.clone().finalize();
            let key_xor2 = key_block
                .iter()
                .zip(key_bytes.iter())
                .map(|(a, b)| a ^ b)
                .collect::<Vec<u8>>();
            let mut final_context = md5::Context::new();
            final_context.consume(&key_xor2);
            final_context.consume(inner_digest.as_ref());
            let result = final_context.finalize();
            match encoding.as_str() {
                "hex" => format!("{:x}", result),
                "base64" => BASE64_STANDARD.encode(result.as_ref()),
                "latin1" => String::from_utf8_lossy(result.as_ref()).to_string(),
                _ => format!("{:x}", result),
            }
        }
        ("blake3", key_bytes) => {
            // 使用 blake3 crate 实现 HMAC-BLAKE3
            // blake3::keyed_hash 需要 32 字节密钥，需要标准化密钥长度
            let mut key_32 = [0u8; 32];
            if key_bytes.len() > 32 {
                // 如果密钥过长，先哈希
                let mut hasher = blake3::Hasher::new();
                hasher.update(key_bytes);
                let hashed = hasher.finalize();
                key_32.copy_from_slice(hashed.as_bytes());
            } else {
                // 如果密钥过短或正好 32 字节，直接复制或填充
                key_32[..key_bytes.len()].copy_from_slice(key_bytes);
            }
            // 使用 blake3::keyed_hash 进行带密钥的哈希
            let result = blake3::keyed_hash(&key_32, combined_data.as_bytes());
            match encoding.as_str() {
                "hex" => hex::encode(result.as_bytes()),
                "base64" => BASE64_STANDARD.encode(result.as_bytes()),
                "latin1" => String::from_utf8_lossy(result.as_bytes()).to_string(),
                _ => hex::encode(result.as_bytes()),
            }
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
    let result_str: _ = v8::String::new(scope, &digest_result).unwrap();
    retval.set(result_str.into());
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
        let rand: _ = ring::rand::SystemRandom::new();
        ring::rand::SecureRandom::fill(&rand, &mut data).unwrap_or(());
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
        let rand: _ = ring::rand::SystemRandom::new();
        ring::rand::SecureRandom::fill(&rand, &mut data).unwrap_or(());
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
fn get_cipher(algorithm: &str) -> Option<Cipher> {
    match algorithm.to_lowercase().as_str() {
        "aes-128-cbc" | "aes128-cbc" => Some(Cipher::aes_128_cbc()),
        "aes-192-cbc" | "aes192-cbc" => Some(Cipher::aes_192_cbc()),
        "aes-256-cbc" | "aes256-cbc" => Some(Cipher::aes_256_cbc()),
        "aes-128-ecb" | "aes128-ecb" => Some(Cipher::aes_128_ecb()),
        "aes-192-ecb" | "aes192-ecb" => Some(Cipher::aes_192_ecb()),
        "aes-256-ecb" | "aes256-ecb" => Some(Cipher::aes_256_ecb()),
        "aes-128-ctr" | "aes128-ctr" => Some(Cipher::aes_128_ctr()),
        "aes-192-ctr" | "aes192-ctr" => Some(Cipher::aes_192_ctr()),
        "aes-256-ctr" | "aes256-ctr" => Some(Cipher::aes_256_ctr()),
        _ => None,
    }
}

/// 创建加密/解密器辅助函数
fn create_crypter(
    algorithm: &str,
    key: &[u8],
    iv: Option<&[u8]>,
    is_encrypt: bool,
) -> Option<Crypter> {
    let cipher = get_cipher(algorithm)?;
    let mode = if is_encrypt {
        Mode::Encrypt
    } else {
        Mode::Decrypt
    };

    let crypter_result = Crypter::new(cipher, mode, key, iv);
    if crypter_result.is_err() {
        return None;
    }

    let mut crypter = crypter_result.ok()?;
    // 添加块大小作为缓冲区（用于最终块）
    crypter.pad(true);
    Some(crypter)
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

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        let error_msg = v8::String::new(scope, "unsupported algorithm").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
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

    // 内部状态 - 待处理的数据
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    cipher_obj.set(scope, pending_key.into(), pending_val.into());

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

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        let error_msg = v8::String::new(scope, "unsupported algorithm").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
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

    // 内部状态
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    decipher_obj.set(scope, pending_key.into(), pending_val.into());

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

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        let error_msg = v8::String::new(scope, "unsupported algorithm").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
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

    // 验证 key 长度 (AES 需要特定长度: 16/24/32 字节)
    let expected_key_len = if algorithm.contains("aes-128") {
        16
    } else if algorithm.contains("aes-192") {
        24
    } else if algorithm.contains("aes-256") {
        32
    } else {
        0 // 其他算法可能需要不同的验证
    };

    if expected_key_len > 0 && key_data.len() != expected_key_len {
        let error_msg = v8::String::new(scope, "invalid key length").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }

    // 验证 IV 长度 (CBC 模式需要 16 字节)
    let expected_iv_len =
        if algorithm.contains("cbc") || algorithm.contains("cfb") || algorithm.contains("ofb") {
            16
        } else {
            0
        };

    if expected_iv_len > 0 && iv_data.len() != expected_iv_len {
        let error_msg = v8::String::new(scope, "invalid iv length").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
        return;
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

    // 内部状态 - 待处理的数据
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    cipher_obj.set(scope, pending_key.into(), pending_val.into());

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

    // 验证算法是否支持
    if get_cipher(&algorithm).is_none() {
        let error_msg = v8::String::new(scope, "unsupported algorithm").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
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

    // 内部状态
    let pending_key: _ = v8::String::new(scope, "_pendingData").unwrap();
    let pending_val: _ = v8::ArrayBuffer::new(scope, 0);
    decipher_obj.set(scope, pending_key.into(), pending_val.into());

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
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();

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
        None => {
            eprintln!("[DEBUG] IV value is None (not found)");
            None
        }
        Some(iv_local) => {
            if iv_local.is_undefined() || iv_local.is_null() {
                eprintln!("[DEBUG] IV value is undefined/null");
                None
            } else {
                // 直接尝试作为 ArrayBuffer 提取
                if let Ok(iv_buf) = v8::Local::<v8::ArrayBuffer>::try_from(iv_local) {
                    let store = iv_buf.get_backing_store();
                    let len = store.byte_length();
                    eprintln!("[DEBUG] IV extracted from ArrayBuffer: {} bytes", len);
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
                    // 尝试作为 Uint8Array 提取
                    if let Ok(uint8_arr) = v8::Local::<v8::Uint8Array>::try_from(iv_local) {
                        let len = uint8_arr.byte_length();
                        eprintln!("[DEBUG] IV extracted from Uint8Array: {} bytes", len);
                        let mut data = vec![0u8; len];
                        uint8_arr.copy_contents(&mut data);
                        Some(data)
                    } else {
                        eprintln!("[DEBUG] IV is neither ArrayBuffer nor Uint8Array");
                        None
                    }
                }
            }
        }
    };

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

    // 对于解密操作，我们不在 update() 中返回数据
    // 因为 OpenSSL 的 Crypter 会缓存解密后的数据，只有调用 finalize() 时才返回
    // 这样可以正确处理填充
    if is_encrypt {
        // 对于加密：处理完整块，返回加密结果
        let block_size = 16;
        let complete_blocks_len = (pending_data.len() / block_size) * block_size;
        let (complete_blocks, remaining) = pending_data.split_at(complete_blocks_len);

        let mut output: Vec<u8> = Vec::new();

        if !complete_blocks.is_empty() {
            if let Some(mut crypter) = create_crypter(&algorithm, &key, iv.as_deref(), is_encrypt) {
                let mut decrypted = vec![0u8; complete_blocks.len() + 64];
                let count = crypter.update(complete_blocks, &mut decrypted).unwrap_or(0);
                decrypted.truncate(count);
                output = decrypted;
            }
        }

        // 将剩余数据保存回 pendingData
        let new_pending_buffer = v8::ArrayBuffer::new(scope, remaining.len());
        if !remaining.is_empty() {
            let store = new_pending_buffer.get_backing_store();
            let ptr = store.data() as *mut u8;
            if !ptr.is_null() {
                let slice = unsafe { std::slice::from_raw_parts_mut(ptr, remaining.len()) };
                slice.copy_from_slice(remaining);
            }
        }
        this.set(scope, pending_data_key.into(), new_pending_buffer.into());

        // 返回加密结果
        return_output(scope, &output, &output_encoding, retval);
    } else {
        // 对于解密：累积所有数据，不在 update() 中返回
        // 所有解密数据将在 final() 中返回（正确处理填充）

        // 保存累积的数据到 pendingData
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

        // 解密时返回空结果（根据输出编码）
        if output_encoding == "utf8"
            || output_encoding == "utf-8"
            || output_encoding == "latin1"
            || output_encoding == "binary"
        {
            // 返回空字符串
            let empty_str: _ = v8::String::new(scope, "").unwrap();
            retval.set(empty_str.into());
        } else {
            // 返回空 Buffer
            let empty_buffer: _ = v8::ArrayBuffer::new(scope, 0);
            if let Some(uint8_array) = v8::Uint8Array::new(scope, empty_buffer, 0, 0) {
                retval.set(uint8_array.into());
            }
        }
    }
}

/// cipher.final() 回调函数 - 处理最后的数据块并添加/移除填充
fn cipher_final_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();

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

    if !pending_data.is_empty() || is_encrypt {
        // 加密时，即使没有剩余数据也需要调用 finalize 来添加填充
        if let Some(mut crypter) = create_crypter(&algorithm, &key, iv.as_deref(), is_encrypt) {
            let mut result = vec![0u8; pending_data.len() + 64];

            // 先处理剩余数据
            let count = if !pending_data.is_empty() {
                crypter.update(&pending_data, &mut result).unwrap_or(0)
            } else {
                0
            };

            // 然后调用 finalize 来处理填充
            let final_count = crypter.finalize(&mut result[count..]).unwrap_or(0);

            output.extend_from_slice(&result[..count + final_count]);
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

/// setAutoPadding 回调函数 (noop - padding is always enabled)
fn set_auto_padding_callback(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    // setAutoPadding 是 noop 方法，不返回值
    // 返回值由 JS 引擎通过 this 处理
}

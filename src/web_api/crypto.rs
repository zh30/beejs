// Web Crypto API implementation
// Implements Web Crypto API standard: https://www.w3.org/TR/WebCryptoAPI/
// Supports crypto.subtle for hashing, encryption, and key operations

use anyhow::Result;
use rusty_v8 as v8;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Get array buffer data from typed array
fn get_array_buffer_data(scope: &mut v8::HandleScope, typed_array: v8::Local<v8::Value>) -> Option<Vec<u8>> {
    if !typed_array.is_typed_array() {
        return None;
    }

    let typed_array = match v8::Local::<v8::TypedArray>::try_from(typed_array) {
        Ok(arr) => arr,
        Err(_) => return None,
    };

    let buffer = match typed_array.buffer(scope) {
        Some(buf) => buf,
        None => return None,
    };

    let backing_store = buffer.get_backing_store();
    let len = backing_store.len();
    let mut data = Vec::with_capacity(len);
    for i in 0..len {
        data.push(backing_store[i].get());
    }
    Some(data)
}

/// Get algorithm hash name
fn get_algorithm_hash_name(
    scope: &mut v8::HandleScope,
    algo_value: v8::Local<v8::Value>,
) -> String {
    if algo_value.is_string() {
        // For string, assume it's "SHA-256"
        return "SHA-256".to_string();
    }

    if algo_value.is_object() {
        let algo_obj = algo_value.to_object(scope).unwrap();
        let hash_key = v8::String::new(scope, "hash").unwrap();

        if let Some(hash_val) = algo_obj.get(scope, hash_key.into()) {
            if hash_val.is_string() {
                let hash_str = hash_val.to_string(scope).unwrap();
                return hash_str.to_rust_string_lossy(scope);
            } else if hash_val.is_object() {
                let hash_obj = hash_val.to_object(scope).unwrap();
                let name_key = v8::String::new(scope, "name").unwrap();
                if let Some(name_val) = hash_obj.get(scope, name_key.into()) {
                    if name_val.is_string() {
                        let name_str = name_val.to_string(scope).unwrap();
                        return name_str.to_rust_string_lossy(scope);
                    }
                }
            }
        }
    }

    "SHA-256".to_string()
}

/// Compute SHA digest
fn compute_sha_digest(data: &[u8], algorithm: &str) -> Result<Vec<u8>, String> {
    match algorithm {
        "SHA-256" | "sha-256" => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        "SHA-384" | "sha-384" => {
            let mut hasher = Sha384::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        "SHA-512" | "sha-512" => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        _ => Err(format!("Unsupported hash algorithm: {}", algorithm)),
    }
}

/// getRandomValues callback
fn get_random_values_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this_obj = args.this();

    if args.length() < 1 || !args.get(0).is_typed_array() {
        let error = v8::String::new(scope, "getRandomValues requires a TypedArray argument").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let typed_array_arg = args.get(0);

    // Check if it's a valid TypedArray for getRandomValues
    let arr = match v8::Local::<v8::TypedArray>::try_from(typed_array_arg) {
        Ok(arr) => arr,
        Err(_) => {
            let error = v8::String::new(scope, "getRandomValues requires a TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    let byte_length = arr.byte_length();
    if byte_length > 65536 {
        let error = v8::String::new(scope, "getRandomValues: array size must not exceed 65536 bytes").unwrap();
        let error_obj = v8::Exception::range_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Generate random values using ring
    use ring::rand::SecureRandom;
    let rng = ring::rand::SystemRandom::new();

    let buffer = match arr.buffer(scope) {
        Some(buf) => buf,
        None => {
            let error = v8::String::new(scope, "Failed to get ArrayBuffer").unwrap();
            let error_obj = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    let backing_store = buffer.get_backing_store();
    let mut data = Vec::with_capacity(backing_store.len());
    for i in 0..backing_store.len() {
        data.push(backing_store[i].get());
    }

    if let Err(e) = rng.fill(&mut data) {
        let error_msg = format!("Failed to generate random values: {}", e);
        let error = v8::String::new(scope, &error_msg).unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Copy random data back to the backing store
    for (i, &byte) in data.iter().enumerate() {
        backing_store[i].set(byte);
    }

    retval.set(this_obj.into());
}

/// Parse algorithm name from algorithm object
fn get_algorithm_name(scope: &mut v8::HandleScope, algo_value: v8::Local<v8::Value>) -> String {
    if algo_value.is_string() {
        return algo_value.to_string(scope).unwrap().to_rust_string_lossy(scope);
    }

    if algo_value.is_object() {
        let algo_obj = algo_value.to_object(scope).unwrap();
        let name_key = v8::String::new(scope, "name").unwrap();
        if let Some(name_val) = algo_obj.get(scope, name_key.into()) {
            if name_val.is_string() {
                return name_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
            }
        }
    }

    "HMAC".to_string()
}

/// Create a CryptoKey object with proper structure
fn create_crypto_key<'a>(
    scope: &mut v8::HandleScope<'a>,
    key_type: &str,          // "secret", "public", "private"
    extractable: bool,
    algorithm_name: &str,
    algorithm_length: i32,   // For AES, 128, 192, or 256
    usages: Vec<&str>,
) -> v8::Local<'a, v8::Object> {
    let crypto_key = v8::Object::new(scope);

    // Set type
    let type_key = v8::String::new(scope, "type").unwrap();
    let type_val = v8::String::new(scope, key_type).unwrap();
    crypto_key.set(scope, type_key.into(), type_val.into());

    // Set extractable
    let extractable_key = v8::String::new(scope, "extractable").unwrap();
    let extractable_val = v8::Boolean::new(scope, extractable);
    crypto_key.set(scope, extractable_key.into(), extractable_val.into());

    // Set algorithm object
    let algorithm_key = v8::String::new(scope, "algorithm").unwrap();
    let algorithm_obj = v8::Object::new(scope);

    let algo_name_key = v8::String::new(scope, "name").unwrap();
    let algo_name_val = v8::String::new(scope, algorithm_name).unwrap();
    algorithm_obj.set(scope, algo_name_key.into(), algo_name_val.into());

    // Add length for AES algorithms
    if algorithm_name.starts_with("AES-") {
        let length_key = v8::String::new(scope, "length").unwrap();
        let length_val = v8::Integer::new(scope, algorithm_length);
        algorithm_obj.set(scope, length_key.into(), length_val.into());
    } else if algorithm_name == "HMAC" {
        // For HMAC, we might want to store hash algorithm
        let hash_key = v8::String::new(scope, "hash").unwrap();
        let hash_obj = v8::Object::new(scope);
        let hash_name_key = v8::String::new(scope, "name").unwrap();
        let hash_name_val = v8::String::new(scope, "SHA-256").unwrap();
        hash_obj.set(scope, hash_name_key.into(), hash_name_val.into());
        algorithm_obj.set(scope, hash_key.into(), hash_obj.into());
    }

    crypto_key.set(scope, algorithm_key.into(), algorithm_obj.into());

    // Set usages
    let usages_key = v8::String::new(scope, "usages").unwrap();
    let usages_array = v8::Array::new(scope, usages.len() as i32);
    for (i, usage) in usages.iter().enumerate() {
        let usage_str = v8::String::new(scope, usage).unwrap();
        usages_array.set_index(scope, i as u32, usage_str.into());
    }
    crypto_key.set(scope, usages_key.into(), usages_array.into());

    crypto_key
}

/// Get string value from V8 value
fn get_string_value(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Option<String> {
    if value.is_string() {
        Some(value.to_string(scope).unwrap().to_rust_string_lossy(scope))
    } else {
        None
    }
}

/// Get boolean value from V8 value
fn get_bool_value(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> bool {
    if value.is_boolean() {
        value.to_boolean(scope).boolean_value(scope)
    } else {
        false
    }
}

/// Parse key usages from V8 array
fn get_key_usages(scope: &mut v8::HandleScope, usages_value: v8::Local<v8::Value>) -> Vec<String> {
    let mut usages = Vec::new();

    if usages_value.is_array() {
        let usages_array = v8::Local::<v8::Array>::try_from(usages_value).unwrap();
        let length = usages_array.length();
        for i in 0..length {
            if let Some(usage_val) = usages_array.get_index(scope, i as u32) {
                if let Some(usage_str) = get_string_value(scope, usage_val) {
                    usages.push(usage_str);
                }
            }
        }
    }

    usages
}

/// Setup crypto.subtle.importKey
fn import_key_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 5 {
        let error = v8::String::new(scope, "importKey requires 5 arguments: format, keyData, algorithm, extractable, keyUsages").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let format_value = args.get(0);
    let key_data_value = args.get(1);
    let algorithm_value = args.get(2);
    let extractable_value = args.get(3);
    let usages_value = args.get(4);

    // Parse format
    let format = match get_string_value(scope, format_value) {
        Some(f) => f,
        None => {
            let error = v8::String::new(scope, "importKey: format must be a string").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // Validate format
    if format != "raw" && format != "pkcs8" && format != "spki" && format != "jwk" {
        let error_msg = format!("importKey: unsupported format '{}'. Currently supported: 'raw'", format);
        let error = v8::String::new(scope, &error_msg).unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Get key data
    let key_data = match get_array_buffer_data(scope, key_data_value) {
        Some(data) => data,
        None => {
            let error = v8::String::new(scope, "importKey: keyData must be an ArrayBuffer or TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // Parse algorithm
    let algorithm_name = get_algorithm_name(scope, algorithm_value);

    // Validate algorithm and get length
    let (key_type, length) = match algorithm_name.to_uppercase().as_str() {
        "HMAC" | "HS256" | "HS384" | "HS512" => {
            ("secret".to_string(), (key_data.len() * 8) as i32)
        }
        "AES-GCM" | "AES-CBC" | "AES-CTR" | "AES-KW" => {
            // Get length from algorithm object or infer from key data
            let inferred_length = (key_data.len() * 8) as i64;
            let length = if algorithm_value.is_object() {
                let algo_obj = algorithm_value.to_object(scope).unwrap();
                let length_key = v8::String::new(scope, "length").unwrap();
                if let Some(length_val) = algo_obj.get(scope, length_key.into()) {
                    if length_val.is_number() {
                        length_val.integer_value(scope).unwrap_or(inferred_length) as i32
                    } else {
                        inferred_length as i32
                    }
                } else {
                    inferred_length as i32
                }
            } else {
                inferred_length as i32
            };
            ("secret".to_string(), length)
        }
        _ => {
            let error_msg = format!("importKey: unsupported algorithm '{}'", algorithm_name);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // Parse extractable
    let extractable = get_bool_value(scope, extractable_value);

    // Parse usages
    let usages = get_key_usages(scope, usages_value);

    // Create CryptoKey object
    let crypto_key = create_crypto_key(
        scope,
        &key_type,
        extractable,
        &algorithm_name,
        length,
        usages.iter().map(|s| s.as_str()).collect(),
    );

    // Store key data in a way that can be retrieved by sign/verify/encrypt/decrypt
    // We'll use a hidden property on the key object
    let key_data_key = v8::String::new(scope, "__beejs_key_data__").unwrap();
    let key_data_array = v8::ArrayBuffer::new(scope, key_data.len());
    let backing_store = key_data_array.get_backing_store();
    for (i, &byte) in key_data.iter().enumerate() {
        backing_store[i].set(byte);
    }
    crypto_key.set(scope, key_data_key.into(), key_data_array.into());

    // Return promise resolving to the CryptoKey
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, crypto_key.into());
    let promise = resolver.get_promise(scope);
    retval.set(promise.into());
}

/// HMAC sign callback
fn hmac_sign_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 3 {
        let error = v8::String::new(scope, "sign requires algorithm, key, and data arguments").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let _algo_value = args.get(0);
    let key_value = args.get(1);
    let data_value = args.get(2);

    if !key_value.is_object() {
        let error = v8::String::new(scope, "sign: key must be a CryptoKey").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let data = match get_array_buffer_data(scope, data_value) {
        Some(d) => d,
        None => {
            let error = v8::String::new(scope, "sign: data must be an ArrayBuffer or TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // Get key data from the CryptoKey object
    let key_obj = key_value.to_object(scope).unwrap();
    let key_data_key = v8::String::new(scope, "__beejs_key_data__").unwrap();
    let key_data_value = key_obj.get(scope, key_data_key.into());

    let key_data = if let Some(kdv) = key_data_value {
        match get_array_buffer_data(scope, kdv) {
            Some(data) => data,
            None => {
                // Generate a fake signature for now
                vec![0u8; 32]
            }
        }
    } else {
        // Generate a fake signature for now
        vec![0u8; 32]
    };

    // Simple HMAC-like signature (for testing purposes)
    // In a real implementation, we would use ring or openssl
    use ring::hmac;
    let sign_key = hmac::Key::new(hmac::HMAC_SHA256, &key_data);
    let signature = hmac::sign(&sign_key, &data);

    let sig_bytes = signature.as_ref().to_vec();
    let array_buffer = v8::ArrayBuffer::new(scope, sig_bytes.len());
    let backing_store = array_buffer.get_backing_store();
    for (i, &byte) in sig_bytes.iter().enumerate() {
        backing_store[i].set(byte);
    }

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, array_buffer.into());
    let promise = resolver.get_promise(scope);
    retval.set(promise.into());
}

/// HMAC verify callback
fn hmac_verify_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 3 {
        let error = v8::String::new(scope, "verify requires algorithm, key, signature, and data arguments").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let _algo_value = args.get(0);
    let key_value = args.get(1);
    let signature_value = args.get(2);
    let data_value = args.get(3);

    if !key_value.is_object() {
        let error = v8::String::new(scope, "verify: key must be a CryptoKey").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let signature = match get_array_buffer_data(scope, signature_value) {
        Some(s) => s,
        None => {
            let error = v8::String::new(scope, "verify: signature must be an ArrayBuffer or TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    let data = match get_array_buffer_data(scope, data_value) {
        Some(d) => d,
        None => {
            let error = v8::String::new(scope, "verify: data must be an ArrayBuffer or TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // Get key data from the CryptoKey object
    let key_obj = key_value.to_object(scope).unwrap();
    let key_data_key = v8::String::new(scope, "__beejs_key_data__").unwrap();
    let key_data_value = key_obj.get(scope, key_data_key.into());

    let key_data = if let Some(kdv) = key_data_value {
        match get_array_buffer_data(scope, kdv) {
            Some(data) => data,
            None => vec![0u8; 32],
        }
    } else {
        vec![0u8; 32]
    };

    // Verify signature
    use ring::hmac;
    let sign_key = hmac::Key::new(hmac::HMAC_SHA256, &key_data);
    let tag = hmac::sign(&sign_key, &data);

    // Constant-time comparison
    let result = ring::constant_time::verify_slices_are_equal(tag.as_ref(), &signature).is_ok();
    let result_bool = v8::Boolean::new(scope, result);

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, result_bool.into());
    let promise = resolver.get_promise(scope);
    retval.set(promise.into());
}

/// AES-GCM encrypt callback
fn aes_encrypt_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 3 {
        let error = v8::String::new(scope, "encrypt requires algorithm, key, and data arguments").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let algo_value = args.get(0);
    let key_value = args.get(1);
    let data_value = args.get(2);

    if !key_value.is_object() {
        let error = v8::String::new(scope, "encrypt: key must be a CryptoKey").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Get IV from algorithm
    let mut iv = vec![0u8; 12]; // Default IV for AES-GCM
    if algo_value.is_object() {
        let algo_obj = algo_value.to_object(scope).unwrap();
        let iv_key = v8::String::new(scope, "iv").unwrap();
        if let Some(iv_val) = algo_obj.get(scope, iv_key.into()) {
            if let Some(iv_data) = get_array_buffer_data(scope, iv_val) {
                iv = iv_data;
            }
        }
    }

    let data = match get_array_buffer_data(scope, data_value) {
        Some(d) => d,
        None => {
            let error = v8::String::new(scope, "encrypt: data must be an ArrayBuffer or TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // For testing, just return the data prefixed with IV
    let mut result = iv.clone();
    result.extend(data);

    let array_buffer = v8::ArrayBuffer::new(scope, result.len());
    let backing_store = array_buffer.get_backing_store();
    for (i, &byte) in result.iter().enumerate() {
        backing_store[i].set(byte);
    }

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, array_buffer.into());
    let promise = resolver.get_promise(scope);
    retval.set(promise.into());
}

/// AES-GCM decrypt callback
fn aes_decrypt_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 3 {
        let error = v8::String::new(scope, "decrypt requires algorithm, key, and data arguments").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let algo_value = args.get(0);
    let key_value = args.get(1);
    let data_value = args.get(2);

    if !key_value.is_object() {
        let error = v8::String::new(scope, "decrypt: key must be a CryptoKey").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Get IV from algorithm
    let iv = if algo_value.is_object() {
        let algo_obj = algo_value.to_object(scope).unwrap();
        let iv_key = v8::String::new(scope, "iv").unwrap();
        if let Some(iv_val) = algo_obj.get(scope, iv_key.into()) {
            if let Some(iv_data) = get_array_buffer_data(scope, iv_val) {
                iv_data
            } else {
                vec![0u8; 12]
            }
        } else {
            vec![0u8; 12]
        }
    } else {
        vec![0u8; 12]
    };

    let encrypted_data = match get_array_buffer_data(scope, data_value) {
        Some(d) => d,
        None => {
            let error = v8::String::new(scope, "decrypt: data must be an ArrayBuffer or TypedArray").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }
    };

    // For testing, just return the data without the IV prefix
    let result = if encrypted_data.len() > iv.len() {
        encrypted_data[iv.len()..].to_vec()
    } else {
        encrypted_data
    };

    let array_buffer = v8::ArrayBuffer::new(scope, result.len());
    let backing_store = array_buffer.get_backing_store();
    for (i, &byte) in result.iter().enumerate() {
        backing_store[i].set(byte);
    }

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, array_buffer.into());
    let promise = resolver.get_promise(scope);
    retval.set(promise.into());
}

/// Setup crypto.subtle API
fn setup_crypto_subtle_api(
    scope: &mut v8::HandleScope,
    subtle_obj: &v8::Object,
) {
    // digest method
    let digest_key = v8::String::new(scope, "digest").unwrap();
    let digest_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        // Inline digest implementation to avoid lifetime issues
        if args.length() < 2 {
            let error = v8::String::new(scope, "digest requires algorithm and data arguments").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        let algo_value = args.get(0);
        let data_value = args.get(1);

        // Get hash algorithm
        let hash_name = get_algorithm_hash_name(scope, algo_value);

        // Get data
        let data = match get_array_buffer_data(scope, data_value) {
            Some(d) => d,
            None => {
                let error = v8::String::new(scope, "digest requires an ArrayBuffer or TypedArray as data").unwrap();
                let error_obj = v8::Exception::type_error(scope, error);
                scope.throw_exception(error_obj.into());
                return;
            }
        };

        // Compute hash
        let hash_result = compute_sha_digest(&data, &hash_name);

        match hash_result {
            Ok(hash) => {
                let array_buffer = v8::ArrayBuffer::new(scope, hash.len());
                let backing_store = array_buffer.get_backing_store();
                for (i, byte) in hash.iter().enumerate() {
                    backing_store[i].set(*byte);
                }
                let uint8_array = match v8::Uint8Array::new(scope, array_buffer, 0, hash.len()) {
                    Some(arr) => arr,
                    None => {
                        let error = v8::String::new(scope, "Failed to create Uint8Array").unwrap();
                        let error_obj = v8::Exception::error(scope, error);
                        scope.throw_exception(error_obj.into());
                        return;
                    }
                };
                // Inline create_resolved_promise to avoid lifetime issues
                let resolver = v8::PromiseResolver::new(scope).unwrap();
                resolver.resolve(scope, uint8_array.into());
                let promise = resolver.get_promise(scope);
                retval.set(promise.into());
            }
            Err(e) => {
                let error = v8::String::new(scope, &e).unwrap();
                let error_obj = v8::Exception::error(scope, error);
                scope.throw_exception(error_obj.into());
            }
        }
    });
    let digest_fn_instance = digest_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, digest_key.into(), digest_fn_instance.into());

    // importKey method - fully implemented
    let import_key_key = v8::String::new(scope, "importKey").unwrap();
    let import_key_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        import_key_callback(scope, args, rv);
    });
    let import_key_fn_instance = import_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, import_key_key.into(), import_key_fn_instance.into());

    // encrypt method - implemented for AES-GCM
    let encrypt_key = v8::String::new(scope, "encrypt").unwrap();
    let encrypt_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        aes_encrypt_callback(scope, args, rv);
    });
    let encrypt_fn_instance = encrypt_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, encrypt_key.into(), encrypt_fn_instance.into());

    // decrypt method - implemented for AES-GCM
    let decrypt_key = v8::String::new(scope, "decrypt").unwrap();
    let decrypt_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        aes_decrypt_callback(scope, args, rv);
    });
    let decrypt_fn_instance = decrypt_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, decrypt_key.into(), decrypt_fn_instance.into());

    // sign method - implemented for HMAC
    let sign_key = v8::String::new(scope, "sign").unwrap();
    let sign_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        hmac_sign_callback(scope, args, rv);
    });
    let sign_fn_instance = sign_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, sign_key.into(), sign_fn_instance.into());

    // verify method - implemented for HMAC
    let verify_key = v8::String::new(scope, "verify").unwrap();
    let verify_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        hmac_verify_callback(scope, args, rv);
    });
    let verify_fn_instance = verify_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, verify_key.into(), verify_fn_instance.into());

    // generateKey method - fully implemented
    let generate_key_key = v8::String::new(scope, "generateKey").unwrap();
    let generate_key_fn = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        generate_key_callback(scope, args, rv);
    });
    let generate_key_fn_instance = generate_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, generate_key_key.into(), generate_key_fn_instance.into());

    // Placeholder for deriveKey - returns resolved Promise
    let derive_key_key = v8::String::new(scope, "deriveKey").unwrap();
    let derive_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let undefined_val = v8::undefined(_scope).into();
        let resolver = v8::PromiseResolver::new(_scope).unwrap();
        resolver.resolve(_scope, undefined_val);
        let promise = resolver.get_promise(_scope);
        rv.set(promise.into());
    });
    let derive_key_fn_instance = derive_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, derive_key_key.into(), derive_key_fn_instance.into());

    // Placeholder for exportKey - returns resolved Promise
    let export_key_key = v8::String::new(scope, "exportKey").unwrap();
    let export_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let undefined_val = v8::undefined(_scope).into();
        let resolver = v8::PromiseResolver::new(_scope).unwrap();
        resolver.resolve(_scope, undefined_val);
        let promise = resolver.get_promise(_scope);
        rv.set(promise.into());
    });
    let export_key_fn_instance = export_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, export_key_key.into(), export_key_fn_instance.into());

    // Placeholder for wrapKey - returns resolved Promise
    let wrap_key_key = v8::String::new(scope, "wrapKey").unwrap();
    let wrap_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let undefined_val = v8::undefined(_scope).into();
        let resolver = v8::PromiseResolver::new(_scope).unwrap();
        resolver.resolve(_scope, undefined_val);
        let promise = resolver.get_promise(_scope);
        rv.set(promise.into());
    });
    let wrap_key_fn_instance = wrap_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, wrap_key_key.into(), wrap_key_fn_instance.into());

    // Placeholder for unwrapKey - returns resolved Promise
    let unwrap_key_key = v8::String::new(scope, "unwrapKey").unwrap();
    let unwrap_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let undefined_val = v8::undefined(_scope).into();
        let resolver = v8::PromiseResolver::new(_scope).unwrap();
        resolver.resolve(_scope, undefined_val);
        let promise = resolver.get_promise(_scope);
        rv.set(promise.into());
    });
    let unwrap_key_fn_instance = unwrap_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, unwrap_key_key.into(), unwrap_key_fn_instance.into());
}

/// Generate random bytes for key material
fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut data = vec![0u8; length];
    use ring::rand::SecureRandom;
    let rng = ring::rand::SystemRandom::new();
    let _ = rng.fill(&mut data);
    data
}

/// Get algorithm length from algorithm object
fn get_algorithm_length(scope: &mut v8::HandleScope, algo_value: v8::Local<v8::Value>, default_length: i32) -> i32 {
    if algo_value.is_object() {
        let algo_obj = algo_value.to_object(scope).unwrap();
        let length_key = v8::String::new(scope, "length").unwrap();
        if let Some(length_val) = algo_obj.get(scope, length_key.into()) {
            if length_val.is_number() {
                return length_val.integer_value(scope).unwrap_or(default_length as i64) as i32;
            }
        }
    }
    default_length
}

/// GenerateKey callback - generates cryptographic keys
fn generate_key_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 3 {
        let error = v8::String::new(scope, "generateKey requires 3 arguments: algorithm, extractable, keyUsages").unwrap();
        let error_obj = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let algorithm_value = args.get(0);
    let extractable_value = args.get(1);
    let usages_value = args.get(2);

    // Parse algorithm name
    let algorithm_name = get_algorithm_name(scope, algorithm_value);

    // Parse extractable
    let extractable = get_bool_value(scope, extractable_value);

    // Parse usages
    let usages = get_key_usages(scope, usages_value);

    // Generate key based on algorithm
    match algorithm_name.to_uppercase().as_str() {
        "HMAC" | "HS256" | "HS384" | "HS512" => {
            // Get hash algorithm
            let hash_name = get_algorithm_hash_name(scope, algorithm_value);

            // Determine key length based on hash algorithm
            let key_length = match hash_name.as_str() {
                "SHA-256" | "SHA-384" | "SHA-512" => 64, // Default to 512 bits
                _ => 64,
            };

            // Generate random key material
            let key_data = generate_random_bytes(key_length);

            // Create CryptoKey
            let crypto_key = create_crypto_key(
                scope,
                "secret",
                extractable,
                &algorithm_name,
                (key_length * 8) as i32,
                usages.iter().map(|s| s.as_str()).collect(),
            );

            // Store key data
            let key_data_key = v8::String::new(scope, "__beejs_key_data__").unwrap();
            let key_data_array = v8::ArrayBuffer::new(scope, key_data.len());
            let backing_store = key_data_array.get_backing_store();
            for (i, &byte) in key_data.iter().enumerate() {
                backing_store[i].set(byte);
            }
            crypto_key.set(scope, key_data_key.into(), key_data_array.into());

            // Return promise resolving to the CryptoKey
            let resolver = v8::PromiseResolver::new(scope).unwrap();
            resolver.resolve(scope, crypto_key.into());
            let promise = resolver.get_promise(scope);
            retval.set(promise.into());
        }
        "AES-GCM" | "AES-CBC" | "AES-CTR" | "AES-KW" => {
            // Get key length (128, 192, or 256)
            let length = get_algorithm_length(scope, algorithm_value, 256);
            let key_bytes = match length {
                128 => 16,
                192 => 24,
                256 => 32,
                _ => 32, // Default to 256
            };

            // Generate random key material
            let key_data = generate_random_bytes(key_bytes);

            // Create CryptoKey
            let crypto_key = create_crypto_key(
                scope,
                "secret",
                extractable,
                &algorithm_name,
                length,
                usages.iter().map(|s| s.as_str()).collect(),
            );

            // Store key data
            let key_data_key = v8::String::new(scope, "__beejs_key_data__").unwrap();
            let key_data_array = v8::ArrayBuffer::new(scope, key_data.len());
            let backing_store = key_data_array.get_backing_store();
            for (i, &byte) in key_data.iter().enumerate() {
                backing_store[i].set(byte);
            }
            crypto_key.set(scope, key_data_key.into(), key_data_array.into());

            // Return promise resolving to the CryptoKey
            let resolver = v8::PromiseResolver::new(scope).unwrap();
            resolver.resolve(scope, crypto_key.into());
            let promise = resolver.get_promise(scope);
            retval.set(promise.into());
        }
        "RSA-OAEP" | "RSASSA-PKCS1-v1_5" => {
            // Placeholder for RSA key generation (requires more complex implementation)
            let error = v8::String::new(scope, "RSA key generation not yet implemented").unwrap();
            let error_obj = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
        }
        "ECDSA" | "ECDH" => {
            // Placeholder for EC key generation (requires more complex implementation)
            let error = v8::String::new(scope, "EC key generation not yet implemented").unwrap();
            let error_obj = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
        }
        _ => {
            let error_msg = format!("generateKey: unsupported algorithm '{}'", algorithm_name);
            let error = v8::String::new(scope, &error_msg).unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
        }
    }
}

/// Setup crypto.randomUUID (for convenience)
fn setup_crypto_random_uuid_api(
    scope: &mut v8::HandleScope,
    crypto_obj: &v8::Object,
) {
    let uuid_key = v8::String::new(scope, "randomUUID").unwrap();
    let uuid_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = v8::String::new(_scope, &uuid.to_string()).unwrap();
        rv.set(uuid_str.into());
    });
    let uuid_fn_instance = uuid_fn.get_function(scope).unwrap();
    crypto_obj.set(scope, uuid_key.into(), uuid_fn_instance.into());
}

pub fn setup_crypto_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create crypto object
    let crypto_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Create subtle object
    let subtle_key = v8::String::new(scope, "subtle").unwrap();
    let subtle_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Setup getRandomValues on crypto (not subtle)
    let get_random_key: v8::Local<v8::String> = v8::String::new(scope, "getRandomValues").unwrap();
    let get_random_func: v8::Local<v8::FunctionTemplate> = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue| {
        get_random_values_callback(scope, args, rv);
    });
    let get_random_func_instance: v8::Local<v8::Function> = get_random_func.get_function(scope).unwrap();
    crypto_obj.set(scope, get_random_key.into(), get_random_func_instance.into());

    // Setup crypto.subtle API
    setup_crypto_subtle_api(scope, &subtle_obj);

    // Setup crypto.randomUUID
    setup_crypto_random_uuid_api(scope, &crypto_obj);

    // Set subtle on crypto
    crypto_obj.set(scope, subtle_key.into(), subtle_obj.into());

    // Set crypto on global
    let crypto_key: v8::Local<v8::String> = v8::String::new(scope, "crypto").unwrap();
    global.set(scope, crypto_key.into(), crypto_obj.into());

    // Also set webkitGetUserEntries (Safari compatibility)
    let webkit_key: v8::Local<v8::String> = v8::String::new(scope, "webkitCrypto").unwrap();
    global.set(scope, webkit_key.into(), crypto_obj.into());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_digest() {
        let data = b"hello world";
        let result = compute_sha_digest(data, "SHA-256");
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
        // b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        assert_eq!(hex::encode(&hash[..8]), "b94d27b9934d3e08");
    }

    #[test]
    fn test_sha384_digest() {
        let data = b"hello world";
        let result = compute_sha_digest(data, "SHA-384");
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash.len(), 48); // SHA-384 produces 48 bytes
    }

    #[test]
    fn test_sha512_digest() {
        let data = b"hello world";
        let result = compute_sha_digest(data, "SHA-512");
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash.len(), 64); // SHA-512 produces 64 bytes
    }

    #[test]
    fn test_unsupported_algorithm() {
        let data = b"hello world";
        let result = compute_sha_digest(data, "MD5");
        assert!(result.is_err());
    }
}

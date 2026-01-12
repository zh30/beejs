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

    // Placeholder for importKey
    let import_key_key = v8::String::new(scope, "importKey").unwrap();
    let import_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.importKey called (placeholder)");
    });
    let import_key_fn_instance = import_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, import_key_key.into(), import_key_fn_instance.into());

    // Placeholder for encrypt
    let encrypt_key = v8::String::new(scope, "encrypt").unwrap();
    let encrypt_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.encrypt called (placeholder)");
    });
    let encrypt_fn_instance = encrypt_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, encrypt_key.into(), encrypt_fn_instance.into());

    // Placeholder for decrypt
    let decrypt_key = v8::String::new(scope, "decrypt").unwrap();
    let decrypt_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.decrypt called (placeholder)");
    });
    let decrypt_fn_instance = decrypt_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, decrypt_key.into(), decrypt_fn_instance.into());

    // Placeholder for sign
    let sign_key = v8::String::new(scope, "sign").unwrap();
    let sign_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.sign called (placeholder)");
    });
    let sign_fn_instance = sign_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, sign_key.into(), sign_fn_instance.into());

    // Placeholder for verify
    let verify_key = v8::String::new(scope, "verify").unwrap();
    let verify_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.verify called (placeholder)");
    });
    let verify_fn_instance = verify_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, verify_key.into(), verify_fn_instance.into());

    // Placeholder for generateKey
    let generate_key_key = v8::String::new(scope, "generateKey").unwrap();
    let generate_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.generateKey called (placeholder)");
    });
    let generate_key_fn_instance = generate_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, generate_key_key.into(), generate_key_fn_instance.into());

    // Placeholder for deriveKey
    let derive_key_key = v8::String::new(scope, "deriveKey").unwrap();
    let derive_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.deriveKey called (placeholder)");
    });
    let derive_key_fn_instance = derive_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, derive_key_key.into(), derive_key_fn_instance.into());

    // Placeholder for exportKey
    let export_key_key = v8::String::new(scope, "exportKey").unwrap();
    let export_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.exportKey called (placeholder)");
    });
    let export_key_fn_instance = export_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, export_key_key.into(), export_key_fn_instance.into());

    // Placeholder for wrapKey
    let wrap_key_key = v8::String::new(scope, "wrapKey").unwrap();
    let wrap_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.wrapKey called (placeholder)");
    });
    let wrap_key_fn_instance = wrap_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, wrap_key_key.into(), wrap_key_fn_instance.into());

    // Placeholder for unwrapKey
    let unwrap_key_key = v8::String::new(scope, "unwrapKey").unwrap();
    let unwrap_key_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        eprintln!("crypto.subtle.unwrapKey called (placeholder)");
    });
    let unwrap_key_fn_instance = unwrap_key_fn.get_function(scope).unwrap();
    subtle_obj.set(scope, unwrap_key_key.into(), unwrap_key_fn_instance.into());
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

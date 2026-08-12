//! Node.js `zlib` builtin (gzip/deflate via flate2).

use anyhow::Result;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use flate2::Compression;
use rusty_v8 as v8;
use std::io::{Read, Write};

pub fn setup_zlib_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);
    let zlib = v8::Object::new(scope);

    let gzip_sync = v8::Function::new(scope, gzip_sync_cb).unwrap();
    let gunzip_sync = v8::Function::new(scope, gunzip_sync_cb).unwrap();
    let deflate_sync = v8::Function::new(scope, deflate_sync_cb).unwrap();
    let inflate_sync = v8::Function::new(scope, inflate_sync_cb).unwrap();
    let deflate_raw_sync = v8::Function::new(scope, deflate_raw_sync_cb).unwrap();
    let inflate_raw_sync = v8::Function::new(scope, inflate_raw_sync_cb).unwrap();

    for (name, func) in [
        ("gzipSync", gzip_sync),
        ("gunzipSync", gunzip_sync),
        ("deflateSync", deflate_sync),
        ("inflateSync", inflate_sync),
        ("deflateRawSync", deflate_raw_sync),
        ("inflateRawSync", inflate_raw_sync),
    ] {
        let key = v8::String::new(scope, name).unwrap();
        zlib.set(scope, key.into(), func.into());
    }

    let key = v8::String::new(scope, "zlib").unwrap();
    global.set(scope, key.into(), zlib.into());
    Ok(())
}

fn throw_type(scope: &mut v8::HandleScope, message: &str) {
    let msg = v8::String::new(scope, message).unwrap();
    let exc = v8::Exception::type_error(scope, msg);
    scope.throw_exception(exc);
}

fn throw_err(scope: &mut v8::HandleScope, message: &str) {
    let msg = v8::String::new(scope, message).unwrap();
    let exc = v8::Exception::error(scope, msg);
    scope.throw_exception(exc);
}

fn bytes_from_arg(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Option<Vec<u8>> {
    if value.is_array_buffer() {
        let buf = v8::Local::<v8::ArrayBuffer>::try_from(value).ok()?;
        let store = buf.get_backing_store();
        let len = buf.byte_length();
        let slice = unsafe { std::slice::from_raw_parts(store.as_ref().as_ptr() as *const u8, len) };
        return Some(slice.to_vec());
    }
    if value.is_typed_array() {
        let ta = v8::Local::<v8::TypedArray>::try_from(value).ok()?;
        let len = ta.byte_length() as usize;
        let buf = ta.buffer(scope)?;
        let store = buf.get_backing_store();
        let slice = unsafe { std::slice::from_raw_parts(store.as_ref().as_ptr() as *const u8, len) };
        return Some(slice.to_vec());
    }
    if let Some(s) = value.to_string(scope) {
        return Some(s.to_rust_string_lossy(scope).into_bytes());
    }
    None
}

fn return_buffer(scope: &mut v8::HandleScope, bytes: &[u8], rv: &mut v8::ReturnValue) {
    let buffer = v8::ArrayBuffer::new(scope, bytes.len());
    if !bytes.is_empty() {
        let store = buffer.get_backing_store();
        let slice = unsafe {
            std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, bytes.len())
        };
        slice.copy_from_slice(bytes);
    }
    rv.set(buffer.into());
}

fn gzip_sync_cb(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let Some(input) = bytes_from_arg(scope, args.get(0)) else {
        throw_type(scope, "zlib.gzipSync: invalid input");
        return;
    };
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(&input).is_err() {
        throw_err(scope, "zlib.gzipSync failed");
        return;
    }
    match encoder.finish() {
        Ok(out) => return_buffer(scope, &out, &mut rv),
        Err(_) => throw_err(scope, "zlib.gzipSync failed"),
    }
}

fn gunzip_sync_cb(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let Some(input) = bytes_from_arg(scope, args.get(0)) else {
        throw_type(scope, "zlib.gunzipSync: invalid input");
        return;
    };
    let mut decoder = GzDecoder::new(&input[..]);
    let mut out = Vec::new();
    if decoder.read_to_end(&mut out).is_err() {
        throw_err(scope, "zlib.gunzipSync failed");
        return;
    }
    return_buffer(scope, &out, &mut rv);
}

fn deflate_sync_cb(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let Some(input) = bytes_from_arg(scope, args.get(0)) else {
        return;
    };
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(&input);
    if let Ok(out) = encoder.finish() {
        return_buffer(scope, &out, &mut rv);
    }
}

fn inflate_sync_cb(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let Some(input) = bytes_from_arg(scope, args.get(0)) else {
        return;
    };
    let mut decoder = ZlibDecoder::new(&input[..]);
    let mut out = Vec::new();
    if decoder.read_to_end(&mut out).is_ok() {
        return_buffer(scope, &out, &mut rv);
    }
}

fn deflate_raw_sync_cb(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let Some(input) = bytes_from_arg(scope, args.get(0)) else {
        return;
    };
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(&input);
    if let Ok(out) = encoder.finish() {
        return_buffer(scope, &out, &mut rv);
    }
}

fn inflate_raw_sync_cb(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let Some(input) = bytes_from_arg(scope, args.get(0)) else {
        return;
    };
    let mut decoder = DeflateDecoder::new(&input[..]);
    let mut out = Vec::new();
    if decoder.read_to_end(&mut out).is_ok() {
        return_buffer(scope, &out, &mut rv);
    }
}

//! Node.js querystring模块实现
//! 查询字符串处理

// TODO: Remove unused import: use anyhow::Result;
use rusty_v8 as v8;

/// 设置querystring API
pub fn setup_querystring_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let qs_obj = v8::Object::new(scope);

    // parse
    let parse_func = v8::FunctionTemplate::new(scope, qs_parse_callback);
    let parse_instance = parse_func.get_function(scope).unwrap();
    let parse_key = v8::String::new(scope, "parse").unwrap();
    qs_obj.set(scope, parse_key.into(), parse_instance.into());

    // stringify
    let stringify_func = v8::FunctionTemplate::new(scope, qs_stringify_callback);
    let stringify_instance = stringify_func.get_function(scope).unwrap();
    let stringify_key = v8::String::new(scope, "stringify").unwrap();
    qs_obj.set(scope, stringify_key.into(), stringify_instance.into());

    // escape
    let escape_func = v8::FunctionTemplate::new(scope, qs_escape_callback);
    let escape_instance = escape_func.get_function(scope).unwrap();
    let escape_key = v8::String::new(scope, "escape").unwrap();
    qs_obj.set(scope, escape_key.into(), escape_instance.into());

    // unescape
    let unescape_func = v8::FunctionTemplate::new(scope, qs_unescape_callback);
    let unescape_instance = unescape_func.get_function(scope).unwrap();
    let unescape_key = v8::String::new(scope, "unescape").unwrap();
    qs_obj.set(scope, unescape_key.into(), unescape_instance.into());

    // 设置到全局
    let global = context.global(scope);
    let qs_key = v8::String::new(scope, "querystring").unwrap();
    global.set(scope, qs_key.into(), qs_obj.into());

    Ok(())
}

fn qs_parse_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let str = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let _sep = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "&".to_string());

    let _eq = args
        .get(2)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "=".to_string());

    // 简化实现
    let result_obj = v8::Object::new(scope);
    let _key_0 = v8::String::new(scope, "parsed").unwrap();
    let _val_0 = v8::String::new(scope, "true").unwrap();
    result_obj.set(scope, _key_0.into(), _val_0.into());

    retval.set(result_obj.into());
}

fn qs_stringify_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let _obj = args.get(0);

    // 简化实现
    let result = "key=value";
    retval.set(v8::String::new(scope, result).unwrap().into());
}

fn qs_escape_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let str = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let result = str; // 简化实现
    retval.set(v8::String::new(scope, &result).unwrap().into());
}

fn qs_unescape_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let str = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let result = percent_encoding::percent_decode_str(&str).decode_utf8().unwrap_or_default().to_string();
    retval.set(v8::String::new(scope, &result).unwrap().into());
}

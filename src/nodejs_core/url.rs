//! Node.js URL模块实现
//! WHATWG URL标准支持

// TODO: Remove unused import: use anyhow::Result;
use rusty_v8 as v8;
// TODO: Remove unused import: use std::collections::HashMap;

/// 设置URL API
pub fn setup_url_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // URL构造函数
    let url_constructor = v8::FunctionTemplate::new(scope, url_constructor_callback);
    let url_func = url_constructor.get_function(scope).unwrap();
    let url_key = v8::String::new(scope, "URL").unwrap();
    global.set(scope, url_key.into(), url_func.into());

    // URLSearchParams构造函数
    let search_params_constructor = v8::FunctionTemplate::new(scope, search_params_constructor_callback);
    let search_params_func = search_params_constructor.get_function(scope).unwrap();
    let search_params_key = v8::String::new(scope, "URLSearchParams").unwrap();
    global.set(scope, search_params_key.into(), search_params_func.into());

    // url对象
    let url_obj = v8::Object::new(scope);

    // url.parse
    let parse_func = v8::FunctionTemplate::new(scope, url_parse_callback);
    let parse_instance = parse_func.get_function(scope).unwrap();
    let parse_key = v8::String::new(scope, "parse").unwrap();
    url_obj.set(scope, parse_key.into(), parse_instance.into());

    // url.format
    let format_func = v8::FunctionTemplate::new(scope, url_format_callback);
    let format_instance = format_func.get_function(scope).unwrap();
    let format_key = v8::String::new(scope, "format").unwrap();
    url_obj.set(scope, format_key.into(), format_instance.into());

    // url.resolve
    let resolve_func = v8::FunctionTemplate::new(scope, url_resolve_callback);
    let resolve_instance = resolve_func.get_function(scope).unwrap();
    let resolve_key = v8::String::new(scope, "resolve").unwrap();
    url_obj.set(scope, resolve_key.into(), resolve_instance.into());

    let url_module_key = v8::String::new(scope, "url").unwrap();
    global.set(scope, url_module_key.into(), url_obj.into());

    Ok(())
}

fn url_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let input = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let base = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let url_obj = v8::Object::new(scope);

    // 解析URL
    let parsed_url = parse_url_string(&input, &base);

    // 设置URL属性
    if let Some(parts) = parsed_url {
        let key_href = v8::String::new(scope, "href").unwrap();
        let val_href = v8::String::new(scope, &parts.href).unwrap();
        url_obj.set(scope, key_href.into(), val_href.into());
        let key_protocol = v8::String::new(scope, "protocol").unwrap();
        let val_protocol = v8::String::new(scope, &parts.protocol).unwrap();
        url_obj.set(scope, key_protocol.into(), val_protocol.into());
        let key_hostname = v8::String::new(scope, "hostname").unwrap();
        let val_hostname = v8::String::new(scope, &parts.hostname).unwrap();
        url_obj.set(scope, key_hostname.into(), val_hostname.into());
        let key_port = v8::String::new(scope, "port").unwrap();
        let val_port = v8::String::new(scope, &parts.port).unwrap();
        url_obj.set(scope, key_port.into(), val_port.into());
        let key_pathname = v8::String::new(scope, "pathname").unwrap();
        let val_pathname = v8::String::new(scope, &parts.pathname).unwrap();
        url_obj.set(scope, key_pathname.into(), val_pathname.into());
        let key_search = v8::String::new(scope, "search").unwrap();
        let val_search = v8::String::new(scope, &parts.search).unwrap();
        url_obj.set(scope, key_search.into(), val_search.into());
        let key_hash = v8::String::new(scope, "hash").unwrap();
        let val_hash = v8::String::new(scope, &parts.hash).unwrap();
        url_obj.set(scope, key_hash.into(), val_hash.into());
        let key_host = v8::String::new(scope, "host").unwrap();
        let val_host = v8::String::new(scope, &parts.host).unwrap();
        url_obj.set(scope, key_host.into(), val_host.into());
    }

    // toString方法
    let to_string_func = v8::FunctionTemplate::new(scope, url_to_string_callback);
    let to_string_instance = to_string_func.get_function(scope).unwrap();
    let to_string_key = v8::String::new(scope, "toString").unwrap();
    url_obj.set(scope, to_string_key.into(), to_string_instance.into());

    // toJSON方法
    let to_json_func = v8::FunctionTemplate::new(scope, url_to_json_callback);
    let to_json_instance = to_json_func.get_function(scope).unwrap();
    let to_json_key = v8::String::new(scope, "toJSON").unwrap();
    url_obj.set(scope, to_json_key.into(), to_json_instance.into());

    // origin属性
    let origin_key = v8::String::new(scope, "origin").unwrap();
    let origin_val = v8::String::new(scope, "").unwrap();
    url_obj.set(scope, origin_key.into(), origin_val.into());

    retval.set(url_obj.into());
}

fn url_to_string_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let href_key = v8::String::new(scope, "href").unwrap();
    let href = this
        .get(scope, href_key.into())
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_default();

    retval.set(v8::String::new(scope, &href).unwrap().into());
}

fn url_to_json_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let href_key = v8::String::new(scope, "href").unwrap();
    let href = this
        .get(scope, href_key.into())
        .unwrap_or(v8::String::new(scope, "").unwrap().into());

    retval.set(href);
}

fn search_params_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let init = args.get(0);

    let params_obj = v8::Object::new(scope);

    // get方法
    let get_func = v8::FunctionTemplate::new(scope, search_params_get_callback);
    let get_instance = get_func.get_function(scope).unwrap();
    let get_key = v8::String::new(scope, "get").unwrap();
    params_obj.set(scope, get_key.into(), get_instance.into());

    // set方法
    let set_func = v8::FunctionTemplate::new(scope, search_params_set_callback);
    let set_instance = set_func.get_function(scope).unwrap();
    let set_key = v8::String::new(scope, "set").unwrap();
    params_obj.set(scope, set_key.into(), set_instance.into());

    // append方法
    let append_func = v8::FunctionTemplate::new(scope, search_params_append_callback);
    let append_instance = append_func.get_function(scope).unwrap();
    let append_key = v8::String::new(scope, "append").unwrap();
    params_obj.set(scope, append_key.into(), append_instance.into());

    // delete方法
    let delete_func = v8::FunctionTemplate::new(scope, search_params_delete_callback);
    let delete_instance = delete_func.get_function(scope).unwrap();
    let delete_key = v8::String::new(scope, "delete").unwrap();
    params_obj.set(scope, delete_key.into(), delete_instance.into());

    // has方法
    let has_func = v8::FunctionTemplate::new(scope, search_params_has_callback);
    let has_instance = has_func.get_function(scope).unwrap();
    let has_key = v8::String::new(scope, "has").unwrap();
    params_obj.set(scope, has_key.into(), has_instance.into());

    // keys方法
    let keys_func = v8::FunctionTemplate::new(scope, search_params_keys_callback);
    let keys_instance = keys_func.get_function(scope).unwrap();
    let keys_key = v8::String::new(scope, "keys").unwrap();
    params_obj.set(scope, keys_key.into(), keys_instance.into());

    // values方法
    let values_func = v8::FunctionTemplate::new(scope, search_params_values_callback);
    let values_instance = values_func.get_function(scope).unwrap();
    let values_key = v8::String::new(scope, "values").unwrap();
    params_obj.set(scope, values_key.into(), values_instance.into());

    // entries方法
    let entries_func = v8::FunctionTemplate::new(scope, search_params_entries_callback);
    let entries_instance = entries_func.get_function(scope).unwrap();
    let entries_key = v8::String::new(scope, "entries").unwrap();
    params_obj.set(scope, entries_key.into(), entries_instance.into());

    // toString方法
    let to_string_func = v8::FunctionTemplate::new(scope, search_params_to_string_callback);
    let to_string_instance = to_string_func.get_function(scope).unwrap();
    let to_string_key = v8::String::new(scope, "toString").unwrap();
    params_obj.set(scope, to_string_key.into(), to_string_instance.into());

    // _params存储
    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = v8::Array::new(scope, 0);
    params_obj.set(scope, params_key.into(), params_array.into());

    // 初始化参数
    if init.is_string() {
        let query_string = init.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let pairs = parse_query_string(&query_string);
        let params_array = v8::Array::new(scope, pairs.len() as i32);
        for (i, (key, value)) in pairs.into_iter().enumerate() {
            let pair_array = v8::Array::new(scope, 2);
            let val_0 = v8::String::new(scope, &key).unwrap().into();
            pair_array.set_index(scope, 0, val_0);
            let val_1 = v8::String::new(scope, &value).unwrap().into();
            pair_array.set_index(scope, 1, val_1);
            params_array.set_index(scope, i as u32, pair_array.into());
        }
        params_obj.set(scope, params_key.into(), params_array.into());
    }

    retval.set(params_obj.into());
}

fn search_params_get_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();
    let mut result: v8::Local<v8::Value> = v8::null(scope).into();

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let key = pair.get_index(scope, 0).unwrap();
                        let value = pair.get_index(scope, 1).unwrap();
                        if key.to_string(scope).unwrap().to_rust_string_lossy(scope) == name {
                            result = value;
                            break;
                        }
                    }
                }
            }
        }
    }

    retval.set(result);
}

fn search_params_set_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let value = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            let mut found = false;
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let key = pair.get_index(scope, 0).unwrap();
                        if key.to_string(scope).unwrap().to_rust_string_lossy(scope) == name {
                            let new_value = v8::String::new(scope, &value).unwrap().into();
                            pair.set_index(scope, 1, new_value);
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                let pair_array = v8::Array::new(scope, 2);
                let name_val = v8::String::new(scope, &name).unwrap().into();
                pair_array.set_index(scope, 0, name_val);
                let value_val = v8::String::new(scope, &value).unwrap().into();
                pair_array.set_index(scope, 1, value_val);
                let length = arr.length();
                arr.set_index(scope, length, pair_array.into());
            }
        }
    }

    retval.set(this.into());
}

fn search_params_append_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let value = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            let pair_array = v8::Array::new(scope, 2);
            let val_0 = v8::String::new(scope, &name).unwrap().into();
            pair_array.set_index(scope, 0, val_0);
            let val_1 = v8::String::new(scope, &value).unwrap().into();
            pair_array.set_index(scope, 1, val_1);
            let length = arr.length();
            arr.set_index(scope, length, pair_array.into());
        }
    }

    retval.set(this.into());
}

fn search_params_delete_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            let mut new_arr = v8::Array::new(scope, 0);
            let mut new_index = 0;
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let key = pair.get_index(scope, 0).unwrap();
                        if key.to_string(scope).unwrap().to_rust_string_lossy(scope) != name {
                            new_arr.set_index(scope, new_index, pair.into());
                            new_index += 1;
                        }
                    }
                }
            }
            this.set(scope, params_key.into(), new_arr.into());
        }
    }

    retval.set(this.into());
}

fn search_params_has_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();
    let mut has = false;

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let key = pair.get_index(scope, 0).unwrap();
                        if key.to_string(scope).unwrap().to_rust_string_lossy(scope) == name {
                            has = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    retval.set(v8::Boolean::new(scope, has).into());
}

fn search_params_keys_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();
    let keys_array = v8::Array::new(scope, 0);

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            let mut key_index = 0;
            let mut seen_keys = std::collections::HashSet::new();
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let key = pair.get_index(scope, 0).unwrap();
                        let key_str = key.to_string(scope).unwrap().to_rust_string_lossy(scope);
                        if seen_keys.insert(key_str.clone()) {
                            let _val_0 = v8::String::new(scope, &key_str).unwrap();
                            keys_array.set_index(scope, key_index, _val_0.into());
                            key_index += 1;
                        }
                    }
                }
            }
        }
    }

    retval.set(keys_array.into());
}

fn search_params_values_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();
    let values_array = v8::Array::new(scope, 0);

    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            let mut value_index = 0;
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let value = pair.get_index(scope, 1).unwrap();
                        values_array.set_index(scope, value_index, value);
                        value_index += 1;
                    }
                }
            }
        }
    }

    retval.set(values_array.into());
}

fn search_params_entries_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();

    retval.set(params_array);
}

fn search_params_to_string_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let params_key = v8::String::new(scope, "_params").unwrap();
    let params_array = this.get(scope, params_key.into()).unwrap();

    let mut query_string = String::new();
    if params_array.is_array() {
        if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
            for i in 0..arr.length() {
                let v = arr.get_index(scope, i).unwrap();
                if let Some(pair) = if v.is_array() {
                    v8::Local::<v8::Array>::try_from(v).ok()
                } else {
                    None
                } {
                    if pair.length() >= 2 {
                        let key = pair.get_index(scope, 0).unwrap();
                        let value = pair.get_index(scope, 1).unwrap();
                        let key_str = key.to_string(scope).unwrap().to_rust_string_lossy(scope);
                        let value_str = value.to_string(scope).unwrap().to_rust_string_lossy(scope);

                        if !query_string.is_empty() {
                            query_string.push('&');
                        }
                        query_string.push_str(&format!("{}={}", key_str, value_str));
                    }
                }
            }
        }
    }

    retval.set(v8::String::new(scope, &query_string).unwrap().into());
}

// 旧版URL API兼容函数
fn url_parse_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let url_str = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let parsed_url = parse_url_string(&url_str, "");

    let url_obj = v8::Object::new(scope);
    if let Some(parts) = parsed_url {
        let key_href = v8::String::new(scope, "href").unwrap();
        let val_href = v8::String::new(scope, &parts.href).unwrap();
        url_obj.set(scope, key_href.into(), val_href.into());
        let key_protocol = v8::String::new(scope, "protocol").unwrap();
        let val_protocol = v8::String::new(scope, &parts.protocol).unwrap();
        url_obj.set(scope, key_protocol.into(), val_protocol.into());
        let key_hostname = v8::String::new(scope, "hostname").unwrap();
        let val_hostname = v8::String::new(scope, &parts.hostname).unwrap();
        url_obj.set(scope, key_hostname.into(), val_hostname.into());
        let key_port = v8::String::new(scope, "port").unwrap();
        let val_port = v8::String::new(scope, &parts.port).unwrap();
        url_obj.set(scope, key_port.into(), val_port.into());
        let key_pathname = v8::String::new(scope, "pathname").unwrap();
        let val_pathname = v8::String::new(scope, &parts.pathname).unwrap();
        url_obj.set(scope, key_pathname.into(), val_pathname.into());
        let key_search = v8::String::new(scope, "search").unwrap();
        let val_search = v8::String::new(scope, &parts.search).unwrap();
        url_obj.set(scope, key_search.into(), val_search.into());
        let key_hash = v8::String::new(scope, "hash").unwrap();
        let val_hash = v8::String::new(scope, &parts.hash).unwrap();
        url_obj.set(scope, key_hash.into(), val_hash.into());
        let key_host = v8::String::new(scope, "host").unwrap();
        let val_host = v8::String::new(scope, &parts.host).unwrap();
        url_obj.set(scope, key_host.into(), val_host.into());
    }

    retval.set(url_obj.into());
}

fn url_format_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let url_obj = args.get(0);
    let mut href = String::new();

    if let Some(obj) = url_obj.to_object(scope) {
        let protocol_key = v8::String::new(scope, "protocol").unwrap();
        let protocol = obj.get(scope, protocol_key.into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let hostname_key = v8::String::new(scope, "hostname").unwrap();
        let hostname = obj.get(scope, hostname_key.into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        let pathname_key = v8::String::new(scope, "pathname").unwrap();
        let pathname = obj.get(scope, pathname_key.into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();

        href = format!("{}{}{}", protocol, hostname, pathname);
    }

    retval.set(v8::String::new(scope, &href).unwrap().into());
}

fn url_resolve_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let from = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let to = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let result = if to.starts_with('/') {
        // 绝对路径
        format!("{}{}", from, to)
    } else {
        // 相对路径
        format!("{}/{}", from.trim_end_matches('/'), to)
    };

    retval.set(v8::String::new(scope, &result).unwrap().into());
}

// 辅助数据结构
struct ParsedUrl {
    href: String,
    protocol: String,
    hostname: String,
    port: String,
    pathname: String,
    search: String,
    hash: String,
    host: String,
}

// URL解析辅助函数
fn parse_url_string(url: &str, base: &str) -> Option<ParsedUrl> {
    // 简化的URL解析实现
    let mut parts = ParsedUrl {
        href: url.to_string(),
        protocol: "".to_string(),
        hostname: "".to_string(),
        port: "".to_string(),
        pathname: "".to_string(),
        search: "".to_string(),
        hash: "".to_string(),
        host: "".to_string(),
    };

    // 提取协议
    if let Some(colon_pos) = url.find("://") {
        parts.protocol = format!("{}:", &url[..colon_pos + 1]);
        let remainder = &url[colon_pos + 3..];

        // 提取主机和路径
        if let Some(slash_pos) = remainder.find('/') {
            let host_part = &remainder[..slash_pos];
            let path_part = &remainder[slash_pos..];

            parts.hostname = host_part.to_string();
            parts.host = host_part.to_string();
            parts.pathname = path_part.to_string();
        } else {
            parts.hostname = remainder.to_string();
            parts.host = remainder.to_string();
            parts.pathname = "/".to_string();
        }
    }

    Some(parts)
}

// 查询字符串解析
fn parse_query_string(query: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let parts = query.split('&');

    for part in parts {
        if let Some(eq_pos) = part.find('=') {
            let key = urlencoding::decode(&part[..eq_pos]).unwrap_or_default().to_string();
            let value = urlencoding::decode(&part[eq_pos + 1..]).unwrap_or_default().to_string();
            pairs.push((key, value));
        }
    }

    pairs
}

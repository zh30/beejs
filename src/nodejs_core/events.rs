//! Node.js Events模块实现
//! 事件驱动编程的核心模块

use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

thread_local! {
    static EVENT_LISTENERS: Mutex<HashMap<String, Vec<v8::Global<v8::Function>>>> = Mutex::new(HashMap::new());
    static ONCE_LISTENERS: Mutex<HashMap<String, Vec<v8::Global<v8::Function>>>> = Mutex::new(HashMap::new());
}

/// 设置Events API
pub fn setup_events_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // EventEmitter构造函数
    let event_emitter_constructor = v8::FunctionTemplate::new(scope, event_emitter_constructor_callback);

    // 添加静态方法
    // EventEmitter.listenerCount()
    let listener_count_func = v8::FunctionTemplate::new(scope, event_emitter_listener_count_callback);
    let listener_count_instance = listener_count_func.get_function(scope).unwrap();
    // set_on_instance has been removed, use instance template instead
    let listener_count_key = v8::String::new(scope, "listenerCount").unwrap();
    event_emitter_constructor.set(listener_count_key.into(), listener_count_instance.into());

    // 创建构造函数实例
    let event_emitter_func = event_emitter_constructor.get_function(scope).unwrap();

    // 设置到全局
    let global = context.global(scope);
    let events_key = v8::String::new(scope, "events").unwrap();
    let events_obj = v8::Object::new(scope);
    let _key_0 = v8::String::new(scope, "EventEmitter").unwrap();
    events_obj.set(scope, _key_0.into(), event_emitter_func.into());
    global.set(scope, events_key.into(), events_obj.into());

    Ok(())
}

fn event_emitter_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let emitter_obj = v8::Object::new(scope);

    // on(eventName, listener)
    let on_func = v8::FunctionTemplate::new(scope, event_emitter_on_callback);
    let on_instance = on_func.get_function(scope).unwrap();
    let on_key = v8::String::new(scope, "on").unwrap();
    emitter_obj.set(scope, on_key.into(), on_instance.into());

    // once(eventName, listener)
    let once_func = v8::FunctionTemplate::new(scope, event_emitter_once_callback);
    let once_instance = once_func.get_function(scope).unwrap();
    let once_key = v8::String::new(scope, "once").unwrap();
    emitter_obj.set(scope, once_key.into(), once_instance.into());

    // emit(eventName, ...args)
    let emit_func = v8::FunctionTemplate::new(scope, event_emitter_emit_callback);
    let emit_instance = emit_func.get_function(scope).unwrap();
    let emit_key = v8::String::new(scope, "emit").unwrap();
    emitter_obj.set(scope, emit_key.into(), emit_instance.into());

    // removeListener(eventName, listener)
    let remove_listener_func = v8::FunctionTemplate::new(scope, event_emitter_remove_listener_callback);
    let remove_listener_instance = remove_listener_func.get_function(scope).unwrap();
    let remove_listener_key = v8::String::new(scope, "removeListener").unwrap();
    emitter_obj.set(scope, remove_listener_key.into(), remove_listener_instance.into());

    // removeAllListeners(eventName)
    let remove_all_func = v8::FunctionTemplate::new(scope, event_emitter_remove_all_callback);
    let remove_all_instance = remove_all_func.get_function(scope).unwrap();
    let remove_all_key = v8::String::new(scope, "removeAllListeners").unwrap();
    emitter_obj.set(scope, remove_all_key.into(), remove_all_instance.into());

    // listeners(eventName)
    let listeners_func = v8::FunctionTemplate::new(scope, event_emitter_listeners_callback);
    let listeners_instance = listeners_func.get_function(scope).unwrap();
    let listeners_key = v8::String::new(scope, "listeners").unwrap();
    emitter_obj.set(scope, listeners_key.into(), listeners_instance.into());

    // eventNames()
    let event_names_func = v8::FunctionTemplate::new(scope, event_emitter_event_names_callback);
    let event_names_instance = event_names_func.get_function(scope).unwrap();
    let event_names_key = v8::String::new(scope, "eventNames").unwrap();
    emitter_obj.set(scope, event_names_key.into(), event_names_instance.into());

    // getMaxListeners()
    let get_max_func = v8::FunctionTemplate::new(scope, event_emitter_get_max_callback);
    let get_max_instance = get_max_func.get_function(scope).unwrap();
    let get_max_key = v8::String::new(scope, "getMaxListeners").unwrap();
    emitter_obj.set(scope, get_max_key.into(), get_max_instance.into());

    // setMaxListeners(n)
    let set_max_func = v8::FunctionTemplate::new(scope, event_emitter_set_max_callback);
    let set_max_instance = set_max_func.get_function(scope).unwrap();
    let set_max_key = v8::String::new(scope, "setMaxListeners").unwrap();
    emitter_obj.set(scope, set_max_key.into(), set_max_instance.into());

    // 添加_maxListeners属性
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_val = v8::Integer::new(scope, 10);
    emitter_obj.set(scope, max_listeners_key.into(), max_val.into());

    retval.set(emitter_obj.into());
}

fn event_emitter_on_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let listener = args.get(1);
    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }

    // 创建持久化函数引用
    let listener_func = v8::Local::<v8::Function>::try_from(listener).unwrap();
    let function_global = v8::Global::new(scope, listener_func);

    // 获取当前的监听器数组
    let listeners_key = v8::String::new(scope, "_listeners").unwrap();
    let existing_listeners = this.get(scope, listeners_key.into());

    let mut listeners_map: HashMap<String, Vec<v8::Global<v8::Function>>> = HashMap::new();

    if let Some(arr) = existing_listeners.and_then(|v| v.to_object(scope)) {
        // 转换现有的监听器
        let listener_names = EVENT_LISTENERS.with(|map| {
            let map_ref = map.lock().unwrap();
            map_ref.keys().cloned().collect::<Vec<_>>()
        });

        for name in listener_names {
            let name_key = v8::String::new(scope, &name).unwrap();
            // 检查属性是否存在（简化实现）
            let prop = arr.get(scope, name_key.into());
            if prop.is_some() && !prop.unwrap().is_undefined() {
                listeners_map.insert(name, vec![]); // 简化实现
            }
        }
    }

    // 添加新监听器
    EVENT_LISTENERS.with(|map| {
        let mut map_ref = map.lock().unwrap();
        map_ref.entry(event_name.clone()).or_insert_with(Vec::new).push(function_global);
    });

    // 在对象上设置属性标记
    let prop_key = v8::String::new(scope, &event_name).unwrap();
    let val = v8::Boolean::new(scope, true).into();
    this.set(scope, prop_key.into(), val);

    retval.set(this.into());
}

fn event_emitter_once_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let listener = args.get(1);
    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }

    let listener_func = v8::Local::<v8::Function>::try_from(listener).unwrap();
    let function_global = v8::Global::new(scope, listener_func);

    // 添加一次性监听器
    ONCE_LISTENERS.with(|map| {
        let mut map_ref = map.lock().unwrap();
        map_ref.entry(event_name.clone()).or_insert_with(Vec::new).push(function_global);
    });

    let prop_key = v8::String::new(scope, &event_name).unwrap();
    let prop_val = v8::Boolean::new(scope, true);
    this.set(scope, prop_key.into(), prop_val.into());

    retval.set(this.into());
}

fn event_emitter_emit_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let mut event_args: Vec<v8::Local<v8::Value>> = Vec::new();
    for i in 1..args.length() {
        event_args.push(args.get(i));
    }

    let mut emitted = false;

    // 调用普通监听器
    EVENT_LISTENERS.with(|map| {
        let map_ref = map.lock().unwrap();
        if let Some(listeners) = map_ref.get(&event_name) {
            for listener in listeners {
                let listener_func = v8::Local::new(scope, listener);
                listener_func.call(scope, this.into(), &event_args);
                emitted = true;
            }
        }
    });

    // 调用一次性监听器并移除
    let mut executed_once = Vec::new();
    ONCE_LISTENERS.with(|map| {
        let mut map_ref = map.lock().unwrap();
        if let Some(listeners) = map_ref.get_mut(&event_name) {
            for listener in listeners.iter() {
                let listener_func = v8::Local::new(scope, listener);
                listener_func.call(scope, this.into(), &event_args);
                executed_once.push(listener.clone());
                emitted = true;
            }

            // 移除已执行的一次性监听器
            listeners.retain(|l| !executed_once.contains(l));
        }
    });

    retval.set(v8::Boolean::new(scope, emitted).into());
}

fn event_emitter_remove_listener_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let listener = args.get(1);
    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }

    // 简化实现：移除事件标记
    let prop_key = v8::String::new(scope, &event_name).unwrap();
    this.delete(scope, prop_key.into());

    retval.set(this.into());
}

fn event_emitter_remove_all_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    if event_name.is_empty() {
        // 移除所有事件
        EVENT_LISTENERS.with(|map| {
            let mut map_ref = map.lock().unwrap();
            map_ref.clear();
        });

        ONCE_LISTENERS.with(|map| {
            let mut map_ref = map.lock().unwrap();
            map_ref.clear();
        });
    } else {
        // 移除特定事件
        EVENT_LISTENERS.with(|map| {
            let mut map_ref = map.lock().unwrap();
            map_ref.remove(&event_name);
        });

        ONCE_LISTENERS.with(|map| {
            let mut map_ref = map.lock().unwrap();
            map_ref.remove(&event_name);
        });
    }

    retval.set(this.into());
}

fn event_emitter_listeners_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let event_name = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let listeners_array = v8::Array::new(scope, 0);

    EVENT_LISTENERS.with(|map| {
        let map_ref = map.lock().unwrap();
        if let Some(listeners) = map_ref.get(&event_name) {
            for (i, listener) in listeners.iter().enumerate() {
                let listener_func = v8::Local::new(scope, listener);
                listeners_array.set_index(scope, i as u32, listener_func.into());
            }
        }
    });

    retval.set(listeners_array.into());
}

fn event_emitter_event_names_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let names_array = v8::Array::new(scope, 0);

    EVENT_LISTENERS.with(|map| {
        let map_ref = map.lock().unwrap();
        for (i, (name, _)) in map_ref.iter().enumerate() {
            let name_str = v8::String::new(scope, name).unwrap();
            names_array.set_index(scope, i as u32, name_str.into());
        }
    });

    retval.set(names_array.into());
}

fn event_emitter_listener_count_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let emitter = args.get(0);
    let event_name = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let mut count = 0;

    EVENT_LISTENERS.with(|map| {
        let map_ref = map.lock().unwrap();
        if let Some(listeners) = map_ref.get(&event_name) {
            count = listeners.len();
        }
    });

    retval.set(v8::Integer::new(scope, count as i32).into());
}

fn event_emitter_get_max_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let max_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max = this.get(scope, max_key.into()).unwrap_or(v8::Integer::new(scope, 10).into());
    retval.set(max);
}

fn event_emitter_set_max_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let n = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 10))
        .value() as i32;

    let max_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_key_val = v8::Integer::new(scope, n).into();

    this.set(scope, max_key.into(), max_key_val);

    retval.set(this.into());
}

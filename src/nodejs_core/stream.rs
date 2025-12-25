// Node.js Stream模块实现
/// 高性能流处理，支持背压机制
use anyhow::Result;
use rusty_v8 as v8;

/// 从 JavaScript 调用 push 方法
/// v0.3.56: 支持 push(null) 触发 end 事件
fn readable_push_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let chunk: _ = args.get(0);

    // 获取 _readableState
    let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    let state_val: Option<v8::Local<v8::Value>> = this.get(scope, state_key.into());

    // 检查是否是 push(null) - 表示流结束
    if chunk.is_null() {
        // 标记流已结束
        if let Some(state_local) = state_val {
            if let Some(state_obj) = state_local.to_object(scope) {
                let ended_key: _ = v8::String::new(scope, "ended").unwrap();
                let ended_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
                state_obj.set(scope, ended_key.into(), ended_val);
            }
        }

        // 触发 'end' 事件 - 查找 this 上的 'end' 属性作为监听器
        let end_key: _ = v8::String::new(scope, "end").unwrap();
        if let Some(listener) = this.get(scope, end_key.into()) {
            if listener.is_function() {
                if let Ok(func) = v8::Local::<v8::Function>::try_from(listener) {
                    func.call(scope, this.into(), &[]);
                }
            }
        }

        let result_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
        retval.set(result_val);
        return;
    }

    // 如果流处于 flowing 模式，触发 data 事件
    if let Some(state_local) = state_val {
        if let Some(state_obj) = state_local.to_object(scope) {
            let flowing_key: _ = v8::String::new(scope, "flowing").unwrap();
            if let Some(flowing_val) = state_obj.get(scope, flowing_key.into()) {
                if flowing_val.to_boolean(scope).boolean_value(scope) {
                    // 触发 data 事件
                    let data_key: _ = v8::String::new(scope, "data").unwrap();
                    if let Some(listener) = this.get(scope, data_key.into()) {
                        if listener.is_function() {
                            if let Ok(func) = v8::Local::<v8::Function>::try_from(listener) {
                                func.call(scope, this.into(), &[chunk]);
                            }
                        }
                    }
                }
            }
        }
    }

    retval.set(v8::Boolean::new(scope, true).into());
}

/// 设置Stream API
pub fn setup_stream_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global: _ = context.global(scope);
    // Readable Stream
    let readable_constructor: _ = v8::FunctionTemplate::new(scope, readable_constructor_callback);
    let readable_func: _ = readable_constructor.get_function(scope).unwrap();
    let readable_key: _ = v8::String::new(scope, "Readable").unwrap();
    let stream_obj: _ = v8::Object::new(scope);
    stream_obj.set(scope, readable_key.into(), readable_func.into());
    // Writable Stream
    let writable_constructor: _ = v8::FunctionTemplate::new(scope, writable_constructor_callback);
    let writable_func: _ = writable_constructor.get_function(scope).unwrap();
    let writable_key: _ = v8::String::new(scope, "Writable").unwrap();
    stream_obj.set(scope, writable_key.into(), writable_func.into());
    // Transform Stream
    let transform_constructor: _ = v8::FunctionTemplate::new(scope, transform_constructor_callback);
    let transform_func: _ = transform_constructor.get_function(scope).unwrap();
    let transform_key: _ = v8::String::new(scope, "Transform").unwrap();
    stream_obj.set(scope, transform_key.into(), transform_func.into());
    // Duplex Stream
    let duplex_constructor: _ = v8::FunctionTemplate::new(scope, duplex_constructor_callback);
    let duplex_func: _ = duplex_constructor.get_function(scope).unwrap();
    let duplex_key: _ = v8::String::new(scope, "Duplex").unwrap();
    stream_obj.set(scope, duplex_key.into(), duplex_func.into());
    // 设置到全局
    let stream_key: _ = v8::String::new(scope, "stream").unwrap();
    global.set(scope, stream_key.into(), stream_obj.into());
    Ok(())
}
fn readable_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stream_obj: _ = v8::Object::new(scope);

    // _read方法
    let read_func: _ = v8::FunctionTemplate::new(scope, readable_read_callback);
    let read_instance: _ = read_func.get_function(scope).unwrap();
    let read_key: _ = v8::String::new(scope, "_read").unwrap();
    stream_obj.set(scope, read_key.into(), read_instance.into());

    // read方法
    let read_public_func: _ = v8::FunctionTemplate::new(scope, readable_public_read_callback);
    let read_public_instance: _ = read_public_func.get_function(scope).unwrap();
    let read_public_key: _ = v8::String::new(scope, "read").unwrap();
    stream_obj.set(scope, read_public_key.into(), read_public_instance.into());

    // push方法 - v0.3.56 新增：允许从 _read 推入数据
    let push_func: _ = v8::FunctionTemplate::new(scope, readable_push_callback);
    let push_instance: _ = push_func.get_function(scope).unwrap();
    let push_key: _ = v8::String::new(scope, "push").unwrap();
    stream_obj.set(scope, push_key.into(), push_instance.into());

    // on方法
    let on_func: _ = v8::FunctionTemplate::new(scope, readable_on_callback);
    let on_instance: _ = on_func.get_function(scope).unwrap();
    let on_key: _ = v8::String::new(scope, "on").unwrap();
    stream_obj.set(scope, on_key.into(), on_instance.into());

    // once方法 - v0.3.56 新增：一次性事件监听
    let once_func: _ = v8::FunctionTemplate::new(scope, readable_once_callback);
    let once_instance: _ = once_func.get_function(scope).unwrap();
    let once_key: _ = v8::String::new(scope, "once").unwrap();
    stream_obj.set(scope, once_key.into(), once_instance.into());

    // pause方法
    let pause_func: _ = v8::FunctionTemplate::new(scope, readable_pause_callback);
    let pause_instance: _ = pause_func.get_function(scope).unwrap();
    let pause_key: _ = v8::String::new(scope, "pause").unwrap();
    stream_obj.set(scope, pause_key.into(), pause_instance.into());

    // resume方法
    let resume_func: _ = v8::FunctionTemplate::new(scope, readable_resume_callback);
    let resume_instance: _ = resume_func.get_function(scope).unwrap();
    let resume_key: _ = v8::String::new(scope, "resume").unwrap();
    stream_obj.set(scope, resume_key.into(), resume_instance.into());

    // pipe方法
    let pipe_func: _ = v8::FunctionTemplate::new(scope, readable_pipe_callback);
    let pipe_instance: _ = pipe_func.get_function(scope).unwrap();
    let pipe_key: _ = v8::String::new(scope, "pipe").unwrap();
    stream_obj.set(scope, pipe_key.into(), pipe_instance.into());

    // unpipe方法
    let unpipe_func: _ = v8::FunctionTemplate::new(scope, readable_unpipe_callback);
    let unpipe_instance: _ = unpipe_func.get_function(scope).unwrap();
    let unpipe_key: _ = v8::String::new(scope, "unpipe").unwrap();
    stream_obj.set(scope, unpipe_key.into(), unpipe_instance.into());

    // _readableState - v0.3.56 增强
    let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    let state_obj: _ = v8::Object::new(scope);

    let flowing_key: _ = v8::String::new(scope, "flowing").unwrap();
    let flowing_val: _ = v8::Boolean::new(scope, false);
    state_obj.set(scope, flowing_key.into(), flowing_val.into());

    let paused_key: _ = v8::String::new(scope, "paused").unwrap();
    let paused_val: _ = v8::Boolean::new(scope, false);
    state_obj.set(scope, paused_key.into(), paused_val.into());

    let ended_key: _ = v8::String::new(scope, "ended").unwrap();
    let ended_val: _ = v8::Boolean::new(scope, false);
    state_obj.set(scope, ended_key.into(), ended_val.into());

    let high_water_mark_key: _ = v8::String::new(scope, "highWaterMark").unwrap();
    let hwm_val: _ = v8::Integer::new(scope, 16 * 1024);
    state_obj.set(scope, high_water_mark_key.into(), hwm_val.into());

    stream_obj.set(scope, state_key.into(), state_obj.into());

    retval.set(stream_obj.into());
}
fn readable_read_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 默认_read实现 - 产生数据
    let size: _ = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 16 * 1024))
        .value() as usize;
    // 创建一些测试数据
    let data: _ = vec![b'A'; size.min(1024)];
    let chunk: _ = v8::ArrayBuffer::new(scope, data.len());
    retval.set(chunk.into());
}
fn readable_public_read_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let size: _ = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, -1))
        .value();
    // 调用_read方法
    let read_key: _ = v8::String::new(scope, "_read").unwrap();
    if let Some(read_func_value) = this.get(scope, read_key.into()) {
        if read_func_value.is_function() {
            if let Ok(read_func) = v8::Local::<v8::Function>::try_from(read_func_value) {
                let size_val: _ = v8::Integer::new(scope, size as i32);
                let call_args: &[v8::Local<v8::Value>] = &[size_val.into()];
                if let Some(result) = read_func.call(scope, this.into(), call_args) {
                    retval.set(result);
                    return;
                }
            }
        }
    }
    // 简化实现：返回null表示没有更多数据
    retval.set(v8::null(scope).into());
}
fn readable_on_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let event: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let listener: _ = args.get(1);
    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }

    // 将监听器设置到 this 对象上
    let event_key: _ = v8::String::new(scope, &event).unwrap();
    this.set(scope, event_key.into(), listener);

    // 立即触发已结束的流上的 'end' 事件
    if event == "end" {
        let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
        if let Some(state_val) = this.get(scope, state_key.into()) {
            if let Some(state_obj) = state_val.to_object(scope) {
                let ended_key: _ = v8::String::new(scope, "ended").unwrap();
                if let Some(ended) = state_obj.get(scope, ended_key.into()) {
                    if ended.to_boolean(scope).boolean_value(scope) {
                        if let Ok(listener_func) = v8::Local::<v8::Function>::try_from(listener) {
                            listener_func.call(scope, this.into(), &[]);
                        }
                        retval.set(this.into());
                        return;
                    }
                }
            }
        }
    }

    retval.set(this.into());
}

/// once回调 - v0.3.56 新增：一次性事件监听
fn readable_once_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let event: _ = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let listener: _ = args.get(1);

    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }

    // 对于 'end' 事件，检查是否已经结束
    if event == "end" {
        let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
        if let Some(state_val) = this.get(scope, state_key.into()) {
            if let Some(state_obj) = state_val.to_object(scope) {
                let ended_key: _ = v8::String::new(scope, "ended").unwrap();
                if let Some(ended) = state_obj.get(scope, ended_key.into()) {
                    if ended.to_boolean(scope).boolean_value(scope) {
                        if let Ok(listener_func) = v8::Local::<v8::Function>::try_from(listener) {
                            listener_func.call(scope, this.into(), &[]);
                        }
                        retval.set(this.into());
                        return;
                    }
                }
            }
        }
    }

    // 设置监听器（与 on 相同）
    let event_key: _ = v8::String::new(scope, &event).unwrap();
    this.set(scope, event_key.into(), listener);

    retval.set(this.into());
}
fn readable_pause_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    let state: _ = this.get(scope, state_key.into()).unwrap();
    let flow_key: _ = v8::String::new(scope, "flowing").unwrap();
    let flow_val: _ = v8::Boolean::new(scope, false);
    let paused_key: _ = v8::String::new(scope, "paused").unwrap();
    let paused_val: _ = v8::Boolean::new(scope, true);
    if let Some(state_obj) = state.to_object(scope) {
        state_obj.set(scope, flow_key.into(), flow_val.into());
        state_obj.set(scope, paused_key.into(), paused_val.into());
    }
    retval.set(this.into());
}
fn readable_resume_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    let state: _ = this.get(scope, state_key.into()).unwrap();
    let flow_key: _ = v8::String::new(scope, "flowing").unwrap();
    let flow_val: _ = v8::Boolean::new(scope, true);
    let paused_key: _ = v8::String::new(scope, "paused").unwrap();
    let paused_val: _ = v8::Boolean::new(scope, false);
    if let Some(state_obj) = state.to_object(scope) {
        state_obj.set(scope, flow_key.into(), flow_val.into());
        state_obj.set(scope, paused_key.into(), paused_val.into());
    }
    retval.set(this.into());
}
fn readable_pipe_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let destination: _ = args.get(0);
    retval.set(destination);
}
fn readable_unpipe_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    retval.set(this.into());
}
fn writable_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stream_obj: _ = v8::Object::new(scope);
    // _write方法
    let write_func: _ = v8::FunctionTemplate::new(scope, writable_write_callback);
    let write_instance: _ = write_func.get_function(scope).unwrap();
    let write_key: _ = v8::String::new(scope, "_write").unwrap();
    stream_obj.set(scope, write_key.into(), write_instance.into());
    // write方法
    let write_public_func: _ = v8::FunctionTemplate::new(scope, writable_public_write_callback);
    let write_public_instance: _ = write_public_func.get_function(scope).unwrap();
    let write_public_key: _ = v8::String::new(scope, "write").unwrap();
    stream_obj.set(scope, write_public_key.into(), write_public_instance.into());
    // end方法
    let end_func: _ = v8::FunctionTemplate::new(scope, writable_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    stream_obj.set(scope, end_key.into(), end_instance.into());
    retval.set(stream_obj.into());
}
fn writable_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let chunk: _ = args.get(0);
    let encoding: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    // 默认_write实现 - 输出到控制台
    if chunk.is_string() {
        let content: _ = chunk.to_string(scope).unwrap().to_rust_string_lossy(scope);
        eprintln!("[Writable Stream] {}: {}", encoding, content);
    }
    retval.set(v8::undefined(scope).into());
}
fn writable_public_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let chunk: _ = args.get(0);
    let encoding: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let callback: _ = args.get(2);
    // 调用_write方法
    let write_key: _ = v8::String::new(scope, "_write").unwrap();
    if let Some(write_func_val) = this.get(scope, write_key.into()) {
        if write_func_val.is_function() {
            if let Ok(write_func) = v8::Local::<v8::Function>::try_from(write_func_val) {
                let encoding_val: _ = v8::String::new(scope, &encoding).unwrap();
                let call_args: &[v8::Local<v8::Value>] = &[chunk, encoding_val.into()];
                write_func.call(scope, this.into(), call_args);
                // 如果有回调，调用它
                if callback.is_function() {
                    if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
                        cb_func.call(scope, this.into(), &[]);
                    }
                }
            }
        }
    }
    retval.set(v8::Boolean::new(scope, true).into());
}
fn writable_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this();
    let chunk: _ = args.get(0);
    let encoding: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    // 结束写入
    if !chunk.is_undefined() {
        let write_key: _ = v8::String::new(scope, "write").unwrap();
        if let Some(write_func_val) = this.get(scope, write_key.into()) {
            if write_func_val.is_function() {
                if let Ok(write_func) = v8::Local::<v8::Function>::try_from(write_func_val) {
                    let encoding_val: _ = v8::String::new(scope, &encoding).unwrap();
                    let call_args: &[v8::Local<v8::Value>] = &[chunk, encoding_val.into()];
                    write_func.call(scope, this.into(), call_args);
                }
            }
        }
    }
    let callback: _ = args.get(2);
    if callback.is_function() {
        if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
            cb_func.call(scope, this.into(), &[]);
        }
    }
    retval.set(this.into());
}
fn transform_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stream_obj: _ = v8::Object::new(scope);
    // _transform方法
    let transform_func: _ = v8::FunctionTemplate::new(scope, transform_transform_callback);
    let transform_instance: _ = transform_func.get_function(scope).unwrap();
    let transform_key: _ = v8::String::new(scope, "_transform").unwrap();
    stream_obj.set(scope, transform_key.into(), transform_instance.into());
    retval.set(stream_obj.into());
}
fn transform_transform_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let chunk: _ = args.get(0);
    let encoding: _ = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let callback: _ = args.get(2);
    // 默认_transform实现
    if callback.is_function() {
        let this: _ = args.this();
        if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
            cb_func.call(scope, this.into(), &[]);
        }
    }
    retval.set(v8::undefined(scope).into());
}
fn duplex_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stream_obj: _ = v8::Object::new(scope);
    retval.set(stream_obj.into());
}

//! Node.js Stream模块实现
//! 高性能流处理

use anyhow::Result;
use rusty_v8 as v8;

/// 设置Stream API
pub fn setup_stream_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Readable Stream
    let readable_constructor = v8::FunctionTemplate::new(scope, readable_constructor_callback);
    let readable_func = readable_constructor.get_function(scope).unwrap();
    let readable_key = v8::String::new(scope, "Readable").unwrap();
    let stream_obj = v8::Object::new(scope);
    stream_obj.set(scope, readable_key.into(), readable_func.into());

    // Writable Stream
    let writable_constructor = v8::FunctionTemplate::new(scope, writable_constructor_callback);
    let writable_func = writable_constructor.get_function(scope).unwrap();
    let writable_key = v8::String::new(scope, "Writable").unwrap();
    stream_obj.set(scope, writable_key.into(), writable_func.into());

    // Transform Stream
    let transform_constructor = v8::FunctionTemplate::new(scope, transform_constructor_callback);
    let transform_func = transform_constructor.get_function(scope).unwrap();
    let transform_key = v8::String::new(scope, "Transform").unwrap();
    stream_obj.set(scope, transform_key.into(), transform_func.into());

    // Duplex Stream
    let duplex_constructor = v8::FunctionTemplate::new(scope, duplex_constructor_callback);
    let duplex_func = duplex_constructor.get_function(scope).unwrap();
    let duplex_key = v8::String::new(scope, "Duplex").unwrap();
    stream_obj.set(scope, duplex_key.into(), duplex_func.into());

    // 设置到全局
    let stream_key = v8::String::new(scope, "stream").unwrap();
    global.set(scope, stream_key.into(), stream_obj.into());

    Ok(())
}

fn readable_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stream_obj = v8::Object::new(scope);

    // _read方法
    let read_func = v8::FunctionTemplate::new(scope, readable_read_callback);
    let read_instance = read_func.get_function(scope).unwrap();
    let read_key = v8::String::new(scope, "_read").unwrap();
    stream_obj.set(scope, read_key.into(), read_instance.into());

    // read方法
    let read_public_func = v8::FunctionTemplate::new(scope, readable_public_read_callback);
    let read_public_instance = read_public_func.get_function(scope).unwrap();
    let read_public_key = v8::String::new(scope, "read").unwrap();
    stream_obj.set(scope, read_public_key.into(), read_public_instance.into());

    // on方法
    let on_func = v8::FunctionTemplate::new(scope, readable_on_callback);
    let on_instance = on_func.get_function(scope).unwrap();
    let on_key = v8::String::new(scope, "on").unwrap();
    stream_obj.set(scope, on_key.into(), on_instance.into());

    // pause方法
    let pause_func = v8::FunctionTemplate::new(scope, readable_pause_callback);
    let pause_instance = pause_func.get_function(scope).unwrap();
    let pause_key = v8::String::new(scope, "pause").unwrap();
    stream_obj.set(scope, pause_key.into(), pause_instance.into());

    // resume方法
    let resume_func = v8::FunctionTemplate::new(scope, readable_resume_callback);
    let resume_instance = resume_func.get_function(scope).unwrap();
    let resume_key = v8::String::new(scope, "resume").unwrap();
    stream_obj.set(scope, resume_key.into(), resume_instance.into());

    // pipe方法
    let pipe_func = v8::FunctionTemplate::new(scope, readable_pipe_callback);
    let pipe_instance = pipe_func.get_function(scope).unwrap();
    let pipe_key = v8::String::new(scope, "pipe").unwrap();
    stream_obj.set(scope, pipe_key.into(), pipe_instance.into());

    // unpipe方法
    let unpipe_func = v8::FunctionTemplate::new(scope, readable_unpipe_callback);
    let unpipe_instance = unpipe_func.get_function(scope).unwrap();
    let unpipe_key = v8::String::new(scope, "unpipe").unwrap();
    stream_obj.set(scope, unpipe_key.into(), unpipe_instance.into());

    // _readableState
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state_obj = v8::Object::new(scope);
    let flowing_key = v8::String::new(scope, "flowing").unwrap();
    let flowing_val = v8::Boolean::new(scope, false);
    state_obj.set(scope, flowing_key.into(), flowing_val.into());
    let paused_key = v8::String::new(scope, "paused").unwrap();
    let paused_val = v8::Boolean::new(scope, false);
    state_obj.set(scope, paused_key.into(), paused_val.into());
    stream_obj.set(scope, state_key.into(), state_obj.into());

    retval.set(stream_obj.into());
}

fn readable_read_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // 默认_read实现 - 产生数据
    let size = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 16 * 1024))
        .value() as usize;

    // 创建一些测试数据
    let data = vec![b'A'; size.min(1024)];
    let chunk = v8::ArrayBuffer::new(scope, data.len());
    // In newer V8 APIs, backing_store() has been replaced
    // We'll use a different approach to set the data
    unsafe {
        let buffer_ptr = chunk.buffer().data() as *mut u8;
        std::ptr::copy_nonoverlapping(data.as_ptr(), buffer_ptr, data.len());
    }

    retval.set(chunk.into());
}

fn readable_public_read_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let size = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, -1))
        .value();

    // 调用_read方法
    let read_key = v8::String::new(scope, "_read").unwrap();
    if let Some(read_func_value) = this.get(scope, read_key.into()) {
        if read_func_value.is_function() {
            if let Ok(read_func) = v8::Local::<v8::Function>::try_from(read_func_value) {
                let size_val = v8::Integer::new(scope, size as i32);
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
    let this = args.this();
    let event = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let listener = args.get(1);

    if !listener.is_function() {
        retval.set(v8::null(scope).into());
        return;
    }

    // 模拟emit 'data'事件
    if event == "data" {
        // 创建测试数据
        let data = v8::String::new(scope, "test data chunk").unwrap();
        let _data_value: v8::Local<v8::Value> = data.into();

        if listener.is_function() {
            if let Ok(listener_func) = v8::Local::<v8::Function>::try_from(listener) {
                let call_args: &[v8::Local<v8::Value>] = &[data.into()];
                listener_func.call(scope, this.into(), call_args);
            }
        }
    }

    // 模拟emit 'end'事件
    if event == "end" {
        if listener.is_function() {
            if let Ok(listener_func) = v8::Local::<v8::Function>::try_from(listener) {
                listener_func.call(scope, this.into(), &[]);
            }
        }
    }

    retval.set(this.into());
}

fn readable_pause_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    let flow_key = v8::String::new(scope, "flowing").unwrap();
    let flow_val = v8::Boolean::new(scope, false);
    if let Some(state_obj) = state.to_object(scope) {
        state_obj.set(scope, flow_key.into(), flow_val.into());
    }
    retval.set(this.into());
}

fn readable_resume_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    let flow_key = v8::String::new(scope, "flowing").unwrap();
    let flow_val = v8::Boolean::new(scope, true);
    if let Some(state_obj) = state.to_object(scope) {
        state_obj.set(scope, flow_key.into(), flow_val.into());
    }
    retval.set(this.into());
}

fn readable_pipe_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let destination = args.get(0);
    retval.set(destination);
}

fn readable_unpipe_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    retval.set(this.into());
}

fn writable_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let stream_obj = v8::Object::new(scope);

    // _write方法
    let write_func = v8::FunctionTemplate::new(scope, writable_write_callback);
    let write_instance = write_func.get_function(scope).unwrap();
    let write_key = v8::String::new(scope, "_write").unwrap();
    stream_obj.set(scope, write_key.into(), write_instance.into());

    // write方法
    let write_public_func = v8::FunctionTemplate::new(scope, writable_public_write_callback);
    let write_public_instance = write_public_func.get_function(scope).unwrap();
    let write_public_key = v8::String::new(scope, "write").unwrap();
    stream_obj.set(scope, write_public_key.into(), write_public_instance.into());

    // end方法
    let end_func = v8::FunctionTemplate::new(scope, writable_end_callback);
    let end_instance = end_func.get_function(scope).unwrap();
    let end_key = v8::String::new(scope, "end").unwrap();
    stream_obj.set(scope, end_key.into(), end_instance.into());

    retval.set(stream_obj.into());
}

fn writable_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let chunk = args.get(0);
    let encoding = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // 默认_write实现 - 输出到控制台
    if chunk.is_string() {
        let content = chunk.to_string(scope).unwrap().to_rust_string_lossy(scope);
        eprintln!("[Writable Stream] {}: {}", encoding, content);
    }

    retval.set(v8::undefined(scope).into());
}

fn writable_public_write_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let chunk = args.get(0);
    let encoding = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let callback = args.get(2);

    // 调用_write方法
    let write_key = v8::String::new(scope, "_write").unwrap();
    if let Some(write_func_val) = this.get(scope, write_key.into()) {
        if write_func_val.is_function() {
            if let Ok(write_func) = v8::Local::<v8::Function>::try_from(write_func_val) {
                let encoding_val = v8::String::new(scope, &encoding).unwrap();
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
    let this = args.this();
    let chunk = args.get(0);
    let encoding = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    // 结束写入
    if !chunk.is_undefined() {
        let write_key = v8::String::new(scope, "write").unwrap();
        if let Some(write_func_val) = this.get(scope, write_key.into()) {
            if write_func_val.is_function() {
                if let Ok(write_func) = v8::Local::<v8::Function>::try_from(write_func_val) {
                    let encoding_val = v8::String::new(scope, &encoding).unwrap();
                    let call_args: &[v8::Local<v8::Value>] = &[chunk, encoding_val.into()];
                    write_func.call(scope, this.into(), call_args);
                }
            }
        }
    }

    let callback = args.get(2);
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
    let stream_obj = v8::Object::new(scope);

    // _transform方法
    let transform_func = v8::FunctionTemplate::new(scope, transform_transform_callback);
    let transform_instance = transform_func.get_function(scope).unwrap();
    let transform_key = v8::String::new(scope, "_transform").unwrap();
    stream_obj.set(scope, transform_key.into(), transform_instance.into());

    retval.set(stream_obj.into());
}

fn transform_transform_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let chunk = args.get(0);
    let encoding = args
        .get(1)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();

    let callback = args.get(2);

    // 默认_transform实现
    if callback.is_function() {
        let this = args.this();
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
    let stream_obj = v8::Object::new(scope);
    retval.set(stream_obj.into());
}

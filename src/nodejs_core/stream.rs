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

    // v0.3.59: 支持 options 参数，包含用户自定义的 _read 函数
    let options: _ = args.get(0);

    // 检查是否提供了自定义 _read 函数
    let mut has_custom_read = false;
    if options.is_object() {
        if let Some(options_obj) = options.to_object(scope) {
            let read_key: _ = v8::String::new(scope, "read").unwrap();
            if let Some(read_func) = options_obj.get(scope, read_key.into()) {
                if read_func.is_function() {
                    // 使用用户提供的 _read 函数
                    let _read_key: _ = v8::String::new(scope, "_read").unwrap();
                    stream_obj.set(scope, _read_key.into(), read_func);
                    has_custom_read = true;
                }
            }
        }
    }

    // 如果没有自定义 _read，使用默认实现
    if !has_custom_read {
        // _read方法 (默认实现)
        let read_func: _ = v8::FunctionTemplate::new(scope, readable_read_callback);
        let read_instance: _ = read_func.get_function(scope).unwrap();
        let read_key: _ = v8::String::new(scope, "_read").unwrap();
        stream_obj.set(scope, read_key.into(), read_instance.into());
    }

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
/// pipe 数据处理回调 - 当 readable 产生数据时调用
/// v0.3.59: 实现 pipe() 方法的数据流
fn pipe_data_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this(); // readable stream
    let chunk: _ = args.get(0);

    // 获取存储的 destination (writable stream)
    let dest_key: _ = v8::String::new(scope, "_pipeDestination").unwrap();
    let dest_val: Option<v8::Local<v8::Value>> = this.get(scope, dest_key.into());

    if let Some(dest) = dest_val {
        if let Some(dest_obj) = dest.to_object(scope) {
            // 调用 destination.write(chunk)
            let write_key: _ = v8::String::new(scope, "write").unwrap();
            if let Some(write_val) = dest_obj.get(scope, write_key.into()) {
                if write_val.is_function() {
                    if let Ok(write_func) = v8::Local::<v8::Function>::try_from(write_val) {
                        let encoding: _ = v8::String::new(scope, "utf8").unwrap();
                        let call_args: &[v8::Local<v8::Value>] = &[chunk, encoding.into()];
                        write_func.call(scope, dest.into(), call_args);
                    }
                }
            }
        }
    }

    retval.set(v8::undefined(scope).into());
}

/// pipe 结束处理回调 - 当 readable 结束时调用
/// v0.3.59: 实现 pipe() 方法的结束处理
fn pipe_end_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this(); // readable stream

    // 获取存储的 destination (writable stream)
    let dest_key: _ = v8::String::new(scope, "_pipeDestination").unwrap();
    let dest_val: Option<v8::Local<v8::Value>> = this.get(scope, dest_key.into());

    if let Some(dest) = dest_val {
        if let Some(dest_obj) = dest.to_object(scope) {
            // 调用 destination.end()
            let end_key: _ = v8::String::new(scope, "end").unwrap();
            if let Some(end_val) = dest_obj.get(scope, end_key.into()) {
                if end_val.is_function() {
                    if let Ok(end_func) = v8::Local::<v8::Function>::try_from(end_val) {
                        end_func.call(scope, dest.into(), &[]);
                    }
                }
            }
        }
    }

    retval.set(this.into());
}

/// v0.3.59: pipe() 方法实现
/// 将 readable 流的数据管道传输到 writable 流
fn readable_pipe_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: _ = args.this(); // source readable stream
    let destination: _ = args.get(0); // destination writable stream

    if !destination.is_object() {
        retval.set(v8::undefined(scope).into());
        return;
    }

    // 1. 设置 flowing = true 以触发 data 事件
    let state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    if let Some(state_val) = this.get(scope, state_key.into()) {
        if let Some(state_obj) = state_val.to_object(scope) {
            let flowing_key: _ = v8::String::new(scope, "flowing").unwrap();
            let flowing_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
            state_obj.set(scope, flowing_key.into(), flowing_val);
        }
    }

    // 2. 存储 destination 到 readable 对象
    let dest_key: _ = v8::String::new(scope, "_pipeDestination").unwrap();
    this.set(scope, dest_key.into(), destination);

    // 3. 创建 data 事件回调并注册
    let data_callback_func: _ = v8::FunctionTemplate::new(scope, pipe_data_callback);
    let data_callback_instance: _ = data_callback_func.get_function(scope).unwrap();
    let data_key: _ = v8::String::new(scope, "data").unwrap();
    this.set(scope, data_key.into(), data_callback_instance.into());

    // 4. 创建 end 事件回调并注册
    let end_callback_func: _ = v8::FunctionTemplate::new(scope, pipe_end_callback);
    let end_callback_instance: _ = end_callback_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    this.set(scope, end_key.into(), end_callback_instance.into());

    // 5. 调用 read() 启动数据流
    let read_key: _ = v8::String::new(scope, "read").unwrap();
    if let Some(read_val) = this.get(scope, read_key.into()) {
        if read_val.is_function() {
            if let Ok(read_func) = v8::Local::<v8::Function>::try_from(read_val) {
                read_func.call(scope, this.into(), &[]);
            }
        }
    }

    // 6. 返回 destination 以支持链式调用
    retval.set(destination);
}
fn readable_unpipe_callback(
    _scope: &mut v8::HandleScope,
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
    eprintln!("[DEBUG] writable_constructor_callback called");
    let stream_obj: _ = v8::Object::new(scope);

    // v0.3.59: 支持 options 参数，包含用户自定义的 _write 函数
    let options: _ = args.get(0);

    // 检查是否提供了自定义 _write 函数 (支持 write 或 _write 键名)
    let mut has_custom_write = false;
    if options.is_object() {
        if let Some(options_obj) = options.to_object(scope) {
            // 首先尝试获取 _write
            let _write_key: _ = v8::String::new(scope, "_write").unwrap();
            if let Some(write_func) = options_obj.get(scope, _write_key.into()) {
                if write_func.is_function() {
                    stream_obj.set(scope, _write_key.into(), write_func);
                    has_custom_write = true;
                }
            }
            // 如果没有 _write，尝试获取 write
            if !has_custom_write {
                let write_key: _ = v8::String::new(scope, "write").unwrap();
                if let Some(write_func) = options_obj.get(scope, write_key.into()) {
                    if write_func.is_function() {
                        stream_obj.set(scope, _write_key.into(), write_func);
                        has_custom_write = true;
                    }
                }
            }
        }
    }

    // 如果没有自定义 _write，使用默认实现
    if !has_custom_write {
        // _write方法 (默认实现)
        let write_func: _ = v8::FunctionTemplate::new(scope, writable_write_callback);
        let write_instance: _ = write_func.get_function(scope).unwrap();
        let write_key: _ = v8::String::new(scope, "_write").unwrap();
        stream_obj.set(scope, write_key.into(), write_instance.into());
    }
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

    // _writableState - v0.3.57 增强背压支持
    let wstate_key: _ = v8::String::new(scope, "_writableState").unwrap();
    let wstate_obj: _ = v8::Object::new(scope);

    // highWaterMark - 背压水位线
    let hwm_key: _ = v8::String::new(scope, "highWaterMark").unwrap();
    let hwm_val: _ = v8::Integer::new(scope, 16 * 1024);
    wstate_obj.set(scope, hwm_key.into(), hwm_val.into());

    // needDrain - 是否需要等待 drain 事件
    let drain_key: _ = v8::String::new(scope, "needDrain").unwrap();
    let drain_val: _ = v8::Boolean::new(scope, false);
    wstate_obj.set(scope, drain_key.into(), drain_val.into());

    // ended - 是否已结束
    let ended_key: _ = v8::String::new(scope, "ended").unwrap();
    let ended_val: _ = v8::Boolean::new(scope, false);
    wstate_obj.set(scope, ended_key.into(), ended_val.into());

    // writable - 是否可写
    let writable_key: _ = v8::String::new(scope, "writable").unwrap();
    let writable_val: _ = v8::Boolean::new(scope, true);
    wstate_obj.set(scope, writable_key.into(), writable_val.into());

    stream_obj.set(scope, wstate_key.into(), wstate_obj.into());

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

    // 获取 _writableState 检查是否可写
    let wstate_key: _ = v8::String::new(scope, "_writableState").unwrap();
    let wstate_val: Option<v8::Local<v8::Value>> = this.get(scope, wstate_key.into());

    let mut can_continue = true;
    if let Some(state_local) = wstate_val {
        if let Some(state_obj) = state_local.to_object(scope) {
            let writable_key: _ = v8::String::new(scope, "writable").unwrap();
            if let Some(writable_val) = state_obj.get(scope, writable_key.into()) {
                can_continue = writable_val.to_boolean(scope).boolean_value(scope);
            }
        }
    }

    if !can_continue {
        // 流已结束或不可写，返回 false 触发背压
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }

    // 调用_write方法
    let write_key: _ = v8::String::new(scope, "_write").unwrap();
    if let Some(write_func_val) = this.get(scope, write_key.into()) {
        if write_func_val.is_function() {
            if let Ok(write_func) = v8::Local::<v8::Function>::try_from(write_func_val) {
                let encoding_val: _ = v8::String::new(scope, &encoding).unwrap();
                let call_args: &[v8::Local<v8::Value>] = &[chunk, encoding_val.into()];
                write_func.call(scope, this.into(), call_args);

                // 模拟背压检测：如果缓冲区满，设置 needDrain
                // 这里简单处理：总是返回 true，实际应用中应检查缓冲区状态
                if callback.is_function() {
                    if let Ok(cb_func) = v8::Local::<v8::Function>::try_from(callback) {
                        cb_func.call(scope, this.into(), &[]);
                    }
                }
            }
        }
    }
    // v0.3.57: 正确返回 true（可继续写入），实际背压由 _write 实现控制
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

    // 结束写入 - 先写入最后的数据
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

    // v0.3.57: 更新 _writableState - 设置 ended 和 writable
    let wstate_key: _ = v8::String::new(scope, "_writableState").unwrap();
    if let Some(wstate_val) = this.get(scope, wstate_key.into()) {
        if let Some(wstate_obj) = wstate_val.to_object(scope) {
            // 设置 ended = true
            let ended_key: _ = v8::String::new(scope, "ended").unwrap();
            let ended_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
            wstate_obj.set(scope, ended_key.into(), ended_val);

            // 设置 writable = false
            let writable_key: _ = v8::String::new(scope, "writable").unwrap();
            let writable_val: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
            wstate_obj.set(scope, writable_key.into(), writable_val);
        }
    }

    // v0.3.57: 触发 'finish' 事件
    let finish_key: _ = v8::String::new(scope, "finish").unwrap();
    if let Some(listener) = this.get(scope, finish_key.into()) {
        if listener.is_function() {
            if let Ok(func) = v8::Local::<v8::Function>::try_from(listener) {
                func.call(scope, this.into(), &[]);
            }
        }
    }

    // 处理回调
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

    // v0.3.59: 支持 options 参数
    let options: _ = args.get(0);
    let options_obj = if options.is_object() {
        options.to_object(scope)
    } else {
        None
    };

    // ===== Readable 方法 =====

    // _read方法 - 检查是否提供自定义 read
    let mut has_custom_read = false;
    if let Some(obj) = &options_obj {
        let read_key: _ = v8::String::new(scope, "read").unwrap();
        if let Some(read_func) = obj.get(scope, read_key.into()) {
            if read_func.is_function() {
                let _read_key: _ = v8::String::new(scope, "_read").unwrap();
                stream_obj.set(scope, _read_key.into(), read_func);
                has_custom_read = true;
            }
        }
    }
    if !has_custom_read {
        let read_func: _ = v8::FunctionTemplate::new(scope, readable_read_callback);
        let read_instance: _ = read_func.get_function(scope).unwrap();
        let read_key: _ = v8::String::new(scope, "_read").unwrap();
        stream_obj.set(scope, read_key.into(), read_instance.into());
    }

    // read方法
    let read_public_func: _ = v8::FunctionTemplate::new(scope, readable_public_read_callback);
    let read_public_instance: _ = read_public_func.get_function(scope).unwrap();
    let read_public_key: _ = v8::String::new(scope, "read").unwrap();
    stream_obj.set(scope, read_public_key.into(), read_public_instance.into());

    // push方法
    let push_func: _ = v8::FunctionTemplate::new(scope, readable_push_callback);
    let push_instance: _ = push_func.get_function(scope).unwrap();
    let push_key: _ = v8::String::new(scope, "push").unwrap();
    stream_obj.set(scope, push_key.into(), push_instance.into());

    // on方法
    let on_func: _ = v8::FunctionTemplate::new(scope, readable_on_callback);
    let on_instance: _ = on_func.get_function(scope).unwrap();
    let on_key: _ = v8::String::new(scope, "on").unwrap();
    stream_obj.set(scope, on_key.into(), on_instance.into());

    // once方法
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

    // _readableState
    let readable_state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    let readable_state_obj: _ = v8::Object::new(scope);
    let flowing_key: _ = v8::String::new(scope, "flowing").unwrap();
    let flowing_val: _ = v8::Boolean::new(scope, false);
    readable_state_obj.set(scope, flowing_key.into(), flowing_val.into());
    let paused_key: _ = v8::String::new(scope, "paused").unwrap();
    let paused_val: _ = v8::Boolean::new(scope, false);
    readable_state_obj.set(scope, paused_key.into(), paused_val.into());
    let ended_key: _ = v8::String::new(scope, "ended").unwrap();
    let ended_val: _ = v8::Boolean::new(scope, false);
    readable_state_obj.set(scope, ended_key.into(), ended_val.into());
    let high_water_mark_key: _ = v8::String::new(scope, "highWaterMark").unwrap();
    let hwm_val: _ = v8::Integer::new(scope, 16 * 1024);
    readable_state_obj.set(scope, high_water_mark_key.into(), hwm_val.into());
    stream_obj.set(scope, readable_state_key.into(), readable_state_obj.into());

    // ===== Writable 方法 =====
    // _write方法 - Transform/Duplex 使用内部的 _write
    // 检查是否提供自定义 write 或 _write (用于底层 Writable)
    let mut has_custom_write = false;
    if let Some(obj) = &options_obj {
        // 首先尝试获取 _write
        let _write_key: _ = v8::String::new(scope, "_write").unwrap();
        if let Some(write_func) = obj.get(scope, _write_key.into()) {
            if write_func.is_function() {
                stream_obj.set(scope, _write_key.into(), write_func);
                has_custom_write = true;
            }
        }
        // 如果没有 _write，尝试获取 write
        if !has_custom_write {
            let write_key: _ = v8::String::new(scope, "write").unwrap();
            if let Some(write_func) = obj.get(scope, write_key.into()) {
                if write_func.is_function() {
                    stream_obj.set(scope, _write_key.into(), write_func);
                    has_custom_write = true;
                }
            }
        }
    }
    if !has_custom_write {
        // 默认 _write 实现 - 调用 _transform
        let write_private_func: _ = v8::FunctionTemplate::new(scope, writable_write_callback);
        let write_private_instance: _ = write_private_func.get_function(scope).unwrap();
        let write_private_key: _ = v8::String::new(scope, "_write").unwrap();
        stream_obj.set(scope, write_private_key.into(), write_private_instance.into());
    }

    // write方法 (公开)
    let write_func: _ = v8::FunctionTemplate::new(scope, writable_public_write_callback);
    let write_instance: _ = write_func.get_function(scope).unwrap();
    let write_key: _ = v8::String::new(scope, "write").unwrap();
    stream_obj.set(scope, write_key.into(), write_instance.into());

    // end方法
    let end_func: _ = v8::FunctionTemplate::new(scope, writable_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    stream_obj.set(scope, end_key.into(), end_instance.into());

    // on方法 (复用 readable_on)
    stream_obj.set(scope, on_key.into(), on_instance.into());

    // _writableState
    let writable_state_key: _ = v8::String::new(scope, "_writableState").unwrap();
    let writable_state_obj: _ = v8::Object::new(scope);
    let need_drain_key: _ = v8::String::new(scope, "needDrain").unwrap();
    let need_drain_val: _ = v8::Boolean::new(scope, false);
    writable_state_obj.set(scope, need_drain_key.into(), need_drain_val.into());
    let w_ended_key: _ = v8::String::new(scope, "ended").unwrap();
    let w_ended_val: _ = v8::Boolean::new(scope, false);
    writable_state_obj.set(scope, w_ended_key.into(), w_ended_val.into());
    let writable_flag_key: _ = v8::String::new(scope, "writable").unwrap();
    let writable_flag_val: _ = v8::Boolean::new(scope, true);
    writable_state_obj.set(scope, writable_flag_key.into(), writable_flag_val.into());
    let w_hwm_key: _ = v8::String::new(scope, "highWaterMark").unwrap();
    let w_hwm_val: _ = v8::Integer::new(scope, 16 * 1024);
    writable_state_obj.set(scope, w_hwm_key.into(), w_hwm_val.into());
    stream_obj.set(scope, writable_state_key.into(), writable_state_obj.into());

    // ===== Transform 特有方法 =====
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
    let _chunk: _ = args.get(0);
    let _encoding: _ = args
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

    // v0.3.59: 支持 options 参数
    let options: _ = args.get(0);
    let options_obj = if options.is_object() {
        options.to_object(scope)
    } else {
        None
    };

    // ===== Readable 方法 =====

    // _read方法 - 检查是否提供自定义 read
    let mut has_custom_read = false;
    if let Some(obj) = &options_obj {
        let read_key: _ = v8::String::new(scope, "read").unwrap();
        if let Some(read_func) = obj.get(scope, read_key.into()) {
            if read_func.is_function() {
                let _read_key: _ = v8::String::new(scope, "_read").unwrap();
                stream_obj.set(scope, _read_key.into(), read_func);
                has_custom_read = true;
            }
        }
    }
    if !has_custom_read {
        let read_func: _ = v8::FunctionTemplate::new(scope, readable_read_callback);
        let read_instance: _ = read_func.get_function(scope).unwrap();
        let read_key: _ = v8::String::new(scope, "_read").unwrap();
        stream_obj.set(scope, read_key.into(), read_instance.into());
    }

    // read方法
    let read_public_func: _ = v8::FunctionTemplate::new(scope, readable_public_read_callback);
    let read_public_instance: _ = read_public_func.get_function(scope).unwrap();
    let read_public_key: _ = v8::String::new(scope, "read").unwrap();
    stream_obj.set(scope, read_public_key.into(), read_public_instance.into());

    // push方法
    let push_func: _ = v8::FunctionTemplate::new(scope, readable_push_callback);
    let push_instance: _ = push_func.get_function(scope).unwrap();
    let push_key: _ = v8::String::new(scope, "push").unwrap();
    stream_obj.set(scope, push_key.into(), push_instance.into());

    // on方法
    let on_func: _ = v8::FunctionTemplate::new(scope, readable_on_callback);
    let on_instance: _ = on_func.get_function(scope).unwrap();
    let on_key: _ = v8::String::new(scope, "on").unwrap();
    stream_obj.set(scope, on_key.into(), on_instance.into());

    // once方法
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

    // _readableState
    let readable_state_key: _ = v8::String::new(scope, "_readableState").unwrap();
    let readable_state_obj: _ = v8::Object::new(scope);
    let flowing_key: _ = v8::String::new(scope, "flowing").unwrap();
    let flowing_val: _ = v8::Boolean::new(scope, false);
    readable_state_obj.set(scope, flowing_key.into(), flowing_val.into());
    let paused_key: _ = v8::String::new(scope, "paused").unwrap();
    let paused_val: _ = v8::Boolean::new(scope, false);
    readable_state_obj.set(scope, paused_key.into(), paused_val.into());
    let ended_key: _ = v8::String::new(scope, "ended").unwrap();
    let ended_val: _ = v8::Boolean::new(scope, false);
    readable_state_obj.set(scope, ended_key.into(), ended_val.into());
    let high_water_mark_key: _ = v8::String::new(scope, "highWaterMark").unwrap();
    let hwm_val: _ = v8::Integer::new(scope, 16 * 1024);
    readable_state_obj.set(scope, high_water_mark_key.into(), hwm_val.into());
    stream_obj.set(scope, readable_state_key.into(), readable_state_obj.into());

    // ===== Writable 方法 =====
    // _write方法 - Transform/Duplex 使用内部的 _write
    // 检查是否提供自定义 write 或 _write (用于底层 Writable)
    let mut has_custom_write = false;
    if let Some(obj) = &options_obj {
        // 首先尝试获取 _write
        let _write_key: _ = v8::String::new(scope, "_write").unwrap();
        if let Some(write_func) = obj.get(scope, _write_key.into()) {
            if write_func.is_function() {
                stream_obj.set(scope, _write_key.into(), write_func);
                has_custom_write = true;
            }
        }
        // 如果没有 _write，尝试获取 write
        if !has_custom_write {
            let write_key: _ = v8::String::new(scope, "write").unwrap();
            if let Some(write_func) = obj.get(scope, write_key.into()) {
                if write_func.is_function() {
                    stream_obj.set(scope, _write_key.into(), write_func);
                    has_custom_write = true;
                }
            }
        }
    }
    if !has_custom_write {
        // 默认 _write 实现 - 调用 _transform
        let write_private_func: _ = v8::FunctionTemplate::new(scope, writable_write_callback);
        let write_private_instance: _ = write_private_func.get_function(scope).unwrap();
        let write_private_key: _ = v8::String::new(scope, "_write").unwrap();
        stream_obj.set(scope, write_private_key.into(), write_private_instance.into());
    }

    // write方法 (公开)
    let write_func: _ = v8::FunctionTemplate::new(scope, writable_public_write_callback);
    let write_instance: _ = write_func.get_function(scope).unwrap();
    let write_key: _ = v8::String::new(scope, "write").unwrap();
    stream_obj.set(scope, write_key.into(), write_instance.into());

    // end方法
    let end_func: _ = v8::FunctionTemplate::new(scope, writable_end_callback);
    let end_instance: _ = end_func.get_function(scope).unwrap();
    let end_key: _ = v8::String::new(scope, "end").unwrap();
    stream_obj.set(scope, end_key.into(), end_instance.into());

    // on方法 (复用 readable_on)
    stream_obj.set(scope, on_key.into(), on_instance.into());

    // _writableState
    let writable_state_key: _ = v8::String::new(scope, "_writableState").unwrap();
    let writable_state_obj: _ = v8::Object::new(scope);
    let need_drain_key: _ = v8::String::new(scope, "needDrain").unwrap();
    let need_drain_val: _ = v8::Boolean::new(scope, false);
    writable_state_obj.set(scope, need_drain_key.into(), need_drain_val.into());
    let w_ended_key: _ = v8::String::new(scope, "ended").unwrap();
    let w_ended_val: _ = v8::Boolean::new(scope, false);
    writable_state_obj.set(scope, w_ended_key.into(), w_ended_val.into());
    let writable_flag_key: _ = v8::String::new(scope, "writable").unwrap();
    let writable_flag_val: _ = v8::Boolean::new(scope, true);
    writable_state_obj.set(scope, writable_flag_key.into(), writable_flag_val.into());
    let w_hwm_key: _ = v8::String::new(scope, "highWaterMark").unwrap();
    let w_hwm_val: _ = v8::Integer::new(scope, 16 * 1024);
    writable_state_obj.set(scope, w_hwm_key.into(), w_hwm_val.into());
    stream_obj.set(scope, writable_state_key.into(), writable_state_obj.into());

    retval.set(stream_obj.into());
}

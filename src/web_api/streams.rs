// Web Streams API implementation for Web standard
// Stage 75: Web Streams API for AI workloads
// Provides ReadableStream, WritableStream, TransformStream, TextDecoderStream
//
// Optimized for streaming LLM responses and AI data processing pipelines
//
// v0.3.286: Enhanced TextDecoderStream with actual UTF-8 decoding

use anyhow::Result;
use rusty_v8 as v8;

// ============================================================
// ReadableStream Implementation
// ============================================================

/// ReadableStream constructor - enhanced with start() and enqueue support
/// Uses JavaScript arrays on the stream object for queue storage
fn readable_stream_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: v8::Local<v8::Object> = args.this();

    // Create JavaScript array to store enqueued chunks
    let queue_array = v8::Array::new(scope, 0);
    let queue_key = v8::String::new(scope, "_queue").unwrap();
    this.set(scope, queue_key.into(), queue_array.into());

    // Create state variable (0=open, 1=closed, 2=errored)
    let state_key = v8::String::new(scope, "_state").unwrap();
    let state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into(); // 0 = Open
    this.set(scope, state_key.into(), state_val);

    // Create index for reading from queue
    let read_index_key = v8::String::new(scope, "_readIndex").unwrap();
    let read_index_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into();
    this.set(scope, read_index_key.into(), read_index_val);

    // Check if first argument is an object with start method
    if args.length() > 0 {
        let underlying_source = args.get(0);
        if underlying_source.is_object() {
            let source_obj = v8::Local::<v8::Object>::try_from(underlying_source).unwrap();
            let start_key = v8::String::new(scope, "start").unwrap();
            if let Some(start_fn_val) = source_obj.get(scope, start_key.into()) {
                if start_fn_val.is_function() {
                    let start_fn = v8::Local::<v8::Function>::try_from(start_fn_val).unwrap();

                    // Create controller object with enqueue, close, error methods
                    let controller = v8::Object::new(scope);

                    // Store reference to stream on controller so methods can access it
                    let controller_stream_key = v8::String::new(scope, "_stream").unwrap();
                    controller.set(scope, controller_stream_key.into(), this.into());

                    // enqueue(chunk) - adds a chunk to the queue array on the stream
                    let enqueue_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                        if args.length() > 0 {
                            let chunk = args.get(0);
                            // Get the stream from controller (stored on controller)
                            let controller_obj = args.this();
                            let stream_key = v8::String::new(scope, "_stream").unwrap();
                            if let Some(stream_val) = controller_obj.get(scope, stream_key.into()) {
                                if let Ok(stream) = v8::Local::<v8::Object>::try_from(stream_val) {
                                    let queue_key = v8::String::new(scope, "_queue").unwrap();
                                    if let Some(queue_val) = stream.get(scope, queue_key.into()) {
                                        if let Ok(queue) = v8::Local::<v8::Array>::try_from(queue_val) {
                                            // Push to the JavaScript array
                                            let length = queue.length();
                                            queue.set_index(scope, length, chunk);
                                        }
                                    }
                                }
                            }
                        }
                    }).unwrap();
                    let enqueue_key = v8::String::new(scope, "enqueue").unwrap();
                    controller.set(scope, enqueue_key.into(), enqueue_fn.into());

                    // close() - sets state to closed
                    let close_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                        let controller_obj = args.this();
                        let stream_key = v8::String::new(scope, "_stream").unwrap();
                        let state_key = v8::String::new(scope, "_state").unwrap();
                        let state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 1).into(); // 1 = Closed
                        if let Some(stream_val) = controller_obj.get(scope, stream_key.into()) {
                            if let Ok(stream) = v8::Local::<v8::Object>::try_from(stream_val) {
                                stream.set(scope, state_key.into(), state_val);
                            }
                        }
                    }).unwrap();
                    let close_key = v8::String::new(scope, "close").unwrap();
                    controller.set(scope, close_key.into(), close_fn.into());

                    // error(e) - sets state to errored
                    let error_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                        let controller_obj = args.this();
                        let stream_key = v8::String::new(scope, "_stream").unwrap();
                        let state_key = v8::String::new(scope, "_state").unwrap();
                        let state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 2).into(); // 2 = Errored
                        if let Some(stream_val) = controller_obj.get(scope, stream_key.into()) {
                            if let Ok(stream) = v8::Local::<v8::Object>::try_from(stream_val) {
                                stream.set(scope, state_key.into(), state_val);
                            }
                        }
                    }).unwrap();
                    let error_key = v8::String::new(scope, "error").unwrap();
                    controller.set(scope, error_key.into(), error_fn.into());

                    // Call start(controller)
                    let undefined = v8::undefined(scope).into();
                    let _ = start_fn.call(scope, undefined, &[controller.into()]);
                }
            }
        }
    }

    // Setup getReader method
    let get_reader_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let reader: v8::Local<v8::Object> = v8::Object::new(_scope);

        // Store stream reference on reader object
        let stream_this = args.this();
        let stream_key = v8::String::new(_scope, "_stream").unwrap();
        reader.set(_scope, stream_key.into(), stream_this.into());

        // Setup read() method - returns Promise<{done, value}>
        let read_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();
            let result: v8::Local<v8::Object> = v8::Object::new(__scope);
            let done_key: v8::Local<v8::String> = v8::String::new(__scope, "done").unwrap();
            let value_key: v8::Local<v8::String> = v8::String::new(__scope, "value").unwrap();

            // Get stream from reader object
            let reader_this = _a.this();
            let stream_key = v8::String::new(__scope, "_stream").unwrap();
            if let Some(stream_val) = reader_this.get(__scope, stream_key.into()).filter(|s| s.is_object()) {
                if let Ok(stream) = v8::Local::<v8::Object>::try_from(stream_val) {
                    // Get queue from stream object
                    let queue_key = v8::String::new(__scope, "_queue").unwrap();
                    let state_key = v8::String::new(__scope, "_state").unwrap();
                    let idx_key = v8::String::new(__scope, "_readIndex").unwrap();

                    if let Some(queue_val) = stream.get(__scope, queue_key.into()) {
                        if let Ok(queue) = v8::Local::<v8::Array>::try_from(queue_val) {
                            // Get state
                            let state = stream.get(__scope, state_key.into())
                                .and_then(|s| s.to_integer(__scope))
                                .map(|i| i.value() as i32)
                                .unwrap_or(0);

                            // Get read index
                            let read_index = stream.get(__scope, idx_key.into())
                                .and_then(|i| i.to_integer(__scope))
                                .map(|i| i.value() as i32)
                                .unwrap_or(0);

                            let queue_length = queue.length();

                            // Priority 1: Has chunks in queue - return next one
                            if (read_index as u32) < queue_length {
                                let chunk = queue.get_index(__scope, read_index as u32).unwrap_or_else(|| v8::undefined(__scope).into());
                                // Increment read index
                                let new_idx: v8::Local<v8::Value> = v8::Integer::new(__scope, read_index + 1).into();
                                stream.set(__scope, idx_key.into(), new_idx);
                                let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, false).into();
                                result.set(__scope, done_key.into(), done_val);
                                result.set(__scope, value_key.into(), chunk);
                                let _ = promise.resolve(__scope, result.into());
                                _r.set(promise.into());
                                return;
                            }

                            // Priority 2: Stream closed or errored, no more chunks
                            if state == 1 || state == 2 {
                                let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, true).into();
                                let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
                                result.set(__scope, done_key.into(), done_val);
                                result.set(__scope, value_key.into(), undefined_val);
                                let _ = promise.resolve(__scope, result.into());
                                _r.set(promise.into());
                                return;
                            }

                            // Priority 3: No chunks yet, stream still open - wait
                            let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, true).into();
                            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
                            result.set(__scope, done_key.into(), done_val);
                            result.set(__scope, value_key.into(), undefined_val);
                            let _ = promise.resolve(__scope, result.into());
                            _r.set(promise.into());
                            return;
                        }
                    }
                }
            }

            // Default: stream done
            let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, true).into();
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            result.set(__scope, done_key.into(), done_val);
            result.set(__scope, value_key.into(), undefined_val);
            let _ = promise.resolve(__scope, result.into());
            _r.set(promise.into());
        });

        // Setup releaseLock() method
        let _release_fn = v8::FunctionTemplate::new(_scope, |_scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, _r: v8::ReturnValue| {
            // releaseLock logic - for basic implementation, does nothing
        });

        // Create closed Promise
        let closed_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let closed_promise_val: v8::Local<v8::Value> = closed_promise.into();
        let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
        let _ = closed_promise.resolve(_scope, undefined_val);

        // Add methods and properties to reader
        let read_key: v8::Local<v8::String> = v8::String::new(_scope, "read").unwrap();
        let release_key: v8::Local<v8::String> = v8::String::new(_scope, "releaseLock").unwrap();
        let closed_key: v8::Local<v8::String> = v8::String::new(_scope, "closed").unwrap();

        let read_func: v8::Local<v8::Function> = read_fn.get_function(_scope).unwrap();
        let release_func = v8::Function::new(_scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _r: v8::ReturnValue| {
            // Release lock - reset read index
            let reader_this = args.this();
            let stream_key = v8::String::new(_scope, "_stream").unwrap();
            let idx_key = v8::String::new(_scope, "_readIndex").unwrap();
            let zero_val: v8::Local<v8::Value> = v8::Integer::new(_scope, 0).into();
            if let Some(stream_val) = reader_this.get(_scope, stream_key.into()).filter(|s| s.is_object()) {
                if let Ok(stream) = v8::Local::<v8::Object>::try_from(stream_val) {
                    stream.set(_scope, idx_key.into(), zero_val);
                }
            }
        }).unwrap();

        reader.set(_scope, read_key.into(), read_func.into());
        reader.set(_scope, release_key.into(), release_func.into());
        reader.set(_scope, closed_key.into(), closed_promise_val);

        retval.set(reader.into());
    });

    let get_reader_key: v8::Local<v8::String> = v8::String::new(scope, "getReader").unwrap();
    let get_reader_func: v8::Local<v8::Function> = get_reader_fn.get_function(scope).unwrap();
    this.set(scope, get_reader_key.into(), get_reader_func.into());

    // v0.3.288: Setup pipeTo() method - pipes this readable to a writable stream
    let pipe_to_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        // Create a promise that will resolve when piping is complete
        let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();

        // Get the destination writable stream
        let dest_writable = args.get(0);
        if !dest_writable.is_object() {
            let reject_reason = v8::String::new(_scope, "Destination must be a WritableStream").unwrap();
            let _ = promise.reject(_scope, reject_reason.into());
            retval.set(promise.into());
            return;
        }

        // Store references for the pump operation
        let this_stream = args.this();
        let context = _scope.get_current_context();

        // Create JavaScript code for the pump operation
        let pump_code = r#"
            (function(readable, writable) {
                var reader = readable.getReader();
                var writer = writable.getWriter();

                function pump() {
                    return reader.read().then(function(result) {
                        if (result.done) {
                            return writer.close();
                        }
                        return writer.write(result.value).then(pump);
                    });
                }

                return pump();
            })
        "#;

        let pump_fn_str = v8::String::new(_scope, pump_code).unwrap();
        let pump_fn_val = v8::Script::compile(_scope, pump_fn_str, None).unwrap().run(_scope).unwrap();

        if let Ok(pump_fn) = v8::Local::<v8::Function>::try_from(pump_fn_val) {
            let undefined = v8::undefined(_scope).into();
            // Call the pump function with this stream and destination
            let this_stream_val: v8::Local<v8::Value> = this_stream.into();
            let dest_writable_val: v8::Local<v8::Value> = dest_writable.into();
            let result = pump_fn.call(_scope, undefined, &[this_stream_val, dest_writable_val]);

            // The promise we return will resolve when the pump completes
            // For now, just resolve immediately since the pump runs async
            let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
            let _ = promise.resolve(_scope, undefined_val);
        } else {
            let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
            let _ = promise.resolve(_scope, undefined_val);
        }

        retval.set(promise.into());
    });
    let pipe_to_key: v8::Local<v8::String> = v8::String::new(scope, "pipeTo").unwrap();
    let pipe_to_func: v8::Local<v8::Function> = pipe_to_fn.get_function(scope).unwrap();
    this.set(scope, pipe_to_key.into(), pipe_to_func.into());

    // v0.3.288: Setup pipeThrough() method - pipes this readable through a transform
    let pipe_through_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        // Get the transform stream
        let transform = args.get(0);
        if !transform.is_object() {
            let undefined = v8::undefined(_scope).into();
            retval.set(undefined);
            return;
        }

        let transform_obj = v8::Local::<v8::Object>::try_from(transform).unwrap();

        // Get the readable and writable properties from transform
        let readable_key = v8::String::new(_scope, "readable").unwrap();
        let writable_key = v8::String::new(_scope, "writable").unwrap();

        let transform_readable = transform_obj.get(_scope, readable_key.into());
        let transform_writable = transform_obj.get(_scope, writable_key.into());

        // Pipe this readable to transform's writable side using pipeTo
        let this_stream = args.this();
        let writable_val: v8::Local<v8::Value> = transform_writable.unwrap().into();

        // Use JavaScript to perform the pipeTo operation
        let pipe_code = r#"
            (function(readable, writable) {
                return readable.pipeTo(writable);
            })
        "#;

        let pipe_fn_str = v8::String::new(_scope, pipe_code).unwrap();
        if let Some(pipe_fn_val) = v8::Script::compile(_scope, pipe_fn_str, None).unwrap().run(_scope) {
            if let Ok(pipe_fn) = v8::Local::<v8::Function>::try_from(pipe_fn_val) {
                let this_val: v8::Local<v8::Value> = this_stream.into();
                let _ = pipe_fn.call(_scope, this_val, &[this_val, writable_val]);
            }
        }

        // Return the transform's readable side wrapped in an object
        let result_obj = v8::Object::new(_scope);
        if let Some(readable) = transform_readable {
            let readable_key = v8::String::new(_scope, "readable").unwrap();
            result_obj.set(_scope, readable_key.into(), readable);
        }
        retval.set(result_obj.into());
    });
    let pipe_through_key: v8::Local<v8::String> = v8::String::new(scope, "pipeThrough").unwrap();
    let pipe_through_func: v8::Local<v8::Function> = pipe_through_fn.get_function(scope).unwrap();
    this.set(scope, pipe_through_key.into(), pipe_through_func.into());

    // Setup locked property
    let locked_value: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
    let locked_key: v8::Local<v8::String> = v8::String::new(scope, "locked").unwrap();
    this.set(scope, locked_key.into(), locked_value);

    retval.set(this.into());
}

// ============================================================
// WritableStream Implementation
// ============================================================

/// WritableStream constructor - enhanced with start() callback and write queue
/// Uses JavaScript arrays on the stream object for write queue storage
fn writable_stream_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this: v8::Local<v8::Object> = args.this();

    // Create JavaScript array to store chunks pending write
    let write_queue_array = v8::Array::new(scope, 0);
    let write_queue_key = v8::String::new(scope, "_writeQueue").unwrap();
    this.set(scope, write_queue_key.into(), write_queue_array.into());

    // Create state variable (0=open, 1=closed, 2=errored)
    let state_key = v8::String::new(scope, "_state").unwrap();
    let state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into(); // 0 = Open
    this.set(scope, state_key.into(), state_val);

    // Create index for tracking write position
    let write_index_key = v8::String::new(scope, "_writeIndex").unwrap();
    let write_index_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into();
    this.set(scope, write_index_key.into(), write_index_val);

    // Store reference to the writable stream on itself for controller access
    let self_key = v8::String::new(scope, "_writable").unwrap();
    this.set(scope, self_key.into(), this.into());

    // Check if first argument is an object with start/write methods
    if args.length() > 0 {
        let underlying_sink = args.get(0);
        if underlying_sink.is_object() {
            let sink_obj = v8::Local::<v8::Object>::try_from(underlying_sink).unwrap();

            // Store write callback if provided
            let write_key = v8::String::new(scope, "write").unwrap();
            if let Some(write_fn_val) = sink_obj.get(scope, write_key.into()) {
                if write_fn_val.is_function() {
                    let write_cb_key = v8::String::new(scope, "_writeCallback").unwrap();
                    this.set(scope, write_cb_key.into(), write_fn_val);
                }
            }

            let start_key = v8::String::new(scope, "start").unwrap();
            if let Some(start_fn_val) = sink_obj.get(scope, start_key.into()) {
                if start_fn_val.is_function() {
                    let start_fn = v8::Local::<v8::Function>::try_from(start_fn_val).unwrap();

                    // Create controller object with writeable reference
                    let controller = v8::Object::new(scope);

                    // Store reference to writable stream on controller
                    let controller_writable_key = v8::String::new(scope, "_writable").unwrap();
                    controller.set(scope, controller_writable_key.into(), this.into());

                    // Call start(controller)
                    let undefined = v8::undefined(scope).into();
                    let _ = start_fn.call(scope, undefined, &[controller.into()]);
                }
            }
        }
    }

    // Setup getWriter method
    let get_writer_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let writer: v8::Local<v8::Object> = v8::Object::new(_scope);

        // Store reference to writable stream on writer
        let writable_this = _args.this();
        let writable_key = v8::String::new(_scope, "_writable").unwrap();
        writer.set(_scope, writable_key.into(), writable_this.into());

        // Setup write() method - adds chunk to queue and calls write callback
        let write_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();

            // Get writable from writer object
            let writer_this = args.this();
            let writable_key = v8::String::new(__scope, "_writable").unwrap();

            if let Some(writable_val) = writer_this.get(__scope, writable_key.into()).filter(|s| s.is_object()) {
                if let Ok(writable) = v8::Local::<v8::Object>::try_from(writable_val) {
                    // Get write queue and state
                    let queue_key = v8::String::new(__scope, "_writeQueue").unwrap();
                    let state_key = v8::String::new(__scope, "_state").unwrap();

                    // Get the stored write callback
                    let write_cb_key = v8::String::new(__scope, "_writeCallback").unwrap();
                    let write_callback = writable.get(__scope, write_cb_key.into());

                    if let Some(queue_val) = writable.get(__scope, queue_key.into()) {
                        if let Ok(queue) = v8::Local::<v8::Array>::try_from(queue_val) {
                            // Get state
                            let state = writable.get(__scope, state_key.into())
                                .and_then(|s| s.to_integer(__scope))
                                .map(|i| i.value() as i32)
                                .unwrap_or(0);

                            // Only add to queue if stream is open
                            if state == 0 && args.length() > 0 {
                                let chunk = args.get(0);
                                let length = queue.length();
                                queue.set_index(__scope, length, chunk);

                                // Call the user's write callback if provided
                                if let Some(cb) = write_callback {
                                    if cb.is_function() {
                                        if let Ok(cb_fn) = v8::Local::<v8::Function>::try_from(cb) {
                                            let undefined = v8::undefined(__scope).into();
                                            let _ = cb_fn.call(__scope, undefined, &[chunk]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Resolve promise immediately (async processing is simplified)
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        // Setup close() method
        let close_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();

            // Get writable from writer object
            let writer_this = args.this();
            let writable_key = v8::String::new(__scope, "_writable").unwrap();

            if let Some(writable_val) = writer_this.get(__scope, writable_key.into()).filter(|s| s.is_object()) {
                if let Ok(writable) = v8::Local::<v8::Object>::try_from(writable_val) {
                    let state_key = v8::String::new(__scope, "_state").unwrap();
                    let state_val: v8::Local<v8::Value> = v8::Integer::new(__scope, 1).into(); // 1 = Closed
                    writable.set(__scope, state_key.into(), state_val);
                }
            }

            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        // Setup abort() method
        let abort_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();

            // Get writable from writer object
            let writer_this = args.this();
            let writable_key = v8::String::new(__scope, "_writable").unwrap();

            if let Some(writable_val) = writer_this.get(__scope, writable_key.into()).filter(|s| s.is_object()) {
                if let Ok(writable) = v8::Local::<v8::Object>::try_from(writable_val) {
                    let state_key = v8::String::new(__scope, "_state").unwrap();
                    let state_val: v8::Local<v8::Value> = v8::Integer::new(__scope, 2).into(); // 2 = Errored
                    writable.set(__scope, state_key.into(), state_val);
                }
            }

            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        // Create ready Promise
        let ready_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let ready_promise_val: v8::Local<v8::Value> = ready_promise.into();
        let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
        let _ = ready_promise.resolve(_scope, undefined_val);

        // Create closed Promise
        let closed_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let closed_promise_val: v8::Local<v8::Value> = closed_promise.into();
        let _ = closed_promise.resolve(_scope, undefined_val);

        // Add methods and properties
        let write_key: v8::Local<v8::String> = v8::String::new(_scope, "write").unwrap();
        let close_key: v8::Local<v8::String> = v8::String::new(_scope, "close").unwrap();
        let abort_key: v8::Local<v8::String> = v8::String::new(_scope, "abort").unwrap();
        let ready_key: v8::Local<v8::String> = v8::String::new(_scope, "ready").unwrap();
        let closed_key: v8::Local<v8::String> = v8::String::new(_scope, "closed").unwrap();

        let write_func: v8::Local<v8::Function> = write_fn.get_function(_scope).unwrap();
        let close_func: v8::Local<v8::Function> = close_fn.get_function(_scope).unwrap();
        let abort_func: v8::Local<v8::Function> = abort_fn.get_function(_scope).unwrap();

        writer.set(_scope, write_key.into(), write_func.into());
        writer.set(_scope, close_key.into(), close_func.into());
        writer.set(_scope, abort_key.into(), abort_func.into());
        writer.set(_scope, ready_key.into(), ready_promise_val);
        writer.set(_scope, closed_key.into(), closed_promise_val);

        // Add desiredSize property
        let desired_size_key: v8::Local<v8::String> = v8::String::new(_scope, "desiredSize").unwrap();
        let desired_size_val: v8::Local<v8::Value> = v8::Number::new(_scope, 1.0).into(); // Default high-water mark
        writer.set(_scope, desired_size_key.into(), desired_size_val);

        retval.set(writer.into());
    });

    let get_writer_key: v8::Local<v8::String> = v8::String::new(scope, "getWriter").unwrap();
    let get_writer_func: v8::Local<v8::Function> = get_writer_fn.get_function(scope).unwrap();
    this.set(scope, get_writer_key.into(), get_writer_func.into());

    // Setup locked property (tracks if writer is attached)
    let locked_value: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
    let locked_key: v8::Local<v8::String> = v8::String::new(scope, "locked").unwrap();
    this.set(scope, locked_key.into(), locked_value);

    retval.set(this.into());
}

// ============================================================
// TransformStream Implementation
// ============================================================

/// TransformStream constructor - enhanced with transform() support
/// Connects writable stream writes to readable stream output via transformer
fn transform_stream_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let transform_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Create queue for transformed chunks (shared between writable and readable)
    let transform_queue = v8::Array::new(scope, 0);
    let queue_key = v8::String::new(scope, "_transformQueue").unwrap();
    transform_obj.set(scope, queue_key.into(), transform_queue.into());

    // Create read index for readable side
    let read_index_key = v8::String::new(scope, "_readIndex").unwrap();
    let read_index_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into();
    transform_obj.set(scope, read_index_key.into(), read_index_val);

    // Create state: 0=readable, 1=error, 2=closed
    let ts_state_key = v8::String::new(scope, "_transformState").unwrap();
    let ts_state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into();
    transform_obj.set(scope, ts_state_key.into(), ts_state_val);

    // Store transformer reference if provided
    if args.length() > 0 {
        let transformer = args.get(0);
        if transformer.is_object() {
            let transformer_obj = v8::Local::<v8::Object>::try_from(transformer).unwrap();

            // Check for transform() method
            let transform_key = v8::String::new(scope, "transform").unwrap();
            if let Some(transform_fn_val) = transformer_obj.get(scope, transform_key.into()) {
                if transform_fn_val.is_function() {
                    // Store transform function on transform object for closure access
                    let tf_key = v8::String::new(scope, "_transformFn").unwrap();
                    transform_obj.set(scope, tf_key.into(), transform_fn_val);
                }
            }

            // v0.3.287: Check for flush() method
            let flush_key = v8::String::new(scope, "flush").unwrap();
            if let Some(flush_fn_val) = transformer_obj.get(scope, flush_key.into()) {
                if flush_fn_val.is_function() {
                    // Store flush function for calling during close
                    let flush_key_name = v8::String::new(scope, "_flushFn").unwrap();
                    transform_obj.set(scope, flush_key_name.into(), flush_fn_val);
                }
            }

            // Create transform controller with enqueue method
            let controller = v8::Object::new(scope);
            let controller_queue_key = v8::String::new(scope, "_queue").unwrap();
            controller.set(scope, controller_queue_key.into(), transform_queue.into());
            let controller_state_key = v8::String::new(scope, "_state").unwrap();
            controller.set(scope, controller_state_key.into(), ts_state_val);
            let controller_readable_key = v8::String::new(scope, "_readable").unwrap();
            controller.set(scope, controller_readable_key.into(), transform_obj.into());

            // Store controller on transform object for closure access
            let ctrl_key = v8::String::new(scope, "_controller").unwrap();
            transform_obj.set(scope, ctrl_key.into(), controller.into());

            // enqueue(chunk) - adds to the transform queue
            let enqueue_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                if args.length() > 0 {
                    let chunk = args.get(0);
                    let ctrl = args.this();
                    let queue_key = v8::String::new(scope, "_queue").unwrap();
                    if let Some(queue_val) = ctrl.get(scope, queue_key.into()) {
                        if let Ok(queue) = v8::Local::<v8::Array>::try_from(queue_val) {
                            let length = queue.length();
                            queue.set_index(scope, length, chunk);
                        }
                    }
                }
            }).unwrap();
            let enqueue_key = v8::String::new(scope, "enqueue").unwrap();
            controller.set(scope, enqueue_key.into(), enqueue_fn.into());

            // close() - marks transform as done
            let close_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                let ctrl = args.this();
                let state_key = v8::String::new(scope, "_state").unwrap();
                let state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 2).into(); // 2 = Closed
                ctrl.set(scope, state_key.into(), state_val);
            }).unwrap();
            let close_key = v8::String::new(scope, "close").unwrap();
            controller.set(scope, close_key.into(), close_fn.into());

            // error(e) - marks transform as errored
            let error_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                let ctrl = args.this();
                let state_key = v8::String::new(scope, "_state").unwrap();
                let state_val: v8::Local<v8::Value> = v8::Integer::new(scope, 1).into(); // 1 = Errored
                ctrl.set(scope, state_key.into(), state_val);
            }).unwrap();
            let error_key = v8::String::new(scope, "error").unwrap();
            controller.set(scope, error_key.into(), error_fn.into());

            // Call start(controller) if provided
            let start_key = v8::String::new(scope, "start").unwrap();
            if let Some(start_fn_val) = transformer_obj.get(scope, start_key.into()) {
                if start_fn_val.is_function() {
                    let start_fn = v8::Local::<v8::Function>::try_from(start_fn_val).unwrap();
                    let undefined = v8::undefined(scope).into();
                    let _ = start_fn.call(scope, undefined, &[controller.into()]);
                }
            }
        }
    }

    // Create readable stream with getReader method
    let readable_stream: v8::Local<v8::Object> = v8::Object::new(scope);

    // Store transform reference on readable
    let readable_transform_key = v8::String::new(scope, "_transform").unwrap();
    readable_stream.set(scope, readable_transform_key.into(), transform_obj.into());

    // Create getReader template for readable stream
    let readable_get_reader_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let reader: v8::Local<v8::Object> = v8::Object::new(_scope);

        // Store transform reference on reader
        let reader_transform_key = v8::String::new(_scope, "_transform").unwrap();
        let this_stream = args.this();
        if let Some(transform_val) = this_stream.get(_scope, reader_transform_key.into()) {
            reader.set(_scope, reader_transform_key.into(), transform_val);
        }

        // Setup read() method - reads from transform queue
        let read_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();
            let result: v8::Local<v8::Object> = v8::Object::new(__scope);
            let done_key: v8::Local<v8::String> = v8::String::new(__scope, "done").unwrap();
            let value_key: v8::Local<v8::String> = v8::String::new(__scope, "value").unwrap();

            // Get transform from reader
            let reader_this = _a.this();
            let transform_key = v8::String::new(__scope, "_transform").unwrap();

            if let Some(transform_val) = reader_this.get(__scope, transform_key.into()).filter(|s| s.is_object()) {
                if let Ok(transform) = v8::Local::<v8::Object>::try_from(transform_val) {
                    let queue_key = v8::String::new(__scope, "_transformQueue").unwrap();
                    let idx_key = v8::String::new(__scope, "_readIndex").unwrap();
                    let state_key = v8::String::new(__scope, "_transformState").unwrap();

                    if let Some(queue_val) = transform.get(__scope, queue_key.into()) {
                        if let Ok(queue) = v8::Local::<v8::Array>::try_from(queue_val) {
                            let read_index = transform.get(__scope, idx_key.into())
                                .and_then(|i| i.to_integer(__scope))
                                .map(|i| i.value() as u32)
                                .unwrap_or(0);
                            let state = transform.get(__scope, state_key.into())
                                .and_then(|s| s.to_integer(__scope))
                                .map(|i| i.value() as i32)
                                .unwrap_or(0);
                            let queue_len = queue.length();

                            if read_index < queue_len {
                                let chunk = queue.get_index(__scope, read_index).unwrap_or_else(|| v8::undefined(__scope).into());
                                let new_idx: v8::Local<v8::Value> = v8::Integer::new(__scope, read_index as i32 + 1).into();
                                transform.set(__scope, idx_key.into(), new_idx);

                                let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, false).into();
                                result.set(__scope, done_key.into(), done_val);
                                result.set(__scope, value_key.into(), chunk);
                                let _ = promise.resolve(__scope, result.into());
                                _r.set(promise.into());
                                return;
                            }

                            // Check if transform is closed
                            if state == 2 {
                                let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, true).into();
                                let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
                                result.set(__scope, done_key.into(), done_val);
                                result.set(__scope, value_key.into(), undefined_val);
                                let _ = promise.resolve(__scope, result.into());
                                _r.set(promise.into());
                                return;
                            }
                        }
                    }
                }
            }

            // Default: not done, no value
            let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, true).into();
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            result.set(__scope, done_key.into(), done_val);
            result.set(__scope, value_key.into(), undefined_val);
            let _ = promise.resolve(__scope, result.into());
            _r.set(promise.into());
        });

        // Setup releaseLock() method
        let release_fn = v8::FunctionTemplate::new(_scope, |_scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, _r: v8::ReturnValue| {});

        // Create closed Promise
        let closed_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let closed_promise_val: v8::Local<v8::Value> = closed_promise.into();
        let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
        let _ = closed_promise.resolve(_scope, undefined_val);

        // Add methods and properties to reader
        let read_key: v8::Local<v8::String> = v8::String::new(_scope, "read").unwrap();
        let release_key: v8::Local<v8::String> = v8::String::new(_scope, "releaseLock").unwrap();
        let closed_key: v8::Local<v8::String> = v8::String::new(_scope, "closed").unwrap();

        let read_func: v8::Local<v8::Function> = read_fn.get_function(_scope).unwrap();
        let release_func: v8::Local<v8::Function> = release_fn.get_function(_scope).unwrap();

        reader.set(_scope, read_key.into(), read_func.into());
        reader.set(_scope, release_key.into(), release_func.into());
        reader.set(_scope, closed_key.into(), closed_promise_val);

        retval.set(reader.into());
    });
    readable_get_reader_fn.set_class_name(v8::String::new(scope, "TransformReadableStreamReader").unwrap());

    // Setup getReader on readable stream
    let readable_get_reader_key: v8::Local<v8::String> = v8::String::new(scope, "getReader").unwrap();
    if let Some(func) = readable_get_reader_fn.get_function(scope) {
        readable_stream.set(scope, readable_get_reader_key.into(), func.into());
    }

    let readable_locked_key: v8::Local<v8::String> = v8::String::new(scope, "locked").unwrap();
    let readable_locked_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
    readable_stream.set(scope, readable_locked_key.into(), readable_locked_val);

    // Create writable stream with getWriter method
    let writable_stream: v8::Local<v8::Object> = v8::Object::new(scope);

    // Store transform reference on writable
    let writable_transform_key = v8::String::new(scope, "_transform").unwrap();
    writable_stream.set(scope, writable_transform_key.into(), transform_obj.into());

    // Create getWriter template for writable stream
    let writable_get_writer_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let writer: v8::Local<v8::Object> = v8::Object::new(_scope);

        // Store transform reference on writer
        let writer_transform_key = v8::String::new(_scope, "_transform").unwrap();
        let this_writable = args.this();
        if let Some(transform_val) = this_writable.get(_scope, writer_transform_key.into()) {
            writer.set(_scope, writer_transform_key.into(), transform_val);
        }

        // Setup write() method - calls transform() and enqueues result
        // Transform function and controller are stored on transform object for closure access
        let write_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();

            // Get transform from writer
            let writer_this = a.this();
            let transform_key = v8::String::new(__scope, "_transform").unwrap();

            if let Some(transform_val) = writer_this.get(__scope, transform_key.into()).filter(|s| s.is_object()) {
                if let Ok(transform) = v8::Local::<v8::Object>::try_from(transform_val) {
                    // Get transform function from transform object
                    let tf_key = v8::String::new(__scope, "_transformFn").unwrap();
                    if let Some(tf_val) = transform.get(__scope, tf_key.into()) {
                        if let Ok(tf) = v8::Local::<v8::Function>::try_from(tf_val) {
                            // Get controller from transform
                            let ctrl_key = v8::String::new(__scope, "_controller").unwrap();
                            if let Some(ctrl_val) = transform.get(__scope, ctrl_key.into()) {
                                if let Ok(ctrl) = v8::Local::<v8::Object>::try_from(ctrl_val) {
                                    if a.length() > 0 {
                                        let chunk = a.get(0);
                                        let undefined = v8::undefined(__scope).into();
                                        let _ = tf.call(__scope, undefined, &[chunk, ctrl.into()]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        // v0.3.287: Setup close() method - calls flush callback before closing
        let close_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();

            // Get transform from writer
            let writer_this = a.this();
            let transform_key = v8::String::new(__scope, "_transform").unwrap();

            if let Some(transform_val) = writer_this.get(__scope, transform_key.into()).filter(|s| s.is_object()) {
                if let Ok(transform) = v8::Local::<v8::Object>::try_from(transform_val) {
                    // v0.3.287: Call flush callback if provided
                    let flush_key = v8::String::new(__scope, "_flushFn").unwrap();
                    if let Some(flush_val) = transform.get(__scope, flush_key.into()) {
                        if let Ok(flush) = v8::Local::<v8::Function>::try_from(flush_val) {
                            // Get controller
                            let ctrl_key = v8::String::new(__scope, "_controller").unwrap();
                            if let Some(ctrl_val) = transform.get(__scope, ctrl_key.into()) {
                                if let Ok(ctrl) = v8::Local::<v8::Object>::try_from(ctrl_val) {
                                    let undefined = v8::undefined(__scope).into();
                                    let _ = flush.call(__scope, undefined, &[ctrl.into()]);
                                }
                            }
                        }
                    }

                    // Mark transform as closed
                    let state_key = v8::String::new(__scope, "_transformState").unwrap();
                    let state_val: v8::Local<v8::Value> = v8::Integer::new(__scope, 2).into(); // 2 = Closed
                    transform.set(__scope, state_key.into(), state_val);
                }
            }

            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        // Setup abort() method
        let abort_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        // Create ready Promise
        let ready_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let ready_promise_val: v8::Local<v8::Value> = ready_promise.into();
        let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
        let _ = ready_promise.resolve(_scope, undefined_val);

        // Create closed Promise
        let closed_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let closed_promise_val: v8::Local<v8::Value> = closed_promise.into();
        let _ = closed_promise.resolve(_scope, undefined_val);

        // Add methods and properties
        let write_key: v8::Local<v8::String> = v8::String::new(_scope, "write").unwrap();
        let close_key: v8::Local<v8::String> = v8::String::new(_scope, "close").unwrap();
        let abort_key: v8::Local<v8::String> = v8::String::new(_scope, "abort").unwrap();
        let ready_key: v8::Local<v8::String> = v8::String::new(_scope, "ready").unwrap();
        let closed_key: v8::Local<v8::String> = v8::String::new(_scope, "closed").unwrap();

        let write_func: v8::Local<v8::Function> = write_fn.get_function(_scope).unwrap();
        let close_func: v8::Local<v8::Function> = close_fn.get_function(_scope).unwrap();
        let abort_func: v8::Local<v8::Function> = abort_fn.get_function(_scope).unwrap();

        writer.set(_scope, write_key.into(), write_func.into());
        writer.set(_scope, close_key.into(), close_func.into());
        writer.set(_scope, abort_key.into(), abort_func.into());
        writer.set(_scope, ready_key.into(), ready_promise_val);
        writer.set(_scope, closed_key.into(), closed_promise_val);

        let desired_size_key: v8::Local<v8::String> = v8::String::new(_scope, "desiredSize").unwrap();
        let desired_size_val: v8::Local<v8::Value> = v8::Number::new(_scope, 0.0).into();
        writer.set(_scope, desired_size_key.into(), desired_size_val);

        retval.set(writer.into());
    });

    // Setup getWriter on writable stream
    let writable_get_writer_key: v8::Local<v8::String> = v8::String::new(scope, "getWriter").unwrap();
    let writable_get_writer_func: v8::Local<v8::Function> = writable_get_writer_fn.get_function(scope).unwrap();
    writable_stream.set(scope, writable_get_writer_key.into(), writable_get_writer_func.into());

    let writable_locked_key: v8::Local<v8::String> = v8::String::new(scope, "locked").unwrap();
    let writable_locked_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
    writable_stream.set(scope, writable_locked_key.into(), writable_locked_val);

    // Add readable and writable properties
    let readable_key: v8::Local<v8::String> = v8::String::new(scope, "readable").unwrap();
    let writable_key: v8::Local<v8::String> = v8::String::new(scope, "writable").unwrap();

    transform_obj.set(scope, readable_key.into(), readable_stream.into());
    transform_obj.set(scope, writable_key.into(), writable_stream.into());

    retval.set(transform_obj.into());
}

// ============================================================
// TextDecoderStream Implementation
// ============================================================

/// TextDecoderStream constructor for streaming UTF-8 decoding
/// Uses TransformStream internally for AI workloads
fn text_decoder_stream_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Parse options
    let mut encoding = "utf-8".to_string();
    let mut fatal = false;
    let mut ignore_bom = false;

    if args.length() > 0 {
        let options = args.get(0);
        if !options.is_undefined() && options.is_object() {
            let options_obj = v8::Local::<v8::Object>::try_from(options).unwrap();
            let enc_key = v8::String::new(scope, "encoding").unwrap();
            if let Some(enc_val) = options_obj.get(scope, enc_key.into()) {
                if enc_val.is_string() {
                    encoding = enc_val.to_string(scope).unwrap().to_rust_string_lossy(scope).to_lowercase();
                }
            }
            let fatal_key = v8::String::new(scope, "fatal").unwrap();
            if let Some(fatal_val) = options_obj.get(scope, fatal_key.into()) {
                fatal = fatal_val.is_true();
            }
            let ignore_bom_key = v8::String::new(scope, "ignoreBOM").unwrap();
            if let Some(ignore_val) = options_obj.get(scope, ignore_bom_key.into()) {
                ignore_bom = ignore_val.is_true();
            }
        }
    }

    // Only support UTF-8 for now
    if encoding != "utf-8" && encoding != "utf8" {
        let error = v8::String::new(scope, &format!("TextDecoderStream: encoding '{}' not supported", encoding)).unwrap();
        let error_obj = v8::Exception::range_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let transform_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Create byte buffer for decoding
    let byte_buffer = v8::Array::new(scope, 0);
    let buffer_key = v8::String::new(scope, "_byteBuffer").unwrap();
    transform_obj.set(scope, buffer_key.into(), byte_buffer.into());

    // Create read index for output
    let read_index_key = v8::String::new(scope, "_readIndex").unwrap();
    let read_index_val: v8::Local<v8::Value> = v8::Integer::new(scope, 0).into();
    transform_obj.set(scope, read_index_key.into(), read_index_val);

    // Create output chunks queue for decoded strings
    let output_queue = v8::Array::new(scope, 0);
    let output_key = v8::String::new(scope, "_outputQueue").unwrap();
    transform_obj.set(scope, output_key.into(), output_queue.into());

    // Store ignoreBOM setting
    let ignore_bom_val: v8::Local<v8::Value> = v8::Boolean::new(scope, ignore_bom).into();
    let ignore_bom_key = v8::String::new(scope, "_ignoreBOM").unwrap();
    transform_obj.set(scope, ignore_bom_key.into(), ignore_bom_val);

    // Create readable stream with getReader
    let readable_stream: v8::Local<v8::Object> = v8::Object::new(scope);

    // Store decoder reference on readable (use non-hidden property name)
    let readable_decoder_key = v8::String::new(scope, "decoderRef").unwrap();
    readable_stream.set(scope, readable_decoder_key.into(), transform_obj.into());

    let readable_get_reader_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let reader: v8::Local<v8::Object> = v8::Object::new(_scope);

        // Store decoder on reader
        let this_obj = args.this();
        let decoder_key = v8::String::new(_scope, "decoderRef").unwrap();
        if let Some(decoder_val) = this_obj.get(_scope, decoder_key.into()) {
            let reader_decoder_key = v8::String::new(_scope, "decoderRef").unwrap();
            reader.set(_scope, reader_decoder_key.into(), decoder_val);
        }

        let read_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();
            let result: v8::Local<v8::Object> = v8::Object::new(__scope);
            let done_key: v8::Local<v8::String> = v8::String::new(__scope, "done").unwrap();
            let value_key: v8::Local<v8::String> = v8::String::new(__scope, "value").unwrap();

            // Get decoder from reader
            let reader_decoder_key = v8::String::new(__scope, "decoderRef").unwrap();
            if let Some(decoder_val) = _a.this().get(__scope, reader_decoder_key.into()) {
                if let Ok(decoder) = v8::Local::<v8::Object>::try_from(decoder_val) {
                    let output_queue_key = v8::String::new(__scope, "_outputQueue").unwrap();
                    let read_index_key = v8::String::new(__scope, "_readIndex").unwrap();

                    if let Some(output_val) = decoder.get(__scope, output_queue_key.into()) {
                        if let Ok(queue) = v8::Local::<v8::Array>::try_from(output_val) {
                            if let Some(index_val) = decoder.get(__scope, read_index_key.into()) {
                                let index = index_val.integer_value(__scope).unwrap_or(0) as u32;
                                let length = queue.length();

                                if index < length {
                                    let chunk_val = queue.get_index(__scope, index).unwrap();
                                    let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, false).into();
                                    result.set(__scope, done_key.into(), done_val);
                                    result.set(__scope, value_key.into(), chunk_val);

                                    let next_index: v8::Local<v8::Value> = v8::Integer::new(__scope, (index as i32) + 1).into();
                                    decoder.set(__scope, read_index_key.into(), next_index);
                                    let _ = promise.resolve(__scope, result.into());
                                    _r.set(promise.into());
                                    return;
                                }
                            }
                        }
                    }
                }
            }

            let done_val: v8::Local<v8::Value> = v8::Boolean::new(__scope, true).into();
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            result.set(__scope, done_key.into(), done_val);
            result.set(__scope, value_key.into(), undefined_val);
            let _ = promise.resolve(__scope, result.into());
            _r.set(promise.into());
        });

        let release_fn = v8::FunctionTemplate::new(_scope, |_scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, _r: v8::ReturnValue| {});

        let closed_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let closed_promise_val: v8::Local<v8::Value> = closed_promise.into();
        let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
        let _ = closed_promise.resolve(_scope, undefined_val);

        let read_key: v8::Local<v8::String> = v8::String::new(_scope, "read").unwrap();
        let release_key: v8::Local<v8::String> = v8::String::new(_scope, "releaseLock").unwrap();
        let closed_key: v8::Local<v8::String> = v8::String::new(_scope, "closed").unwrap();

        let read_func: v8::Local<v8::Function> = read_fn.get_function(_scope).unwrap();
        let release_func: v8::Local<v8::Function> = release_fn.get_function(_scope).unwrap();

        reader.set(_scope, read_key.into(), read_func.into());
        reader.set(_scope, release_key.into(), release_func.into());
        reader.set(_scope, closed_key.into(), closed_promise_val);

        retval.set(reader.into());
    });

    let readable_get_reader_key: v8::Local<v8::String> = v8::String::new(scope, "getReader").unwrap();
    let readable_get_reader_func: v8::Local<v8::Function> = readable_get_reader_fn.get_function(scope).unwrap();
    readable_stream.set(scope, readable_get_reader_key.into(), readable_get_reader_func.into());

    let readable_locked_key: v8::Local<v8::String> = v8::String::new(scope, "locked").unwrap();
    let readable_locked_val: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
    readable_stream.set(scope, readable_locked_key.into(), readable_locked_val);

    // Create writable stream
    let writable_stream: v8::Local<v8::Object> = v8::Object::new(scope);

    // Store decoder reference on writable (use non-hidden property name)
    let writable_decoder_key = v8::String::new(scope, "decoderRef").unwrap();
    writable_stream.set(scope, writable_decoder_key.into(), transform_obj.into());

    let writable_get_writer_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let writer: v8::Local<v8::Object> = v8::Object::new(_scope);

        // Store decoder on writer
        let this_obj = args.this();
        let decoder_key = v8::String::new(_scope, "decoderRef").unwrap();
        if let Some(decoder_val) = this_obj.get(_scope, decoder_key.into()) {
            let writer_decoder_key = v8::String::new(_scope, "decoderRef").unwrap();
            writer.set(_scope, writer_decoder_key.into(), decoder_val);
        }

        let write_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();

            // Get chunk to write
            let chunk = a.get(0);
            let mut bytes: Vec<u8> = Vec::new();

            if chunk.is_uint8_array() {
                let array = v8::Local::<v8::Uint8Array>::try_from(chunk).unwrap();
                let len = array.byte_length();
                bytes.resize(len, 0);
                array.copy_contents(&mut bytes);
            } else if chunk.is_string() {
                let chunk_str = chunk.to_string(__scope).unwrap().to_rust_string_lossy(__scope);
                bytes.extend_from_slice(chunk_str.as_bytes());
            }

            // Get decoder from writer
            let writer_decoder_key = v8::String::new(__scope, "decoderRef").unwrap();
            if let Some(decoder_val) = a.this().get(__scope, writer_decoder_key.into()) {
                if let Ok(decoder) = v8::Local::<v8::Object>::try_from(decoder_val) {
                    let buffer_key = v8::String::new(__scope, "_byteBuffer").unwrap();
                    let output_key = v8::String::new(__scope, "_outputQueue").unwrap();
                    let ignore_bom_key = v8::String::new(__scope, "_ignoreBOM").unwrap();

                    if let Some(buffer_val) = decoder.get(__scope, buffer_key.into()) {
                        if let Ok(buffer) = v8::Local::<v8::Array>::try_from(buffer_val) {
                            let buffer_len = buffer.length();
                            for (i, byte) in bytes.iter().enumerate() {
                                let byte_val: v8::Local<v8::Value> = v8::Integer::new(__scope, *byte as i32).into();
                                buffer.set_index(__scope, buffer_len + (i as u32), byte_val);
                            }

                            if let Some(output_val) = decoder.get(__scope, output_key.into()) {
                                if let Ok(output) = v8::Local::<v8::Array>::try_from(output_val) {
                                    let ignore_bom = if let Some(ib_val) = decoder.get(__scope, ignore_bom_key.into()) {
                                        ib_val.is_true()
                                    } else {
                                        false
                                    };

                                    let buffer_len = buffer.length();

                                    // Collect all bytes from buffer - using explicit loop to avoid E0524
                                    let mut all_bytes: Vec<u8> = Vec::new();
                                    for i in 0..buffer_len {
                                        if let Some(byte_val) = buffer.get_index(__scope, i) {
                                            if let Some(byte) = byte_val.integer_value(__scope) {
                                                all_bytes.push(byte as u8);
                                            }
                                        }
                                    }

                                    let decoded = if ignore_bom {
                                        let start = if all_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) { 3 } else { 0 };
                                        String::from_utf8_lossy(&all_bytes[start..]).into_owned()
                                    } else {
                                        String::from_utf8_lossy(&all_bytes).into_owned()
                                    };

                                    let output_len = output.length();
                                    let decoded_val = v8::String::new(__scope, &decoded).unwrap();
                                    output.set_index(__scope, output_len, decoded_val.into());

                                    let empty_buffer = v8::Array::new(__scope, 0);
                                    decoder.set(__scope, buffer_key.into(), empty_buffer.into());
                                }
                            }
                        }
                    }
                }
            }

            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        let close_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        let abort_fn = v8::FunctionTemplate::new(_scope, |__scope: &mut v8::HandleScope, _a: v8::FunctionCallbackArguments, mut _r: v8::ReturnValue| {
            let promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(__scope).unwrap();
            let undefined_val: v8::Local<v8::Value> = v8::undefined(__scope).into();
            let _ = promise.resolve(__scope, undefined_val);
            _r.set(promise.into());
        });

        let ready_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let ready_promise_val: v8::Local<v8::Value> = ready_promise.into();
        let undefined_val: v8::Local<v8::Value> = v8::undefined(_scope).into();
        let _ = ready_promise.resolve(_scope, undefined_val);

        let closed_promise: v8::Local<v8::PromiseResolver> = v8::PromiseResolver::new(_scope).unwrap();
        let closed_promise_val: v8::Local<v8::Value> = closed_promise.into();
        let _ = closed_promise.resolve(_scope, undefined_val);

        let write_key: v8::Local<v8::String> = v8::String::new(_scope, "write").unwrap();
        let close_key: v8::Local<v8::String> = v8::String::new(_scope, "close").unwrap();
        let abort_key: v8::Local<v8::String> = v8::String::new(_scope, "abort").unwrap();
        let ready_key: v8::Local<v8::String> = v8::String::new(_scope, "ready").unwrap();
        let closed_key: v8::Local<v8::String> = v8::String::new(_scope, "closed").unwrap();

        let write_func: v8::Local<v8::Function> = write_fn.get_function(_scope).unwrap();
        let close_func: v8::Local<v8::Function> = close_fn.get_function(_scope).unwrap();
        let abort_func: v8::Local<v8::Function> = abort_fn.get_function(_scope).unwrap();

        writer.set(_scope, write_key.into(), write_func.into());
        writer.set(_scope, close_key.into(), close_func.into());
        writer.set(_scope, abort_key.into(), abort_func.into());
        writer.set(_scope, ready_key.into(), ready_promise_val);
        writer.set(_scope, closed_key.into(), closed_promise_val);

        let desired_size_key: v8::Local<v8::String> = v8::String::new(_scope, "desiredSize").unwrap();
        let desired_size_val: v8::Local<v8::Value> = v8::Number::new(_scope, 0.0).into();
        writer.set(_scope, desired_size_key.into(), desired_size_val);

        retval.set(writer.into());
    });

    let writable_get_writer_key: v8::Local<v8::String> = v8::String::new(scope, "getWriter").unwrap();
    let writable_get_writer_func: v8::Local<v8::Function> = writable_get_writer_fn.get_function(scope).unwrap();
    writable_stream.set(scope, writable_get_writer_key.into(), writable_get_writer_func.into());

    let writable_locked_key: v8::Local<v8::String> = v8::String::new(scope, "locked").unwrap();
    let writable_locked_val: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
    writable_stream.set(scope, writable_locked_key.into(), writable_locked_val);

    // Add properties
    let encoding_value: v8::Local<v8::Value> = v8::String::new(scope, "utf-8").unwrap().into();
    let fatal_value: v8::Local<v8::Value> = v8::Boolean::new(scope, fatal).into();
    let ignore_bom_value: v8::Local<v8::Value> = v8::Boolean::new(scope, ignore_bom).into();

    let readable_key: v8::Local<v8::String> = v8::String::new(scope, "readable").unwrap();
    let writable_key: v8::Local<v8::String> = v8::String::new(scope, "writable").unwrap();
    let encoding_key: v8::Local<v8::String> = v8::String::new(scope, "encoding").unwrap();
    let fatal_key: v8::Local<v8::String> = v8::String::new(scope, "fatal").unwrap();
    let ignore_bom_key: v8::Local<v8::String> = v8::String::new(scope, "ignoreBOM").unwrap();

    transform_obj.set(scope, readable_key.into(), readable_stream.into());
    transform_obj.set(scope, writable_key.into(), writable_stream.into());
    transform_obj.set(scope, encoding_key.into(), encoding_value);
    transform_obj.set(scope, fatal_key.into(), fatal_value);
    transform_obj.set(scope, ignore_bom_key.into(), ignore_bom_value);

    retval.set(transform_obj.into());
}

// ============================================================
// Setup Functions
// ============================================================

/// Setup ReadableStream constructor in V8 context
fn setup_readable_stream(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: v8::Local<v8::Context>,
) {
    let global: v8::Local<v8::Object> = context.global(scope);
    let readable_stream_template: v8::Local<v8::FunctionTemplate> = v8::FunctionTemplate::new(scope, readable_stream_constructor);
    let readable_stream_constructor: v8::Local<v8::Function> = readable_stream_template.get_function(scope).unwrap();
    let readable_stream_key: v8::Local<v8::String> = v8::String::new(scope, "ReadableStream").unwrap();
    global.set(scope, readable_stream_key.into(), readable_stream_constructor.into());
}

/// Setup WritableStream constructor in V8 context
fn setup_writable_stream(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: v8::Local<v8::Context>,
) {
    let global: v8::Local<v8::Object> = context.global(scope);
    let writable_stream_template: v8::Local<v8::FunctionTemplate> = v8::FunctionTemplate::new(scope, writable_stream_constructor);
    let writable_stream_constructor: v8::Local<v8::Function> = writable_stream_template.get_function(scope).unwrap();
    let writable_stream_key: v8::Local<v8::String> = v8::String::new(scope, "WritableStream").unwrap();
    global.set(scope, writable_stream_key.into(), writable_stream_constructor.into());
}

/// Setup TransformStream constructor in V8 context
fn setup_transform_stream(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: v8::Local<v8::Context>,
) {
    let global: v8::Local<v8::Object> = context.global(scope);
    let transform_stream_template: v8::Local<v8::FunctionTemplate> = v8::FunctionTemplate::new(scope, transform_stream_constructor);
    let transform_stream_constructor: v8::Local<v8::Function> = transform_stream_template.get_function(scope).unwrap();
    let transform_stream_key: v8::Local<v8::String> = v8::String::new(scope, "TransformStream").unwrap();
    global.set(scope, transform_stream_key.into(), transform_stream_constructor.into());
}

/// Setup TextDecoderStream constructor in V8 context
fn setup_text_decoder_stream(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: v8::Local<v8::Context>,
) {
    let global: v8::Local<v8::Object> = context.global(scope);
    let text_decoder_stream_template: v8::Local<v8::FunctionTemplate> = v8::FunctionTemplate::new(scope, text_decoder_stream_constructor);
    let text_decoder_stream_constructor: v8::Local<v8::Function> = text_decoder_stream_template.get_function(scope).unwrap();
    let text_decoder_stream_key: v8::Local<v8::String> = v8::String::new(scope, "TextDecoderStream").unwrap();
    global.set(scope, text_decoder_stream_key.into(), text_decoder_stream_constructor.into());
}

/// Setup all Streams APIs in V8 context
pub fn setup_streams_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    eprintln!("[STAGE75] Setting up ReadableStream...");
    setup_readable_stream(scope, *context);
    eprintln!("[STAGE75] ReadableStream done");

    eprintln!("[STAGE75] Setting up WritableStream...");
    setup_writable_stream(scope, *context);
    eprintln!("[STAGE75] WritableStream done");

    eprintln!("[STAGE75] Setting up TransformStream...");
    setup_transform_stream(scope, *context);
    eprintln!("[STAGE75] TransformStream done");

    eprintln!("[STAGE75] Setting up TextDecoderStream...");
    setup_text_decoder_stream(scope, *context);
    eprintln!("[STAGE75] TextDecoderStream done");

    eprintln!("[STAGE75] All Streams APIs initialized!");
    Ok(())
}

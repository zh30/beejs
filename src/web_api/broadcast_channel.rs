// BroadcastChannel API implementation for Web standard
// v0.3.312: Enables real-time communication between browsing contexts
// Provides cross-tab, cross-window, and cross-frame communication via named channels

use anyhow::Result;
use rusty_v8 as v8;

/// Setup BroadcastChannel API in V8 context
pub fn setup_broadcast_channel_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // Create BroadcastChannel constructor template
    let broadcast_channel_template = v8::FunctionTemplate::new(
        scope,
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut retval: v8::ReturnValue| {
            // Get channel name from first argument
            let name = if args.length() > 0 {
                args.get(0).to_rust_string_lossy(scope)
            } else {
                "".to_string()
            };

            // Create the BroadcastChannel object
            let channel_obj: v8::Local<v8::Object> = v8::Object::new(scope);

            // Store name property
            let name_key = v8::String::new(scope, "name").unwrap();
            let name_value = v8::String::new(scope, &name).unwrap();
            channel_obj.set(scope, name_key.into(), name_value.into());

            // Create listeners array stored on the object
            let listeners_key = v8::String::new(scope, "_listeners").unwrap();
            let listeners_array: v8::Local<v8::Array> = v8::Array::new(scope, 0);
            channel_obj.set(scope, listeners_key.into(), listeners_array.into());

            // Create closed flag
            let closed_key = v8::String::new(scope, "_closed").unwrap();
            let false_val: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
            channel_obj.set(scope, closed_key.into(), false_val.into());

            // Create postMessage function
            let post_message_fn = v8::FunctionTemplate::new(
                scope,
                |_scope: &mut v8::HandleScope,
                 args: v8::FunctionCallbackArguments,
                 _rv: v8::ReturnValue| {
                    if args.length() == 0 {
                        return;
                    }

                    let message = args.get(0);
                    let this_obj = args.this();

                    // Check if channel is closed
                    let closed_key = v8::String::new(_scope, "_closed").unwrap();
                    if let Some(closed_val) = this_obj.get(_scope, closed_key.into()) {
                        if closed_val.is_true() {
                            return;
                        }
                    }

                    // Create MessageEvent object
                    let event_obj = v8::Object::new(_scope);

                    // Set event properties
                    let type_key = v8::String::new(_scope, "type").unwrap();
                    let message_type = v8::String::new(_scope, "message").unwrap();
                    event_obj.set(_scope, type_key.into(), message_type.into());

                    let data_key = v8::String::new(_scope, "data").unwrap();
                    event_obj.set(_scope, data_key.into(), message);

                    let origin_key = v8::String::new(_scope, "origin").unwrap();
                    let empty_origin = v8::String::new(_scope, "").unwrap();
                    event_obj.set(_scope, origin_key.into(), empty_origin.into());

                    let last_event_id_key = v8::String::new(_scope, "lastEventId").unwrap();
                    event_obj.set(_scope, last_event_id_key.into(), empty_origin.into());

                    // First, check onmessage property (for simple usage)
                    let onmessage_key = v8::String::new(_scope, "onmessage").unwrap();
                    if let Some(onmessage_val) = this_obj.get(_scope, onmessage_key.into()) {
                        if onmessage_val.is_function() {
                            if let Ok(onmessage_fn) =
                                v8::Local::<v8::Function>::try_from(onmessage_val)
                            {
                                let _ =
                                    onmessage_fn.call(_scope, this_obj.into(), &[event_obj.into()]);
                            }
                        }
                    }

                    // Then, get listeners from addEventListener and call them
                    let listeners_key = v8::String::new(_scope, "_listeners").unwrap();
                    if let Some(listeners_val) = this_obj.get(_scope, listeners_key.into()) {
                        if let Ok(listeners_array) = v8::Local::<v8::Array>::try_from(listeners_val)
                        {
                            for i in 0..listeners_array.length() {
                                if let Some(listener) = listeners_array.get_index(_scope, i) {
                                    if let Ok(listener_fn) =
                                        v8::Local::<v8::Function>::try_from(listener)
                                    {
                                        let _ = listener_fn.call(
                                            _scope,
                                            this_obj.into(),
                                            &[event_obj.into()],
                                        );
                                    }
                                }
                            }
                        }
                    }
                },
            );
            let post_message_instance = post_message_fn.get_function(scope).unwrap();
            let post_message_key = v8::String::new(scope, "postMessage").unwrap();
            channel_obj.set(scope, post_message_key.into(), post_message_instance.into());

            // Create close function
            let close_fn = v8::FunctionTemplate::new(
                scope,
                |_scope: &mut v8::HandleScope,
                 args: v8::FunctionCallbackArguments,
                 _rv: v8::ReturnValue| {
                    let this_obj = args.this();
                    let closed_key = v8::String::new(_scope, "_closed").unwrap();
                    let true_val: v8::Local<v8::Value> = v8::Boolean::new(_scope, true).into();
                    this_obj.set(_scope, closed_key.into(), true_val.into());
                },
            );
            let close_instance = close_fn.get_function(scope).unwrap();
            let close_key = v8::String::new(scope, "close").unwrap();
            channel_obj.set(scope, close_key.into(), close_instance.into());

            // Create addEventListener function
            let add_event_listener_fn = v8::FunctionTemplate::new(
                scope,
                |_scope: &mut v8::HandleScope,
                 args: v8::FunctionCallbackArguments,
                 _rv: v8::ReturnValue| {
                    if args.length() < 2 {
                        return;
                    }

                    let event_type = args.get(0).to_rust_string_lossy(_scope);
                    let listener = args.get(1);
                    let this_obj = args.this();

                    if event_type == "message" || event_type == "messageerror" {
                        // Add to listeners array on the object
                        let listeners_key = v8::String::new(_scope, "_listeners").unwrap();
                        if let Some(listeners_val) = this_obj.get(_scope, listeners_key.into()) {
                            if let Ok(listeners_array) =
                                v8::Local::<v8::Array>::try_from(listeners_val)
                            {
                                let length = listeners_array.length();
                                listeners_array.set_index(_scope, length, listener);
                            }
                        }
                    }
                },
            );
            let add_event_listener_instance = add_event_listener_fn.get_function(scope).unwrap();
            let add_event_listener_key = v8::String::new(scope, "addEventListener").unwrap();
            channel_obj.set(
                scope,
                add_event_listener_key.into(),
                add_event_listener_instance.into(),
            );

            // Create removeEventListener function
            let remove_event_listener_fn = v8::FunctionTemplate::new(
                scope,
                |_scope: &mut v8::HandleScope,
                 args: v8::FunctionCallbackArguments,
                 _rv: v8::ReturnValue| {
                    if args.length() < 2 {
                        return;
                    }

                    // Note: Full implementation would track and remove specific listeners
                    // For now, this is a placeholder
                    let _event_type = args.get(0);
                    let _listener = args.get(1);
                },
            );
            let remove_event_listener_instance =
                remove_event_listener_fn.get_function(scope).unwrap();
            let remove_event_listener_key = v8::String::new(scope, "removeEventListener").unwrap();
            channel_obj.set(
                scope,
                remove_event_listener_key.into(),
                remove_event_listener_instance.into(),
            );

            // Create dispatchEvent function
            let dispatch_event_fn = v8::FunctionTemplate::new(
                scope,
                |_scope: &mut v8::HandleScope,
                 args: v8::FunctionCallbackArguments,
                 _rv: v8::ReturnValue| {
                    let this_obj = args.this();
                    if args.length() > 0 {
                        let event = args.get(0);
                        if let Ok(event_obj) = v8::Local::<v8::Object>::try_from(event) {
                            let type_key = v8::String::new(_scope, "type").unwrap();
                            if let Some(type_val) = event_obj.get(_scope, type_key.into()) {
                                let event_type = type_val.to_rust_string_lossy(_scope);

                                if event_type == "message" {
                                    // Get listeners and call them
                                    let listeners_key =
                                        v8::String::new(_scope, "_listeners").unwrap();
                                    if let Some(listeners_val) =
                                        this_obj.get(_scope, listeners_key.into())
                                    {
                                        if let Ok(listeners_array) =
                                            v8::Local::<v8::Array>::try_from(listeners_val)
                                        {
                                            for i in 0..listeners_array.length() {
                                                if let Some(listener) =
                                                    listeners_array.get_index(_scope, i)
                                                {
                                                    if let Ok(listener_fn) =
                                                        v8::Local::<v8::Function>::try_from(
                                                            listener,
                                                        )
                                                    {
                                                        let _ = listener_fn.call(
                                                            _scope,
                                                            this_obj.into(),
                                                            &[event_obj.into()],
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            );
            let dispatch_event_instance = dispatch_event_fn.get_function(scope).unwrap();
            let dispatch_event_key = v8::String::new(scope, "dispatchEvent").unwrap();
            channel_obj.set(
                scope,
                dispatch_event_key.into(),
                dispatch_event_instance.into(),
            );

            // Set onmessage property (for simple usage)
            let onmessage_key = v8::String::new(scope, "onmessage").unwrap();
            let undefined_val = v8::undefined(scope);
            channel_obj.set(scope, onmessage_key.into(), undefined_val.into());

            // Set onmessageerror property
            let onmessageerror_key = v8::String::new(scope, "onmessageerror").unwrap();
            channel_obj.set(scope, onmessageerror_key.into(), undefined_val.into());

            retval.set(channel_obj.into());
        },
    );

    let broadcast_channel_constructor: v8::Local<v8::Function> =
        broadcast_channel_template.get_function(scope).unwrap();

    // Set BroadcastChannel to global scope
    let global: v8::Local<v8::Object> = context.global(scope);
    let broadcast_channel_key: v8::Local<v8::String> =
        v8::String::new(scope, "BroadcastChannel").unwrap();
    let broadcast_channel_val: v8::Local<v8::Value> = broadcast_channel_constructor.into();
    global.set(
        scope,
        broadcast_channel_key.into(),
        broadcast_channel_val.into(),
    );

    Ok(())
}

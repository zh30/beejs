//! AbortController API implementation
use anyhow::Result;
use rusty_v8 as v8;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
pub fn setup_abort_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let abort_controller_template: _ = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let abort_controller_obj: _ = v8::Object::new(scope);
        
        let signal_key: _ = v8::String::new(scope, "signal").unwrap();
        let signal_obj: _ = v8::Object::new(scope);
        let aborted_key: _ = v8::String::new(scope, "aborted").unwrap();
        let aborted_val: _ = v8::Boolean::new(scope, false).into();
        signal_obj.set(scope, aborted_key.into(), aborted_val);
        abort_controller_obj.set(scope, signal_key.into(), signal_obj.into());
        let abort_key: _ = v8::String::new(scope, "abort").unwrap();
        let abort_func: _ = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            println!("AbortController.abort called");
        });
        let abort_func_instance: _ = abort_func.get_function(scope).unwrap();
        abort_controller_obj.set(scope, abort_key.into(), abort_func_instance.into());
        
        retval.set(abort_controller_obj.into());
    });
    
    let abort_controller_constructor: _ = abort_controller_template.get_function(scope).unwrap();
    
    let global: _ = context.global(scope);
    let abort_controller_key: _ = v8::String::new(scope, "AbortController").unwrap();
    let abort_controller_val: _ = abort_controller_constructor.into();
    global.set(scope, abort_controller_key.into(), abort_controller_val);
    
    Ok(())
}
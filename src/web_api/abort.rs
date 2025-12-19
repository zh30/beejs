//! AbortController API implementation

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_abort_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let abort_controller_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let abort_controller_obj = v8::Object::new(scope);
        
        let signal_key = v8::String::new(scope, "signal").unwrap();
        let signal_obj = v8::Object::new(scope);
        let aborted_key = v8::String::new(scope, "aborted").unwrap();
        signal_obj.set(scope, aborted_key.into(), v8::Boolean::new(scope, false).into());
        abort_controller_obj.set(scope, signal_key.into(), signal_obj.into());
        
        let abort_key = v8::String::new(scope, "abort").unwrap();
        let abort_func = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            println!("AbortController.abort called");
        });
        abort_controller_obj.set(scope, abort_key.into(), abort_func.get_function(scope).unwrap().into());
        
        retval.set(abort_controller_obj.into());
    });
    
    let abort_controller_constructor = abort_controller_template.get_function(scope).unwrap();
    
    let global = context.global(scope);
    let abort_controller_key = v8::String::new(scope, "AbortController").unwrap();
    global.set(scope, abort_controller_key.into(), abort_controller_constructor.into());
    
    Ok(())
}

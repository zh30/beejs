//! http polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let http_key = v8::String::new(scope, "http").unwrap();
    let http_obj = v8::Object::new(scope);
    
    // Get
    let get_fn = v8::Function::new(scope, get).unwrap();
    http_obj.set(scope, "get".into(), get_fn.into());
    
    global.set(scope, http_key.into(), http_obj.into());
}

fn get(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let url = args.get(0).to_string(scope).unwrap().to_rust_string();
    
    // Simple async wrapper
    let result = v8::Object::new(scope);
    result.set(scope, "statusCode".into(), v8::Integer::new(scope, 200).into());
    result.set(scope, "data".into(), v8::String::new(scope, "Mock response").unwrap().into());
    
    retval.set(result.into());
}

//! http polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let http_key = v8::String::new(scope, "http").unwrap();
    let http_obj = v8::Object::new(scope);

    // Get
    let get_fn = v8::FunctionTemplate::new(scope, get).get_function(scope).unwrap();
    let get_key = v8::String::new(scope, "get").unwrap().into();
    http_obj.set(scope, get_key, get_fn.into());

    global.set(scope, http_key.into(), http_obj.into());
}

fn get(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let url_arg = args.get(0);
    let _url = url_arg.to_string(scope).unwrap().to_rust_string_lossy(scope);

    // Simple async wrapper
    let result = v8::Object::new(scope);
    let status_key = v8::String::new(scope, "statusCode").unwrap();
    let status_val = v8::Integer::new(scope, 200).into();
    result.set(scope, status_key.into(), status_val);

    let data_key = v8::String::new(scope, "data").unwrap();
    let data_val = v8::String::new(scope, "Mock response").unwrap().into();
    result.set(scope, data_key.into(), data_val);

    retval.set(result.into());
}

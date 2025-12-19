//! util polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let util_key = v8::String::new(scope, "util").unwrap();
    let util_obj = v8::Object::new(scope);
    
    // Inspect
    let inspect_fn = v8::Function::new(scope, inspect).unwrap();
    util_obj.set(scope, "inspect".into(), inspect_fn.into());
    
    global.set(scope, util_key.into(), util_obj.into());
}

fn inspect(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let obj = args.get(0);
    let result = obj.to_string(scope).unwrap().to_rust_string();
    retval.set(v8::String::new(scope, &result).unwrap().into());
}

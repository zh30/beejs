//! querystring polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let qs_key = v8::String::new(scope, "querystring").unwrap();
    let qs_obj = v8::Object::new(scope);
    
    // Parse query string
    let parse_fn = v8::Function::new(scope, parse).unwrap();
    qs_obj.set(scope, "parse".into(), parse_fn.into());
    
    // Stringify
    let stringify_fn = v8::Function::new(scope, stringify).unwrap();
    qs_obj.set(scope, "stringify".into(), stringify_fn.into());
    
    global.set(scope, qs_key.into(), qs_obj.into());
}

fn parse(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let query_str = args.get(0).to_string(scope).unwrap().to_rust_string();
    let obj = v8::Object::new(scope);
    
    for pair in query_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            obj.set(scope,
                v8::String::new(scope, key).unwrap().into(),
                v8::String::new(scope, value).unwrap().into()
            );
        }
    }
    
    retval.set(obj.into());
}

fn stringify(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let obj = args.get(0).to_object(scope).unwrap();
    let keys = obj.get_own_property_names(scope).unwrap();
    
    let mut parts = Vec::new();
    for i in 0..keys.length() {
        let key = keys.get_index(scope, i).unwrap().to_string(scope).unwrap().to_rust_string();
        let value = obj.get(scope, key.as_str().into()).unwrap().to_string(scope).unwrap().to_rust_string();
        parts.push(format!("{}={}", key, value));
    }
    
    retval.set(v8::String::new(scope, &parts.join("&")).unwrap().into());
}

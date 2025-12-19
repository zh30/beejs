//! path polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let path_key = v8::String::new(scope, "path").unwrap();
    let path_obj = v8::Object::new(scope);
    
    // Join paths
    let join_fn = v8::Function::new(scope, join).unwrap();
    path_obj.set(scope, "join".into(), join_fn.into());
    
    // Resolve
    let resolve_fn = v8::Function::new(scope, resolve).unwrap();
    path_obj.set(scope, "resolve".into(), resolve_fn.into());
    
    // Basename
    let basename_fn = v8::Function::new(scope, basename).unwrap();
    path_obj.set(scope, "basename".into(), basename_fn.into());
    
    global.set(scope, path_key.into(), path_obj.into());
}

fn join(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let mut result = String::new();
    
    for i in 0..args.length() {
        let arg = args.get(i).to_string(scope).unwrap().to_rust_string();
        if i > 0 && !result.ends_with('/') && !arg.starts_with('/') {
            result.push('/');
        }
        result.push_str(&arg);
    }
    
    retval.set(v8::String::new(scope, &result).unwrap().into());
}

fn resolve(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let mut paths = Vec::new();
    
    for i in 0..args.length() {
        paths.push(args.get(i).to_string(scope).unwrap().to_rust_string());
    }
    
    let result = std::path::Path::new(&paths.join("/"))
        .canonicalize()
        .unwrap_or_else(|_| std::path::Path::new(&paths.join("/")).to_path_buf());
    
    retval.set(v8::String::new(scope, &result.to_string_lossy()).unwrap().into());
}

fn basename(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let path_str = args.get(0).to_string(scope).unwrap().to_rust_string();
    let path = std::path::Path::new(&path_str);
    
    if let Some(file_name) = path.file_name() {
        retval.set(v8::String::new(scope, &file_name.to_string_lossy()).unwrap().into());
    } else {
        retval.set(v8::String::new(scope, "").unwrap().into());
    }
}

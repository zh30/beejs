//! fs (file system) polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let fs_key = v8::String::new(scope, "fs").unwrap();
    let fs_obj = v8::Object::new(scope);
    
    // Read file
    let read_file_fn = v8::Function::new(scope, read_file).unwrap();
    fs_obj.set(scope, "readFile".into(), read_file_fn.into());
    
    // Write file
    let write_file_fn = v8::Function::new(scope, write_file).unwrap();
    fs_obj.set(scope, "writeFile".into(), write_file_fn.into());
    
    // Exists
    let exists_fn = v8::Function::new(scope, exists).unwrap();
    fs_obj.set(scope, "existsSync".into(), exists_fn.into());
    
    global.set(scope, fs_key.into(), fs_obj.into());
}

fn read_file(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let path = args.get(0).to_string(scope).unwrap().to_rust_string();
    
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let content_v8 = v8::String::new(scope, &content).unwrap();
            retval.set(content_v8.into());
        }
        Err(_) => {
            retval.set(v8::Null::new(scope).into());
        }
    }
}

fn write_file(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let path = args.get(0).to_string(scope).unwrap().to_rust_string();
    let content = args.get(1).to_string(scope).unwrap().to_rust_string();
    
    match std::fs::write(&path, content) {
        Ok(_) => {
            retval.set(v8::Undefined::new(scope).into());
        }
        Err(_) => {
            retval.set(v8::Undefined::new(scope).into());
        }
    }
}

fn exists(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let path = args.get(0).to_string(scope).unwrap().to_rust_string();
    
    let exists = std::path::Path::new(&path).exists();
    retval.set(v8::Boolean::new(scope, exists).into());
}

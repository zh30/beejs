//! crypto polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let crypto_key = v8::String::new(scope, "crypto").unwrap();
    let crypto_obj = v8::Object::new(scope);
    
    // Random bytes
    let random_fn = v8::Function::new(scope, random_bytes).unwrap();
    crypto_obj.set(scope, "randomBytes".into(), random_fn.into());
    
    global.set(scope, crypto_key.into(), crypto_obj.into());
}

fn random_bytes(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let size = args.get(0).int32_value(scope).unwrap_or(0) as usize;
    
    let mut bytes = vec![0u8; size];
    // getrandom::getrandom(&mut bytes).unwrap_or(());
    
    let buffer = v8::ArrayBuffer::new(scope, size);
    let view = unsafe { v8::Local::<v8::Uint8Array>::view_unchecked(&buffer) };
    view.set_contents_of_elements(bytes.as_slice());
    
    retval.set(view.into());
}

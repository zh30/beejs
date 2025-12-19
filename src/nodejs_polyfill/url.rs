//! url polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let url_key = v8::String::new(scope, "url").unwrap();
    let url_obj = v8::Object::new(scope);

    // Parse URL
    let parse_fn = v8::Function::new(scope, parse).unwrap();
    url_obj.set(scope, "parse".into(), parse_fn.into());

    global.set(scope, url_key.into(), url_obj.into());
}

fn parse(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let url_str = args.get(0).to_string(scope).unwrap().to_rust_string();
    
    match url::Url::parse(&url_str) {
        Ok(url) => {
            let url_obj = v8::Object::new(scope);
            url_obj.set(scope, "href".into(), v8::String::new(scope, url.as_str()).unwrap().into());
            url_obj.set(scope, "protocol".into(), v8::String::new(scope, url.scheme()).unwrap().into());
            url_obj.set(scope, "hostname".into(), v8::String::new(scope, url.host_str().unwrap_or("")).unwrap().into());
            url_obj.set(scope, "port".into(), v8::String::new(scope, url.port().map(|p| p.to_string()).unwrap_or_default()).unwrap().into());
            url_obj.set(scope, "pathname".into(), v8::String::new(scope, url.path()).unwrap().into());
            url_obj.set(scope, "search".into(), v8::String::new(scope, url.query().unwrap_or("")).unwrap().into());
            retval.set(url_obj.into());
        }
        Err(_) => {
            retval.set(v8::Null::new(scope).into());
        }
    }
}

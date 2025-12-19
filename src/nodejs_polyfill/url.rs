//! url polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let url_key = v8::String::new(scope, "url").unwrap();
    let url_obj = v8::Object::new(scope);

    // Parse URL
    let parse_fn = v8::FunctionTemplate::new(scope, parse).get_function(scope).unwrap();
    let parse_key = v8::String::new(scope, "parse").unwrap().into();
    url_obj.set(scope, parse_key, parse_fn.into());

    global.set(scope, url_key.into(), url_obj.into());
}

fn parse(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let url_arg = args.get(0);
    let url_str = url_arg.to_string(scope).unwrap().to_rust_string_lossy(scope);

    match url::Url::parse(&url_str) {
        Ok(url) => {
            let url_obj = v8::Object::new(scope);
            let href_key = v8::String::new(scope, "href").unwrap();
            let href_val = v8::String::new(scope, url.as_str()).unwrap().into();
            url_obj.set(scope, href_key.into(), href_val);

            let protocol_key = v8::String::new(scope, "protocol").unwrap();
            let protocol_val = v8::String::new(scope, url.scheme()).unwrap().into();
            url_obj.set(scope, protocol_key.into(), protocol_val);

            let hostname_key = v8::String::new(scope, "hostname").unwrap();
            let hostname_val = v8::String::new(scope, url.host_str().unwrap_or("")).unwrap().into();
            url_obj.set(scope, hostname_key.into(), hostname_val);

            let port_key = v8::String::new(scope, "port").unwrap();
            let port_val = v8::String::new(scope, &url.port().map(|p| p.to_string()).unwrap_or_default()).unwrap().into();
            url_obj.set(scope, port_key.into(), port_val);

            let pathname_key = v8::String::new(scope, "pathname").unwrap();
            let pathname_val = v8::String::new(scope, url.path()).unwrap().into();
            url_obj.set(scope, pathname_key.into(), pathname_val);

            let search_key = v8::String::new(scope, "search").unwrap();
            let search_val = v8::String::new(scope, url.query().unwrap_or("")).unwrap().into();
            url_obj.set(scope, search_key.into(), search_val);

            retval.set(url_obj.into());
        }
        Err(_) => {
            retval.set(v8::null(scope).into());
        }
    }
}

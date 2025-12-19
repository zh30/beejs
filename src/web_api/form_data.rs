//! FormData API implementation

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_form_data_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let form_data_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let form_data_obj = v8::Object::new(scope);
        let proto = v8::Object::new(scope);
        
        let append_key = v8::String::new(scope, "append").unwrap();
        let append_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let name = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
            println!("FormData.append: {}", name);
        });
        let append_func_instance = append_func.get_function(scope).unwrap();
        proto.set(scope, append_key.into(), append_func_instance.into());
        
        form_data_obj.set_prototype(scope, proto.into());
        retval.set(form_data_obj.into());
    });
    
    let form_data_constructor = form_data_template.get_function(scope).unwrap();
    
    let global = context.global(scope);
    let form_data_key = v8::String::new(scope, "FormData").unwrap();
    let form_data_val = form_data_constructor.into();
    global.set(scope, form_data_key.into(), form_data_val);
    
    Ok(())
}

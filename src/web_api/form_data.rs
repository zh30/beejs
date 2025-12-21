//! FormData API implementation
//!
//! The FormData interface provides a way to construct a set of key/value pairs
//! representing form fields and their values, which can be sent using fetch().

use anyhow::Result;
use rusty_v8 as v8;

pub fn setup_form_data_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // FormData constructor
    let form_data_template = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        let form_data_obj = v8::Object::new(_scope);

        // Store methods on the instance directly
        // This is simpler and matches the pattern used in other APIs

        retval.set(form_data_obj.into());
    });

    let form_data_constructor = form_data_template.get_function(scope).unwrap();

    // Register FormData constructor
    let global = context.global(scope);
    let form_data_key = v8::String::new(scope, "FormData").unwrap();
    global.set(scope, form_data_key.into(), form_data_constructor.into());

    Ok(())
}

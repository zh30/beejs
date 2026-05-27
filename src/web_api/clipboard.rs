// Clipboard API implementation
// Provides clipboard read/write capability for AI workloads
// that need to copy/paste content
//
// v0.3.342: Initial implementation of Clipboard API

use anyhow::Result;
use once_cell::sync::Lazy;
use rusty_v8 as v8;
use std::sync::Mutex;

/// Global clipboard state (in-memory storage for non-browser environment)
static CLIPBOARD_CONTENT: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// Set up Clipboard API in the V8 context
pub fn setup_clipboard_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create navigator property if it doesn't exist
    let navigator_key = v8::String::new(scope, "navigator").unwrap();
    let navigator_val = global.get(scope, navigator_key.into());

    // Create or get navigator object
    let navigator_obj = if let Some(val) = navigator_val {
        if val.is_object() {
            v8::Local::<v8::Object>::try_from(val).unwrap()
        } else {
            let new_navigator: v8::Local<v8::Object> = v8::Object::new(scope);
            global.set(scope, navigator_key.into(), new_navigator.into());
            new_navigator
        }
    } else {
        let new_navigator: v8::Local<v8::Object> = v8::Object::new(scope);
        global.set(scope, navigator_key.into(), new_navigator.into());
        new_navigator
    };

    // Create clipboard object
    let clipboard_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Create writeText method template
    let write_text_template = v8::FunctionTemplate::new(scope, write_text_callback);
    let write_text_fn = write_text_template.get_function(scope).unwrap();
    let write_text_key = v8::String::new(scope, "writeText").unwrap();
    clipboard_obj.set(scope, write_text_key.into(), write_text_fn.into());

    // Create readText method template
    let read_text_template = v8::FunctionTemplate::new(scope, read_text_callback);
    let read_text_fn = read_text_template.get_function(scope).unwrap();
    let read_text_key = v8::String::new(scope, "readText").unwrap();
    clipboard_obj.set(scope, read_text_key.into(), read_text_fn.into());

    // Create read method template (modern API)
    let read_template = v8::FunctionTemplate::new(scope, read_callback);
    let read_fn = read_template.get_function(scope).unwrap();
    let read_key = v8::String::new(scope, "read").unwrap();
    clipboard_obj.set(scope, read_key.into(), read_fn.into());

    // Create write method template (modern API)
    let write_template = v8::FunctionTemplate::new(scope, write_callback);
    let write_fn = write_template.get_function(scope).unwrap();
    let write_key = v8::String::new(scope, "write").unwrap();
    clipboard_obj.set(scope, write_key.into(), write_fn.into());

    // Set clipboard object on navigator
    let clipboard_key = v8::String::new(scope, "clipboard").unwrap();
    navigator_obj.set(scope, clipboard_key.into(), clipboard_obj.into());

    Ok(())
}

/// writeText callback - writes text to clipboard
fn write_text_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let text_arg = args.get(0);

    let text = if text_arg.is_string() {
        text_arg
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Store in global clipboard state
    let mut clip_content = CLIPBOARD_CONTENT.lock().unwrap();
    *clip_content = Some(text);

    // Return undefined (Promise resolves to undefined)
    retval.set(v8::undefined(scope).into());
}

/// readText callback - reads text from clipboard
fn read_text_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get clipboard content
    let clip_content = CLIPBOARD_CONTENT.lock().unwrap();
    let text = clip_content.as_ref().cloned().unwrap_or_default();

    // Return the text
    let text_val = v8::String::new(scope, &text).unwrap();
    retval.set(text_val.into());
}

/// read callback - modern read API (returns ClipboardItem array)
fn read_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // For now, return empty array (simplified implementation)
    let empty_array: v8::Local<v8::Array> = v8::Array::new(scope, 0);
    retval.set(empty_array.into());
}

/// write callback - modern write API (takes ClipboardItem array)
fn write_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // For now, just return undefined (simplified implementation)
    // In full implementation, would parse ClipboardItem objects
    retval.set(v8::undefined(scope).into());
}

/// Helper function to get clipboard content (for testing)
pub fn get_clipboard_content() -> Option<String> {
    CLIPBOARD_CONTENT.lock().unwrap().clone()
}

/// Helper function to set clipboard content (for testing)
pub fn set_clipboard_content(content: &str) {
    let mut clip = CLIPBOARD_CONTENT.lock().unwrap();
    *clip = Some(content.to_string());
}

/// Clear clipboard content (for testing)
pub fn clear_clipboard() {
    let mut clip = CLIPBOARD_CONTENT.lock().unwrap();
    *clip = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_set_get() {
        set_clipboard_content("test content");
        assert_eq!(get_clipboard_content(), Some("test content".to_string()));

        clear_clipboard();
        assert_eq!(get_clipboard_content(), None);
    }

    #[test]
    fn test_clipboard_empty() {
        clear_clipboard();
        set_clipboard_content("");
        assert_eq!(get_clipboard_content(), Some("".to_string()));
    }
}

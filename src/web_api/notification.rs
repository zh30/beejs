// Web Notification API implementation for Web standard
// v0.3.328: Notification API for displaying system notifications
// Provides Notification constructor, permission management, and instance methods

use anyhow::Result;
use rusty_v8 as v8;

/// Notification permission state
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationPermission {
    Granted, // User has granted permission
    Denied,  // User has denied permission
    Default, // User has not been asked yet
}

impl NotificationPermission {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationPermission::Granted => "granted",
            NotificationPermission::Denied => "denied",
            NotificationPermission::Default => "default",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "granted" => NotificationPermission::Granted,
            "denied" => NotificationPermission::Denied,
            _ => NotificationPermission::Default,
        }
    }
}

/// Setup Notification API in V8 context
pub fn setup_notification_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
    _global: v8::Local<v8::Object>,
) -> Result<()> {
    let global = context.global(scope);

    // Notification constructor
    let notification_fn = v8::FunctionTemplate::new(scope, notification_constructor_callback);
    let notification_constructor = notification_fn.get_function(scope).unwrap();
    let notification_key = v8::String::new(scope, "Notification").unwrap();
    global.set(
        scope,
        notification_key.into(),
        notification_constructor.into(),
    );

    // Notification.permission (static property)
    let permission_key = v8::String::new(scope, "permission").unwrap();
    let permission_val = v8::String::new(scope, "default").unwrap();
    global.set(scope, permission_key.into(), permission_val.into());

    // Notification.requestPermission (static method)
    let request_perm_fn =
        v8::FunctionTemplate::new(scope, notification_request_permission_callback);
    let request_perm_key = v8::String::new(scope, "requestPermission").unwrap();
    let request_perm_func = request_perm_fn.get_function(scope).unwrap();
    global.set(scope, request_perm_key.into(), request_perm_func.into());

    // Notification.permission property on constructor
    let constructor_permission_key = v8::String::new(scope, "permission").unwrap();
    let constructor_permission = v8::String::new(scope, "default").unwrap();
    global.set(
        scope,
        constructor_permission_key.into(),
        constructor_permission.into(),
    );

    Ok(())
}

/// Notification constructor callback
fn notification_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get title from first argument
    let title = if args.length() > 0 {
        args.get(0)
            .to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "").unwrap())
            .to_rust_string_lossy(scope)
    } else {
        "Notification".to_string()
    };

    // Get options from second argument
    let mut body = String::new();
    let mut icon = None;
    let mut tag = None;
    let mut data = None;

    if args.length() > 1 {
        let options = args.get(1);
        if let Some(options_obj) = options.to_object(scope) {
            // body
            let body_key = v8::String::new(scope, "body").unwrap();
            if let Some(body_val) = options_obj
                .get(scope, body_key.into())
                .and_then(|v| v.to_string(scope))
            {
                body = body_val.to_rust_string_lossy(scope);
            }

            // icon - store the string directly
            let icon_key = v8::String::new(scope, "icon").unwrap();
            if let Some(icon_val) = options_obj
                .get(scope, icon_key.into())
                .and_then(|v| v.to_string(scope))
            {
                icon = Some(icon_val.to_rust_string_lossy(scope));
            }

            // tag - store the string directly
            let tag_key = v8::String::new(scope, "tag").unwrap();
            if let Some(tag_val) = options_obj
                .get(scope, tag_key.into())
                .and_then(|v| v.to_string(scope))
            {
                tag = Some(tag_val.to_rust_string_lossy(scope));
            }

            // data - store the string directly
            let data_key = v8::String::new(scope, "data").unwrap();
            if let Some(data_val) = options_obj
                .get(scope, data_key.into())
                .and_then(|v| v.to_string(scope))
            {
                data = Some(data_val.to_rust_string_lossy(scope));
            }
        }
    }

    // Create notification object
    let notification_obj = v8::Object::new(scope);

    // Store undefined in a local to avoid multiple mutable borrows
    let undefined_val: v8::Local<v8::Value> = v8::undefined(scope).into();

    // title property (read-only)
    let title_key = v8::String::new(scope, "title").unwrap();
    let title_val = v8::String::new(scope, &title).unwrap();
    notification_obj.set(scope, title_key.into(), title_val.into());

    // body property
    let body_key = v8::String::new(scope, "body").unwrap();
    let body_val = v8::String::new(scope, &body).unwrap();
    notification_obj.set(scope, body_key.into(), body_val.into());

    // icon property
    let icon_key = v8::String::new(scope, "icon").unwrap();
    if let Some(icon_str) = &icon {
        let icon_val = v8::String::new(scope, icon_str).unwrap();
        notification_obj.set(scope, icon_key.into(), icon_val.into());
    } else {
        notification_obj.set(scope, icon_key.into(), undefined_val);
    }

    // tag property
    let tag_key = v8::String::new(scope, "tag").unwrap();
    if let Some(tag_str) = &tag {
        let tag_val = v8::String::new(scope, tag_str).unwrap();
        notification_obj.set(scope, tag_key.into(), tag_val.into());
    } else {
        notification_obj.set(scope, tag_key.into(), undefined_val);
    }

    // data property
    let data_key = v8::String::new(scope, "data").unwrap();
    if let Some(data_str) = &data {
        let data_val = v8::String::new(scope, data_str).unwrap();
        notification_obj.set(scope, data_key.into(), data_val.into());
    } else {
        notification_obj.set(scope, data_key.into(), undefined_val);
    }

    // onclick event handler property
    let onclick_key = v8::String::new(scope, "onclick").unwrap();
    notification_obj.set(scope, onclick_key.into(), undefined_val);

    // onshow event handler property
    let onshow_key = v8::String::new(scope, "onshow").unwrap();
    notification_obj.set(scope, onshow_key.into(), undefined_val);

    // onclose event handler property
    let onclose_key = v8::String::new(scope, "onclose").unwrap();
    notification_obj.set(scope, onclose_key.into(), undefined_val);

    // onerror event handler property
    let onerror_key = v8::String::new(scope, "onerror").unwrap();
    notification_obj.set(scope, onerror_key.into(), undefined_val);

    // close() method
    let close_fn = v8::FunctionTemplate::new(scope, notification_close_callback);
    let close_key = v8::String::new(scope, "close").unwrap();
    let close_func = close_fn.get_function(scope).unwrap();
    notification_obj.set(scope, close_key.into(), close_func.into());

    // Return the basic object
    rv.set(notification_obj.into());

    // Log for debugging
    eprintln!(
        "[Notification] Created: title='{}', body='{}', icon={:?}, tag={:?}",
        title, body, icon, tag
    );
}

/// Notification.close() callback
fn notification_close_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Close the notification
    eprintln!("[Notification.close] Notification closed");
    rv.set(v8::undefined(scope).into());
}

/// Notification.requestPermission() callback
fn notification_request_permission_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a promise that resolves to the permission
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(r) => r,
        None => {
            let error = v8::String::new(scope, "Failed to create promise resolver").unwrap();
            scope.throw_exception(error.into());
            return;
        }
    };
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    // Update permission to "granted" (for demo purposes)
    let permission = "granted";

    // Get context and global from scope
    let context = scope.get_current_context();
    let global = context.global(scope);

    // Update global permission property
    let permission_key = v8::String::new(scope, "permission").unwrap();
    let permission_val = v8::String::new(scope, permission).unwrap();
    global.set(scope, permission_key.into(), permission_val.into());

    // Resolve the promise
    let permission_str = v8::String::new(scope, permission).unwrap();
    resolver.resolve(scope, permission_str.into());

    eprintln!("[Notification.requestPermission] Permission granted");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_permission_values() {
        assert_eq!(NotificationPermission::Granted.as_str(), "granted");
        assert_eq!(NotificationPermission::Denied.as_str(), "denied");
        assert_eq!(NotificationPermission::Default.as_str(), "default");
    }

    #[test]
    fn test_notification_permission_from_string() {
        assert_eq!(
            NotificationPermission::from_string("granted"),
            NotificationPermission::Granted
        );
        assert_eq!(
            NotificationPermission::from_string("denied"),
            NotificationPermission::Denied
        );
        assert_eq!(
            NotificationPermission::from_string("unknown"),
            NotificationPermission::Default
        );
        assert_eq!(
            NotificationPermission::from_string("GRANTED"),
            NotificationPermission::Granted
        );
    }

    #[test]
    fn test_notification_permission_default_case() {
        // Test that case-insensitive matching works
        assert_eq!(
            NotificationPermission::from_string("Default"),
            NotificationPermission::Default
        );
        assert_eq!(
            NotificationPermission::from_string("DEFAULT"),
            NotificationPermission::Default
        );
    }
}

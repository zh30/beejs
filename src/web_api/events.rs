// EventTarget and Event API implementation for Web standard
// Provides addEventListener, removeEventListener, dispatchEvent, Event, ExtendableEvent

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rusty_v8 as v8;

/// Event type enum
#[derive(Debug, Clone)]
pub enum EventType {
    Custom(String),
    BuiltIn(String),
}

/// Event structure
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub target: Option<String>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub current_target: Option<String>,
    pub default_prevented: bool,
    pub event_phase: u8,
    pub is_trusted: bool,
}
impl Event {
    pub fn new(event_type: String) -> Self {
        Self {
            event_type,
            target: None,
            bubbles: false,
            cancelable: true,
            composed: false,
            current_target: None,
            default_prevented: false,
            event_phase: 0,
            is_trusted: true,
        }
    }
}

/// ExtendableEvent - Base class for events that support waitUntil()
/// Used by ServiceWorker lifecycle events (install, activate)
#[derive(Debug, Clone)]
pub struct ExtendableEvent {
    pub event_type: String,
    pub target: Option<String>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub current_target: Option<String>,
    pub default_prevented: bool,
    pub event_phase: u8,
    pub is_trusted: bool,
    pub is_extended: bool,  // Whether waitUntil() has been called
}
impl ExtendableEvent {
    pub fn new(event_type: String) -> Self {
        Self {
            event_type,
            target: None,
            bubbles: false,
            cancelable: true,
            composed: false,
            current_target: None,
            default_prevented: false,
            event_phase: 0,
            is_trusted: true,
            is_extended: false,
        }
    }
}
/// EventTarget structure
#[derive(Clone)]
pub struct EventTarget {
    listeners: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(&Event) + Send + Sync>>>>>,
}
impl EventTarget {
    /// Create new EventTarget
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    /// Add event listener
    pub fn add_event_listener(&self, event_type: String, listener: Box<dyn Fn(&Event) + Send + Sync>) {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.entry(event_type).or_insert_with(Vec::new).push(listener);
        }
    }
    /// Remove event listener
    pub fn remove_event_listener(&self, event_type: &str) {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.remove(event_type);
        }
    }
    /// Dispatch event
    pub fn dispatch_event(&self, event: &Event) -> bool {
        let result = true;
        if let Ok(listeners) = self.listeners.lock() {
            if let Some(event_listeners) = listeners.get(&event.event_type) {
                for listener in event_listeners {
                    listener(event);
                }
            }
        }
        result
    }
}
impl Default for EventTarget {
    fn default() -> Self {
        Self::new()
    }
}
/// Setup EventTarget and Event API in V8 context
pub fn setup_events_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> anyhow::Result<()> {
    // Create EventTarget constructor
    let event_target_template: _ = v8::FunctionTemplate::new(scope, event_target_constructor_callback);
    let event_target_constructor: _ = event_target_template.get_function(scope).unwrap();

    // Set EventTarget to global
    let global: _ = context.global(scope);
    let event_target_key: _ = v8::String::new(scope, "EventTarget").unwrap();
    global.set(scope, event_target_key.into(), event_target_constructor.into());

    // Set Event constructor to global
    let event_fn = v8::FunctionTemplate::new(scope, event_constructor_callback);
    let event_constructor_func = event_fn.get_function(scope).unwrap();
    let event_key: _ = v8::String::new(scope, "Event").unwrap();
    global.set(scope, event_key.into(), event_constructor_func.into());

    // Set ExtendableEvent to global
    let extendable_event_fn = v8::FunctionTemplate::new(scope, extendable_event_constructor_callback);
    let extendable_event_func = extendable_event_fn.get_function(scope).unwrap();
    let extendable_event_key: _ = v8::String::new(scope, "ExtendableEvent").unwrap();
    global.set(scope, extendable_event_key.into(), extendable_event_func.into());

    Ok(())
}
/// EventTarget constructor callback
fn event_target_constructor_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let event_target_obj: _ = v8::Object::new(scope);
    // Add prototype methods to instance
    let add_event_key: _ = v8::String::new(scope, "addEventListener").unwrap();
    let add_event_func: _ = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let event_type: _ = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
        println!("addEventListener: {}", event_type);
    });
    let add_event_func_instance: _ = add_event_func.get_function(scope).unwrap();
    event_target_obj.set(scope, add_event_key.into(), add_event_func_instance.into());
    let remove_event_key: _ = v8::String::new(scope, "removeEventListener").unwrap();
    let remove_event_func: _ = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let event_type: _ = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
        println!("removeEventListener: {}", event_type);
    });
    let remove_event_func_instance: _ = remove_event_func.get_function(scope).unwrap();
    event_target_obj.set(scope, remove_event_key.into(), remove_event_func_instance.into());
    let dispatch_event_key: _ = v8::String::new(scope, "dispatchEvent").unwrap();
    let dispatch_event_func: _ = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        println!("dispatchEvent");
    });
    let dispatch_event_func_instance: _ = dispatch_event_func.get_function(scope).unwrap();
    event_target_obj.set(scope, dispatch_event_key.into(), dispatch_event_func_instance.into());
    retval.set(event_target_obj.into());
}

/// Event constructor callback
fn event_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let event_obj = v8::Object::new(scope);

    // Get event type from arguments
    let event_type = if args.length() > 0 {
        args.get(0).to_string(scope).unwrap_or_else(|| v8::String::new(scope, "").unwrap())
            .to_rust_string_lossy(scope)
    } else {
        "".to_string()
    };

    // Store type as internal property (using symbol)
    let type_key = v8::String::new(scope, "_type").unwrap();
    let type_val = v8::String::new(scope, &event_type).unwrap();
    event_obj.set(scope, type_key.into(), type_val.into());

    // Set properties - extract values first to avoid scope borrow issues
    let type_prop_key = v8::String::new(scope, "type").unwrap();
    event_obj.set(scope, type_prop_key.into(), type_val.into());

    let bubbles_false = v8::Boolean::new(scope, false);
    let bubbles_key = v8::String::new(scope, "bubbles").unwrap();
    event_obj.set(scope, bubbles_key.into(), bubbles_false.into());

    let cancelable_true = v8::Boolean::new(scope, true);
    let cancelable_key = v8::String::new(scope, "cancelable").unwrap();
    event_obj.set(scope, cancelable_key.into(), cancelable_true.into());

    let default_prevented_false = v8::Boolean::new(scope, false);
    let default_prevented_key = v8::String::new(scope, "defaultPrevented").unwrap();
    event_obj.set(scope, default_prevented_key.into(), default_prevented_false.into());

    rv.set(event_obj.into());
}

/// ExtendableEvent constructor callback
fn extendable_event_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let event_obj = v8::Object::new(scope);

    // Get event type from arguments
    let event_type = if args.length() > 0 {
        args.get(0).to_string(scope).unwrap_or_else(|| v8::String::new(scope, "").unwrap())
            .to_rust_string_lossy(scope)
    } else {
        "".to_string()
    };

    // Store type as internal property
    let type_key = v8::String::new(scope, "_type").unwrap();
    let type_val = v8::String::new(scope, &event_type).unwrap();
    event_obj.set(scope, type_key.into(), type_val.into());

    // Set properties - extract values first to avoid scope borrow issues
    let type_prop_key = v8::String::new(scope, "type").unwrap();
    event_obj.set(scope, type_prop_key.into(), type_val.into());

    let bubbles_false = v8::Boolean::new(scope, false);
    let bubbles_key = v8::String::new(scope, "bubbles").unwrap();
    event_obj.set(scope, bubbles_key.into(), bubbles_false.into());

    let cancelable_true = v8::Boolean::new(scope, true);
    let cancelable_key = v8::String::new(scope, "cancelable").unwrap();
    event_obj.set(scope, cancelable_key.into(), cancelable_true.into());

    rv.set(event_obj.into());
}
#[cfg(test)]
mod tests {
    use super::{Event, EventTarget};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_event_creation() {
        let event: _ = Event::new("click".to_string());
        assert_eq!(event.event_type, "click");
        assert_eq!(event.bubbles, false);
        assert_eq!(event.cancelable, true);
    }
    #[test]
    fn test_event_target_creation() {
        let target: _ = EventTarget::new();
        assert!(target.listeners.lock().is_ok());
    }
    #[test]
    fn test_event_listener_management() {
        let target: _ = EventTarget::new();
        let event_called: _ = Arc::new(Mutex::new(false));
        let event_called_clone: _ = event_called.clone();
        let listener: _ = Box::new(move |event: &Event| {
            if event.event_type == "test" {
                *event_called_clone.lock().unwrap() = true;
            }
        });
        target.add_event_listener("test".to_string(), listener);
        let event: _ = Event::new("test".to_string());
        let result: _ = target.dispatch_event(&event);
        assert!(result);
        assert!(*event_called.lock().unwrap());
    }
}
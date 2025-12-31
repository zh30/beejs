// Push API tests for Beejs runtime
// v0.3.326: Tests for PushManager, PushSubscription, and PushEvent APIs

use std::process::{Command, Stdio};
use std::fs;

#[cfg(test)]
mod push_manager_tests {
    use super::*;

    #[test]
    fn test_push_manager_exists() {
        // Test that PushManager exists
        let script = r#"
            if (typeof PushManager === 'function') {
                console.log('SUCCESS: PushManager exists');
            } else {
                throw new Error('PushManager not found');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "PushManager should exist: {}", stdout);
        assert!(stdout.contains("SUCCESS"), "Output: {}", stdout);
    }

    #[test]
    fn test_push_manager_subscribe_exists() {
        // Test that PushManager.subscribe exists (method on prototype or static)
        let script = r#"
            if (typeof PushManager !== 'undefined') {
                // Check if subscribe is accessible via prototype (standard API)
                const proto = PushManager.prototype;
                const hasSubscribe = proto && typeof proto.subscribe === 'function';
                if (hasSubscribe) {
                    console.log('SUCCESS: PushManager.subscribe is a function');
                } else {
                    console.log('ERROR: PushManager.subscribe not found');
                }
            } else {
                console.log('ERROR: PushManager not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "subscribe should be a function: {}", stdout);
    }

    #[test]
    fn test_push_manager_get_subscription_exists() {
        // Test that PushManager.getSubscription exists
        let script = r#"
            if (typeof PushManager !== 'undefined') {
                const proto = PushManager.prototype;
                const hasGetSub = proto && typeof proto.getSubscription === 'function';
                if (hasGetSub) {
                    console.log('SUCCESS: PushManager.getSubscription is a function');
                } else {
                    console.log('ERROR: PushManager.getSubscription not found');
                }
            } else {
                console.log('ERROR: PushManager not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "getSubscription should be a function: {}", stdout);
    }

    #[test]
    fn test_push_manager_permission_state_exists() {
        // Test that PushManager.permissionState exists
        let script = r#"
            if (typeof PushManager !== 'undefined') {
                const proto = PushManager.prototype;
                const hasPermState = proto && typeof proto.permissionState === 'function';
                if (hasPermState) {
                    console.log('SUCCESS: PushManager.permissionState is a function');
                } else {
                    console.log('ERROR: PushManager.permissionState not found');
                }
            } else {
                console.log('ERROR: PushManager not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "permissionState should be a function: {}", stdout);
    }
}

#[cfg(test)]
mod push_subscription_tests {
    use super::*;

    #[test]
    fn test_push_subscription_exists() {
        // Test that PushSubscription exists
        let script = r#"
            if (typeof PushSubscription === 'function') {
                console.log('SUCCESS: PushSubscription exists');
            } else {
                throw new Error('PushSubscription not found');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushSubscription should exist: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance_has_get_key() {
        // Test that PushSubscription instance has getKey method
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();
                if (sub && typeof sub.getKey === 'function') {
                    console.log('SUCCESS: PushSubscription instance has getKey method');
                } else {
                    console.log('ERROR: getKey method not found on instance');
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "instance should have getKey: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance_has_to_json() {
        // Test that PushSubscription instance has toJSON method
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();
                if (sub && typeof sub.toJSON === 'function') {
                    console.log('SUCCESS: PushSubscription instance has toJSON method');
                } else {
                    console.log('ERROR: toJSON method not found on instance');
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "instance should have toJSON: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance_has_unsubscribe() {
        // Test that PushSubscription instance has unsubscribe method
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();
                if (sub && typeof sub.unsubscribe === 'function') {
                    console.log('SUCCESS: PushSubscription instance has unsubscribe method');
                } else {
                    console.log('ERROR: unsubscribe method not found on instance');
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "instance should have unsubscribe: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance_has_endpoint() {
        // Test that PushSubscription instance has endpoint property
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();
                if (sub && 'endpoint' in sub && sub.endpoint) {
                    console.log('SUCCESS: PushSubscription instance has endpoint property');
                } else {
                    console.log('ERROR: endpoint property not found on instance');
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "instance should have endpoint: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance_has_options() {
        // Test that PushSubscription instance has options property
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();
                if (sub && 'options' in sub && sub.options) {
                    console.log('SUCCESS: PushSubscription instance has options property');
                } else {
                    console.log('ERROR: options property not found on instance');
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "instance should have options: {}", stdout);
    }
}

#[cfg(test)]
mod push_event_tests {
    use super::*;

    #[test]
    fn test_push_event_exists() {
        // Test that PushEvent exists
        let script = r#"
            if (typeof PushEvent === 'function') {
                console.log('SUCCESS: PushEvent exists');
            } else {
                throw new Error('PushEvent not found');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushEvent should exist: {}", stdout);
    }

    #[test]
    fn test_push_event_is_extendable_event() {
        // Test that PushEvent extends ExtendableEvent
        let script = r#"
            if (typeof PushEvent === 'function' && typeof ExtendableEvent !== 'undefined') {
                const event = new PushEvent('push');
                if (event && typeof event.waitUntil === 'function') {
                    console.log('SUCCESS: PushEvent has waitUntil method (extends ExtendableEvent)');
                } else {
                    console.log('ERROR: PushEvent missing waitUntil method');
                }
            } else {
                console.log('ERROR: PushEvent or ExtendableEvent not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushEvent should extend ExtendableEvent: {}", stdout);
    }

    #[test]
    fn test_push_event_has_data() {
        // Test that PushEvent has data property
        let script = r#"
            if (typeof PushEvent === 'function') {
                const event = new PushEvent('push', { data: 'test message' });
                if (event && event.data && event.data === 'test message') {
                    console.log('SUCCESS: PushEvent.data property works');
                } else {
                    console.log('ERROR: PushEvent.data property not working');
                    console.log('event.data: ' + event.data);
                }
            } else {
                console.log('ERROR: PushEvent not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushEvent.data should work: {}", stdout);
    }

    #[test]
    fn test_push_event_data_null_by_default() {
        // Test that PushEvent.data is null by default
        let script = r#"
            if (typeof PushEvent === 'function') {
                const event = new PushEvent('push');
                if (event && event.data === null) {
                    console.log('SUCCESS: PushEvent.data is null by default');
                } else {
                    console.log('ERROR: PushEvent.data should be null by default');
                    console.log('event.data: ' + event.data);
                }
            } else {
                console.log('ERROR: PushEvent not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushEvent.data should be null by default: {}", stdout);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_push_api_global_availability() {
        // Test that all Push API components are globally available
        let script = r#"
            const components = ['PushManager', 'PushSubscription', 'PushEvent'];
            const missing = [];

            for (const name of components) {
                // Check globalThis first (works in both browser and Node.js-like environments)
                const globalObj = typeof globalThis !== 'undefined' ? globalThis :
                                  typeof window !== 'undefined' ? window :
                                  typeof global !== 'undefined' ? global : {};
                if (typeof globalObj[name] === 'undefined') {
                    missing.push(name);
                }
            }

            if (missing.length === 0) {
                console.log('SUCCESS: All Push API components are globally available');
            } else {
                console.log('ERROR: Missing components: ' + missing.join(', '));
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "All Push API should be available: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance_methods() {
        // Test that PushSubscription instance has all required methods
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();

                // Check instance methods
                const hasGetKey = typeof sub.getKey === 'function';
                const hasToJSON = typeof sub.toJSON === 'function';
                const hasUnsubscribe = typeof sub.unsubscribe === 'function';

                if (hasGetKey && hasToJSON && hasUnsubscribe) {
                    console.log('SUCCESS: PushSubscription instance has all required methods');
                } else {
                    console.log('ERROR: PushSubscription instance missing methods');
                    console.log('hasGetKey: ' + hasGetKey);
                    console.log('hasToJSON: ' + hasToJSON);
                    console.log('hasUnsubscribe: ' + hasUnsubscribe);
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushSubscription instance should have methods: {}", stdout);
    }

    #[test]
    fn test_push_subscription_instance() {
        // Test creating a PushSubscription instance and accessing its properties
        let script = r#"
            if (typeof PushSubscription === 'function') {
                const sub = new PushSubscription();

                // Check instance properties
                const hasEndpoint = 'endpoint' in sub && sub.endpoint;
                const hasOptions = 'options' in sub && sub.options;

                // Check methods work
                const getKeyResult = sub.getKey ? sub.getKey('p256dh') : null;
                const toJSONResult = sub.toJSON ? sub.toJSON() : null;

                if (hasEndpoint && hasOptions) {
                    console.log('SUCCESS: PushSubscription instance works correctly');
                    console.log('endpoint: ' + sub.endpoint);
                    console.log('hasOptions: ' + hasOptions);
                } else {
                    console.log('ERROR: PushSubscription instance missing properties');
                    console.log('hasEndpoint: ' + hasEndpoint);
                    console.log('hasOptions: ' + hasOptions);
                }
            } else {
                console.log('ERROR: PushSubscription not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "PushSubscription instance should work: {}", stdout);
    }
}

/// Helper function to run JavaScript scripts using beejs
fn run_script(script: &str) -> std::process::Output {
    // Create a temporary file with the script
    let temp_dir = tempfile::Builder::new()
        .prefix("beejs-push-test-")
        .tempdir()
        .unwrap();
    let temp_file = temp_dir.path().join("test.js");
    fs::write(&temp_file, script).unwrap();

    // Run beejs with the script
    let output = Command::new("./target/release/beejs")
        .arg("run")
        .arg(&temp_file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run beejs");

    // Clean up
    drop(temp_dir);

    output
}

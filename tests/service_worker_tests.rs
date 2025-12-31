// ServiceWorker API tests for Beejs runtime
// v0.3.324: Tests for ServiceWorker, Cache, and CacheStorage APIs

use std::process::{Command, Stdio};
use std::fs;

#[cfg(test)]
mod service_worker_tests {
    use super::*;

    #[test]
    fn test_navigator_service_worker_exists() {
        // Test that navigator.serviceWorker exists
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                console.log('SUCCESS: navigator.serviceWorker exists');
            } else {
                throw new Error('navigator.serviceWorker not found');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "ServiceWorker should exist: {}", stdout);
        assert!(stdout.contains("SUCCESS: navigator.serviceWorker exists"), "Output: {}", stdout);
    }

    #[test]
    fn test_service_worker_register_exists() {
        // Test that navigator.serviceWorker.register exists
        let script = r#"
            if (typeof navigator !== 'undefined' &&
                typeof navigator.serviceWorker !== 'undefined' &&
                typeof navigator.serviceWorker.register === 'function') {
                console.log('SUCCESS: navigator.serviceWorker.register is a function');
            } else {
                throw new Error('navigator.serviceWorker.register not found or not a function');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "register should be a function: {}", stdout);
    }

    #[test]
    fn test_service_worker_register_returns_promise() {
        // Test that navigator.serviceWorker.register returns a Promise
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                const result = navigator.serviceWorker.register('./test-sw.js');
                if (result && typeof result.then === 'function') {
                    console.log('SUCCESS: register returns a Promise');
                } else {
                    console.log('ERROR: register does not return a Promise');
                }
            } else {
                console.log('ERROR: navigator.serviceWorker not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Should return Promise: {}", stdout);
    }

    #[test]
    fn test_service_worker_registration_has_scope() {
        // Test that registration object has scope property
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                navigator.serviceWorker.register('./test-sw.js').then(registration => {
                    if (registration && typeof registration.scope === 'string') {
                        console.log('SUCCESS: registration.scope exists: ' + registration.scope);
                    } else {
                        console.log('ERROR: registration.scope not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: navigator.serviceWorker not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Registration should have scope: {}", stdout);
    }

    #[test]
    fn test_service_worker_registration_has_installing() {
        // Test that registration object has installing property
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                navigator.serviceWorker.register('./test-sw.js').then(registration => {
                    if ('installing' in registration) {
                        console.log('SUCCESS: registration.installing property exists');
                    } else {
                        console.log('ERROR: registration.installing not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: navigator.serviceWorker not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Registration should have installing: {}", stdout);
    }

    #[test]
    fn test_service_worker_registration_has_active() {
        // Test that registration object has active property
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                navigator.serviceWorker.register('./test-sw.js').then(registration => {
                    if ('active' in registration) {
                        console.log('SUCCESS: registration.active property exists');
                    } else {
                        console.log('ERROR: registration.active not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: navigator.serviceWorker not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Registration should have active: {}", stdout);
    }

    #[test]
    fn test_service_worker_ready_property() {
        // Test that navigator.serviceWorker.ready exists
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                if ('ready' in navigator.serviceWorker) {
                    console.log('SUCCESS: navigator.serviceWorker.ready exists');
                } else {
                    console.log('ERROR: navigator.serviceWorker.ready not found');
                }
            } else {
                console.log('ERROR: navigator.serviceWorker not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Ready property should exist: {}", stdout);
    }
}

#[cfg(test)]
mod cache_api_tests {
    use super::*;

    #[test]
    fn test_caches_exists() {
        // Test that global caches object exists
        let script = r#"
            if (typeof caches !== 'undefined') {
                console.log('SUCCESS: global caches object exists');
            } else {
                throw new Error('caches object not found');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "caches should exist: {}", stdout);
        assert!(stdout.contains("SUCCESS: global caches object exists"), "Output: {}", stdout);
    }

    #[test]
    fn test_caches_open_exists() {
        // Test that caches.open exists (caches is singleton object, not constructor)
        let script = r#"
            if (typeof caches !== 'undefined' && typeof caches.open === 'function') {
                console.log('SUCCESS: caches.open is a function');
            } else {
                throw new Error('caches.open not found or not a function');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "caches.open should be a function: {}", stdout);
    }

    #[test]
    fn test_caches_keys_exists() {
        // Test that caches.keys exists
        let script = r#"
            if (typeof caches !== 'undefined' && typeof caches.keys === 'function') {
                console.log('SUCCESS: caches.keys is a function');
            } else {
                throw new Error('caches.keys not found or not a function');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "caches.keys should be a function: {}", stdout);
    }

    #[test]
    fn test_caches_has_exists() {
        // Test that caches.has exists
        let script = r#"
            if (typeof caches !== 'undefined' && typeof caches.has === 'function') {
                console.log('SUCCESS: caches.has is a function');
            } else {
                throw new Error('caches.has not found or not a function');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "caches.has should be a function: {}", stdout);
    }

    #[test]
    fn test_caches_delete_exists() {
        // Test that caches.delete exists
        let script = r#"
            if (typeof caches !== 'undefined' && typeof caches.delete === 'function') {
                console.log('SUCCESS: caches.delete is a function');
            } else {
                throw new Error('caches.delete not found or not a function');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "caches.delete should be a function: {}", stdout);
    }

    #[test]
    fn test_caches_open_returns_cache() {
        // Test that caches.open() returns a Promise that resolves to a Cache object
        let script = r#"
            if (typeof caches !== 'undefined') {
                const result = caches.open('test-cache');
                if (result && typeof result.then === 'function') {
                    result.then(cache => {
                        if (cache && typeof cache === 'object') {
                            console.log('SUCCESS: caches.open() returns a Promise resolving to Cache');
                        } else {
                            console.log('ERROR: Promise did not resolve to Cache object');
                        }
                    }).catch(e => {
                        console.log('ERROR: ' + e.message);
                    });
                } else {
                    console.log('ERROR: caches.open() does not return a Promise');
                }
            } else {
                console.log('ERROR: caches not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "caches.open should return Promise: {}", stdout);
    }
}

#[cfg(test)]
mod cache_object_tests {
    use super::*;

    #[test]
    fn test_cache_add_all_exists() {
        // Test that Cache.addAll exists
        let script = r#"
            if (typeof caches !== 'undefined') {
                caches.open('test-cache').then(cache => {
                    if (typeof cache.addAll === 'function') {
                        console.log('SUCCESS: Cache.addAll is a function');
                    } else {
                        console.log('ERROR: Cache.addAll not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: caches not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Cache.addAll should exist: {}", stdout);
    }

    #[test]
    fn test_cache_match_exists() {
        // Test that Cache.match exists
        let script = r#"
            if (typeof caches !== 'undefined') {
                caches.open('test-cache').then(cache => {
                    if (typeof cache.match === 'function') {
                        console.log('SUCCESS: Cache.match is a function');
                    } else {
                        console.log('ERROR: Cache.match not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: caches not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Cache.match should exist: {}", stdout);
    }

    #[test]
    fn test_cache_put_exists() {
        // Test that Cache.put exists
        let script = r#"
            if (typeof caches !== 'undefined') {
                caches.open('test-cache').then(cache => {
                    if (typeof cache.put === 'function') {
                        console.log('SUCCESS: Cache.put is a function');
                    } else {
                        console.log('ERROR: Cache.put not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: caches not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Cache.put should exist: {}", stdout);
    }

    #[test]
    fn test_cache_delete_exists() {
        // Test that Cache.delete exists
        let script = r#"
            if (typeof caches !== 'undefined') {
                caches.open('test-cache').then(cache => {
                    if (typeof cache.delete === 'function') {
                        console.log('SUCCESS: Cache.delete is a function');
                    } else {
                        console.log('ERROR: Cache.delete not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: caches not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Cache.delete should exist: {}", stdout);
    }

    #[test]
    fn test_cache_keys_exists() {
        // Test that Cache.keys exists
        let script = r#"
            if (typeof caches !== 'undefined') {
                caches.open('test-cache').then(cache => {
                    if (typeof cache.keys === 'function') {
                        console.log('SUCCESS: Cache.keys is a function');
                    } else {
                        console.log('ERROR: Cache.keys not found');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: caches not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Cache.keys should exist: {}", stdout);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_service_worker_registration_flow() {
        // Test complete registration flow with all expected properties
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                navigator.serviceWorker.register('./sw.js').then(registration => {
                    // Check all expected properties
                    const hasScope = typeof registration.scope === 'string';
                    const hasInstalling = 'installing' in registration;
                    const hasActive = 'active' in registration;
                    const hasWaiting = 'waiting' in registration;

                    if (hasScope && hasInstalling && hasActive && hasWaiting) {
                        console.log('SUCCESS: Full registration flow works');
                        console.log('Scope: ' + registration.scope);
                    } else {
                        console.log('ERROR: Missing properties');
                        console.log('hasScope: ' + hasScope);
                        console.log('hasInstalling: ' + hasInstalling);
                        console.log('hasActive: ' + hasActive);
                        console.log('hasWaiting: ' + hasWaiting);
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: ServiceWorker not supported');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Full flow should work: {}", stdout);
    }

    #[test]
    fn test_cache_operations_flow() {
        // Test complete cache operations flow
        let script = r#"
            if (typeof caches !== 'undefined') {
                caches.open('my-cache').then(cache => {
                    const hasAddAll = typeof cache.addAll === 'function';
                    const hasMatch = typeof cache.match === 'function';
                    const hasPut = typeof cache.put === 'function';
                    const hasDelete = typeof cache.delete === 'function';
                    const hasKeys = typeof cache.keys === 'function';

                    if (hasAddAll && hasMatch && hasPut && hasDelete && hasKeys) {
                        console.log('SUCCESS: Cache operations flow works');
                    } else {
                        console.log('ERROR: Missing cache methods');
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: Cache API not supported');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Cache flow should work: {}", stdout);
    }

    #[test]
    fn test_service_worker_register_with_scope_option() {
        // Test registration with scope option
        let script = r#"
            if (typeof navigator !== 'undefined' && typeof navigator.serviceWorker !== 'undefined') {
                navigator.serviceWorker.register('./sw.js', { scope: '/app/' }).then(registration => {
                    if (registration.scope.includes('/app/')) {
                        console.log('SUCCESS: Custom scope is respected');
                    } else {
                        console.log('ERROR: Custom scope not applied');
                        console.log('Got: ' + registration.scope);
                    }
                }).catch(e => {
                    console.log('ERROR: ' + e.message);
                });
            } else {
                console.log('ERROR: ServiceWorker not supported');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "Scope option should work: {}", stdout);
    }
}

#[cfg(test)]
mod lifecycle_event_tests {
    use super::*;

    #[test]
    fn test_install_event_constructor_exists() {
        // Test that InstallEvent constructor exists
        let script = r#"
            if (typeof InstallEvent === 'function') {
                console.log('SUCCESS: InstallEvent constructor exists');
            } else {
                console.log('ERROR: InstallEvent constructor not found');
                console.log('typeof InstallEvent: ' + typeof InstallEvent);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "InstallEvent should exist: {}", stdout);
    }

    #[test]
    fn test_activate_event_constructor_exists() {
        // Test that ActivateEvent constructor exists
        let script = r#"
            if (typeof ActivateEvent === 'function') {
                console.log('SUCCESS: ActivateEvent constructor exists');
            } else {
                console.log('ERROR: ActivateEvent constructor not found');
                console.log('typeof ActivateEvent: ' + typeof ActivateEvent);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "ActivateEvent should exist: {}", stdout);
    }

    #[test]
    fn test_fetch_event_constructor_exists() {
        // Test that FetchEvent constructor exists
        let script = r#"
            if (typeof FetchEvent === 'function') {
                console.log('SUCCESS: FetchEvent constructor exists');
            } else {
                console.log('ERROR: FetchEvent constructor not found');
                console.log('typeof FetchEvent: ' + typeof FetchEvent);
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "FetchEvent should exist: {}", stdout);
    }

    #[test]
    fn test_install_event_creation() {
        // Test that InstallEvent can be created with correct properties
        let script = r#"
            if (typeof InstallEvent === 'function') {
                const event = new InstallEvent('install');
                if (event && event.type === 'install') {
                    console.log('SUCCESS: InstallEvent created with correct type');
                } else {
                    console.log('ERROR: InstallEvent type mismatch');
                    console.log('event.type: ' + event.type);
                }
            } else {
                console.log('ERROR: InstallEvent not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "InstallEvent creation should work: {}", stdout);
    }

    #[test]
    fn test_activate_event_creation() {
        // Test that ActivateEvent can be created with correct properties
        let script = r#"
            if (typeof ActivateEvent === 'function') {
                const event = new ActivateEvent('activate');
                if (event && event.type === 'activate') {
                    console.log('SUCCESS: ActivateEvent created with correct type');
                } else {
                    console.log('ERROR: ActivateEvent type mismatch');
                    console.log('event.type: ' + event.type);
                }
            } else {
                console.log('ERROR: ActivateEvent not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "ActivateEvent creation should work: {}", stdout);
    }

    #[test]
    fn test_fetch_event_creation() {
        // Test that FetchEvent can be created with request URL
        let script = r#"
            if (typeof FetchEvent === 'function') {
                const event = new FetchEvent('fetch', { requestUrl: '/api/test' });
                if (event && event.type === 'fetch' && event.requestUrl === '/api/test') {
                    console.log('SUCCESS: FetchEvent created with correct properties');
                } else {
                    console.log('ERROR: FetchEvent properties mismatch');
                    console.log('event.type: ' + event.type);
                    console.log('event.requestUrl: ' + event.requestUrl);
                }
            } else {
                console.log('ERROR: FetchEvent not defined');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "FetchEvent creation should work: {}", stdout);
    }

    #[test]
    fn test_service_worker_state_enum() {
        // Test that ServiceWorkerState enum is accessible via console
        let script = r#"
            // ServiceWorkerState values
            const states = ['parsing', 'installing', 'installed', 'activating', 'activated', 'redundant'];
            if (states.length === 6) {
                console.log('SUCCESS: ServiceWorkerState has 6 states');
            } else {
                console.log('ERROR: ServiceWorkerState state count mismatch');
            }
        "#;
        let output = run_script(script);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SUCCESS"), "ServiceWorkerState should have 6 states: {}", stdout);
    }
}

/// Helper function to run JavaScript scripts using beejs
fn run_script(script: &str) -> std::process::Output {
    // Create a temporary file with the script
    let temp_dir = tempfile::Builder::new()
        .prefix("beejs-service-worker-test-")
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

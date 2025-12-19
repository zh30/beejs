//! os polyfill

use rusty_v8 as v8;

pub fn register(scope: &mut v8::HandleScope, global: &v8::Local<v8::Object>) {
    let os_key = v8::String::new(scope, "os").unwrap();
    let os_obj = v8::Object::new(scope);

    // Platform
    let platform_fn = v8::FunctionTemplate::new(scope, platform).get_function(scope).unwrap();
    let platform_key = v8::String::new(scope, "platform").unwrap().into();
    os_obj.set(scope, platform_key, platform_fn.into());

    // Type
    let ostype_fn = v8::FunctionTemplate::new(scope, ostype).get_function(scope).unwrap();
    let ostype_key = v8::String::new(scope, "type").unwrap().into();
    os_obj.set(scope, ostype_key, ostype_fn.into());

    // Arch
    let arch_fn = v8::FunctionTemplate::new(scope, arch).get_function(scope).unwrap();
    let arch_key = v8::String::new(scope, "arch").unwrap().into();
    os_obj.set(scope, arch_key, arch_fn.into());

    global.set(scope, os_key.into(), os_obj.into());
}

fn platform(scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    #[cfg(target_os = "linux")]
    let platform = "linux";
    #[cfg(target_os = "macos")]
    let platform = "darwin";
    #[cfg(target_os = "windows")]
    let platform = "win32";
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let platform = "unknown";

    retval.set(v8::String::new(scope, platform).unwrap().into());
}

fn ostype(scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    #[cfg(target_os = "linux")]
    let ostype = "Linux";
    #[cfg(target_os = "macos")]
    let ostype = "Darwin";
    #[cfg(target_os = "windows")]
    let ostype = "Windows_NT";
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let ostype = "Unknown";

    retval.set(v8::String::new(scope, ostype).unwrap().into());
}

fn arch(scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    #[cfg(target_arch = "x86_64")]
    let arch = "x64";
    #[cfg(target_arch = "aarch64")]
    let arch = "arm64";
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let arch = "unknown";

    retval.set(v8::String::new(scope, arch).unwrap().into());
}

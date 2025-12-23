#!/usr/bin/env python3
"""Fix V8 scope mutable borrow issues in runtime_minimal.rs"""

import re

with open('src/runtime_minimal.rs', 'r') as f:
    content = f.read()

# Find the setup_module_system function and fix the require_fn closure
# The issue is that we call v8::String::new(scope, ...) multiple times in the same scope

# Pattern to find and replace: pre-create all string keys before require_fn
old_pattern = r'''(        let module_paths_key = v8::String::new\(scope, "paths"\)\.unwrap\(\)\.into\(\);
        module_obj\.set\(scope, module_paths_key, module_paths_arr\.into\(\)\);

        // Add module\.require function
        let require_fn = v8::Function::new\(scope, \|scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue\| \{)'''

new_code = r'''        let module_paths_key = v8::String::new(scope, "paths").unwrap().into();
        module_obj.set(scope, module_paths_key, module_paths_arr.into());

        // Pre-create all string keys for module system to avoid repeated scope borrowing
        let buffer_key = v8::String::new(scope, "Buffer").unwrap().into();
        let inspect_max_bytes_key = v8::String::new(scope, "INSPECT_MAX_BYTES").unwrap().into();
        let k_max_length_key = v8::String::new(scope, "kMaxLength").unwrap().into();
        let process_global_key = v8::String::new(scope, "process").unwrap().into();
        let join_key = v8::String::new(scope, "join").unwrap().into();
        let resolve_key = v8::String::new(scope, "resolve").unwrap().into();
        let dirname_key = v8::String::new(scope, "dirname").unwrap().into();
        let basename_key = v8::String::new(scope, "basename").unwrap().into();
        let extname_key = v8::String::new(scope, "extname").unwrap().into();
        let is_absolute_key = v8::String::new(scope, "isAbsolute").unwrap().into();
        let normalize_key = v8::String::new(scope, "normalize").unwrap().into();
        let delimiter_key = v8::String::new(scope, "delimiter").unwrap().into();
        let sep_key = v8::String::new(scope, "sep").unwrap().into();
        let on_key = v8::String::new(scope, "on").unwrap().into();
        let emit_key = v8::String::new(scope, "emit").unwrap().into();
        let inspect_key = v8::String::new(scope, "inspect").unwrap().into();
        let is_array_key = v8::String::new(scope, "isArray").unwrap().into();
        let is_regexp_key = v8::String::new(scope, "isRegExp").unwrap().into();
        let readable_key = v8::String::new(scope, "Readable").unwrap().into();
        let writable_key = v8::String::new(scope, "Writable").unwrap().into();
        let platform_key = v8::String::new(scope, "platform").unwrap().into();
        let arch_key = v8::String::new(scope, "arch").unwrap().into();
        let homedir_key = v8::String::new(scope, "homedir").unwrap().into();
        let url_constructor_key = v8::String::new(scope, "URL").unwrap().into();

        // Add module.require function
        let require_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {'''

content = re.sub(old_pattern, new_code, content)

# Fix buffer module - remove redundant buffer_key creation
content = content.replace('''                    if name == "buffer" || name == "Buffer" {
                        // Return Buffer object with static methods
                        let buffer_key = v8::String::new(scope, "Buffer").unwrap().into();
                        let global = context.global(scope);
                        if let Some(buffer_val) = global.get(scope, buffer_key) {
                            module_obj.set(scope, v8::String::new(scope, "Buffer").unwrap().into(), buffer_val);
                        }
                        // Add INSPECT_MAX_BYTES
                        let inspect_max_bytes_key = v8::String::new(scope, "INSPECT_MAX_BYTES").unwrap().into();
                        let inspect_max_bytes_val = v8::Integer::new(scope, 50);
                        module_obj.set(scope, inspect_max_bytes_key, inspect_max_bytes_val.into());
                        // Add kMaxLength
                        let k_max_length_key = v8::String::new(scope, "kMaxLength").unwrap().into();
                        let k_max_length_val = v8::Integer::new(scope, 2147483647);
                        module_obj.set(scope, k_max_length_key, k_max_length_val.into());
                        retval.set(module_obj.into());''', '''                    if name == "buffer" || name == "Buffer" {
                        // Return Buffer object with static methods
                        let global = context.global(scope);
                        if let Some(buffer_val) = global.get(scope, buffer_key) {
                            module_obj.set(scope, buffer_key, buffer_val);
                        }
                        // Add INSPECT_MAX_BYTES
                        let inspect_max_bytes_val = v8::Integer::new(scope, 50);
                        module_obj.set(scope, inspect_max_bytes_key, inspect_max_bytes_val.into());
                        // Add kMaxLength
                        let k_max_length_val = v8::Integer::new(scope, 2147483647);
                        module_obj.set(scope, k_max_length_key, k_max_length_val.into());
                        retval.set(module_obj.into());''')

# Fix process module - remove redundant process_global_key creation
content = content.replace('''                    } else if name == "process" || name == "Process" {
                        // Return process object with env and other properties
                        let process_global_key = v8::String::new(scope, "process").unwrap().into();
                        if let Some(process_val) = global.get(scope, process_global_key) {''', '''                    } else if name == "process" || name == "Process" {
                        // Return process object with env and other properties
                        if let Some(process_val) = global.get(scope, process_global_key) {''')

# Fix path.join - use join_key
content = content.replace('''                        if let Some(fn_val) = join_fn {
                            path_obj.set(scope, v8::String::new(scope, "join").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = join_fn {
                            path_obj.set(scope, join_key, fn_val);
                        }''')

# Fix path.resolve - use resolve_key
content = content.replace('''                        if let Some(fn_val) = resolve_fn {
                            path_obj.set(scope, v8::String::new(scope, "resolve").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = resolve_fn {
                            path_obj.set(scope, resolve_key, fn_val);
                        }''')

# Fix path.dirname - use dirname_key
content = content.replace('''                        if let Some(fn_val) = dirname_fn {
                            path_obj.set(scope, v8::String::new(scope, "dirname").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = dirname_fn {
                            path_obj.set(scope, dirname_key, fn_val);
                        }''')

# Fix path.basename - use basename_key
content = content.replace('''                        if let Some(fn_val) = basename_fn {
                            path_obj.set(scope, v8::String::new(scope, "basename").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = basename_fn {
                            path_obj.set(scope, basename_key, fn_val);
                        }''')

# Fix path.extname - use extname_key
content = content.replace('''                        if let Some(fn_val) = extname_fn {
                            path_obj.set(scope, v8::String::new(scope, "extname").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = extname_fn {
                            path_obj.set(scope, extname_key, fn_val);
                        }''')

# Fix path.isAbsolute - use is_absolute_key
content = content.replace('''                        if let Some(fn_val) = is_absolute_fn {
                            path_obj.set(scope, v8::String::new(scope, "isAbsolute").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = is_absolute_fn {
                            path_obj.set(scope, is_absolute_key, fn_val);
                        }''')

# Fix path.normalize - use normalize_key
content = content.replace('''                        if let Some(fn_val) = normalize_fn {
                            path_obj.set(scope, v8::String::new(scope, "normalize").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = normalize_fn {
                            path_obj.set(scope, normalize_key, fn_val);
                        }''')

# Fix path.delimiter - use delimiter_key
content = content.replace('''                        // path.delimiter
                        let delimiter_val = v8::String::new(scope, ":").unwrap().into();
                        path_obj.set(scope, v8::String::new(scope, "delimiter").unwrap().into(), delimiter_val);

                        // path.sep
                        let sep_val = v8::String::new(scope, "/").unwrap().into();
                        path_obj.set(scope, v8::String::new(scope, "sep").unwrap().into(), sep_val);''', '''                        // path.delimiter
                        let delimiter_val = v8::String::new(scope, ":").unwrap().into();
                        path_obj.set(scope, delimiter_key, delimiter_val);

                        // path.sep
                        let sep_val = v8::String::new(scope, "/").unwrap().into();
                        path_obj.set(scope, sep_key, sep_val);''')

# Fix events.on - use on_key
content = content.replace('''                        if let Some(fn_val) = on_fn {
                            events_obj.set(scope, v8::String::new(scope, "on").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = on_fn {
                            events_obj.set(scope, on_key, fn_val);
                        }''')

# Fix events.emit - use emit_key
content = content.replace('''                        if let Some(fn_val) = emit_fn {
                            events_obj.set(scope, v8::String::new(scope, "emit").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = emit_fn {
                            events_obj.set(scope, emit_key, fn_val);
                        }''')

# Fix util.inspect - use inspect_key
content = content.replace('''                        if let Some(fn_val) = inspect_fn {
                            util_obj.set(scope, v8::String::new(scope, "inspect").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = inspect_fn {
                            util_obj.set(scope, inspect_key, fn_val);
                        }''')

# Fix util.isArray - use is_array_key
content = content.replace('''                        if let Some(fn_val) = is_array_fn {
                            util_obj.set(scope, v8::String::new(scope, "isArray").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = is_array_fn {
                            util_obj.set(scope, is_array_key, fn_val);
                        }''')

# Fix util.isRegExp - use is_regexp_key
content = content.replace('''                        if let Some(fn_val) = is_regexp_fn {
                            util_obj.set(scope, v8::String::new(scope, "isRegExp").unwrap().into(), fn_val);
                        }''', '''                        if let Some(fn_val) = is_regexp_fn {
                            util_obj.set(scope, is_regexp_key, fn_val);
                        }''')

# Fix stream.Readable - use readable_key
content = content.replace('''                        let readable_val = v8::String::new(scope, "Readable").unwrap().into();
                        stream_obj.set(scope, v8::String::new(scope, "Readable").unwrap().into(), readable_val);
                        let writable_val = v8::String::new(scope, "Writable").unwrap().into();
                        stream_obj.set(scope, v8::String::new(scope, "Writable").unwrap().into(), writable_val);''', '''                        let readable_val = v8::String::new(scope, "Readable").unwrap().into();
                        stream_obj.set(scope, readable_key, readable_val);
                        let writable_val = v8::String::new(scope, "Writable").unwrap().into();
                        stream_obj.set(scope, writable_key, writable_val);''')

# Fix os module
content = content.replace('''                    } else if name == "os" || name == "Os" {
                        // Return os module
                        let os_obj = v8::Object::new(scope);
                        let platform_val = v8::String::new(scope, std::env::consts::OS).unwrap().into();
                        os_obj.set(scope, v8::String::new(scope, "platform").unwrap().into(), platform_val);
                        let arch_val = v8::String::new(scope, std::env::consts::ARCH).unwrap().into();
                        os_obj.set(scope, v8::String::new(scope, "arch").unwrap().into(), arch_val);
                        let homedir_val = v8::String::new(scope, &std::env::var("HOME").unwrap_or("/".to_string())).unwrap().into();
                        os_obj.set(scope, v8::String::new(scope, "homedir").unwrap().into(), homedir_val);
                        retval.set(os_obj.into());''', '''                    } else if name == "os" || name == "Os" {
                        // Return os module
                        let os_obj = v8::Object::new(scope);
                        let platform_val = v8::String::new(scope, std::env::consts::OS).unwrap().into();
                        os_obj.set(scope, platform_key, platform_val);
                        let arch_val = v8::String::new(scope, std::env::consts::ARCH).unwrap().into();
                        os_obj.set(scope, arch_key, arch_val);
                        let homedir_val = v8::String::new(scope, &std::env::var("HOME").unwrap_or("/".to_string())).unwrap().into();
                        os_obj.set(scope, homedir_key, homedir_val);
                        retval.set(os_obj.into());''')

# Fix url module
content = content.replace('''                    } else if name == "url" || name == "Url" {
                        // Return URL object
                        let url_constructor_key = v8::String::new(scope, "URL").unwrap().into();
                        if let Some(url_constructor) = global.get(scope, url_constructor_key) {
                            module_obj.set(scope, v8::String::new(scope, "URL").unwrap().into(), url_constructor);
                        }
                        retval.set(module_obj.into());''', '''                    } else if name == "url" || name == "Url" {
                        // Return URL object
                        if let Some(url_constructor) = global.get(scope, url_constructor_key) {
                            module_obj.set(scope, url_constructor_key, url_constructor);
                        }
                        retval.set(module_obj.into());''')

with open('src/runtime_minimal.rs', 'w') as f:
    f.write(content)

print("Fixed V8 scope mutable borrow issues!")

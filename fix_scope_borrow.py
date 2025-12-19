#!/usr/bin/env python3
"""Fix V8 scope mutable borrow errors by extracting intermediate variables."""

import re
import sys

def fix_set_call(match):
    """Fix obj.set(scope, v8::String::new(scope, "key").unwrap().into(), v8::Integer::new(scope, val).into());"""
    indent = match.group(1)
    obj = match.group(2)
    key_str = match.group(3)
    val_type = match.group(4)  # Integer, String, Boolean, Number
    val_arg = match.group(5)

    # Generate unique variable names based on key
    safe_key = re.sub(r'[^a-zA-Z0-9_]', '_', key_str).lower()

    return f'''{indent}let key_{safe_key} = v8::String::new(scope, "{key_str}").unwrap();
{indent}let val_{safe_key} = v8::{val_type}::new(scope, {val_arg});
{indent}{obj}.set(scope, key_{safe_key}.into(), val_{safe_key}.into());'''

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Pattern for: obj.set(scope, v8::String::new(scope, "KEY").unwrap().into(), v8::Integer::new(scope, VAL).into());
    pattern = r'^(\s*)(\w+)\.set\(scope, v8::String::new\(scope, "([^"]+)"\)\.unwrap\(\)\.into\(\), v8::(Integer|String|Boolean|Number)::new\(scope, ([^)]+)\)(?:\.unwrap\(\))?\.into\(\)\);'

    lines = content.split('\n')
    new_lines = []

    for line in lines:
        match = re.match(pattern, line)
        if match:
            fixed = fix_set_call(match)
            new_lines.append(fixed)
        else:
            new_lines.append(line)

    new_content = '\n'.join(new_lines)

    with open(filepath, 'w') as f:
        f.write(new_content)

    print(f"Fixed {filepath}")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python fix_scope_borrow.py <file>")
        sys.exit(1)

    fix_file(sys.argv[1])

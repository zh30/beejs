#!/usr/bin/env python3
"""Simplify module system by removing pre-created keys"""

import re

with open('src/runtime_minimal.rs', 'r') as f:
    content = f.read()

# Remove pre-created keys section
content = re.sub(
    r'        // Pre-create all string keys for module system to avoid repeated scope borrowing\n.*?url_constructor_key = v8::String::new\(scope, "URL"\)\.unwrap\(\)\.into\(\);\n\n',
    '',
    content,
    flags=re.DOTALL
)

# Restore original require_fn that creates keys dynamically but avoids nested closure issues
# by restructuring to not capture scope in nested functions

with open('src/runtime_minimal.rs', 'w') as f:
    f.write(content)

print("Simplified module system!")

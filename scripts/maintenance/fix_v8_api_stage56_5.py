#!/usr/bin/env python3
"""Fix V8 API issues in v8_bindings.rs"""

import re

# Read the file
with open('src/testing/v8_bindings.rs', 'r') as f:
    content = f.read()

# Pattern to match .into() calls that need to be converted to v8::String::new()
# This pattern looks for strings being set as keys
patterns = [
    # Pattern: .set(scope, "string".into(), ...)
    (r'(\.set\(scope, )"([^"]+)"(\.into\(\),)', r'\1v8::String::new(scope, "\2").unwrap().into()\3'),
    # Pattern: .set(scope, "string".into(), ...) with different spacing
    (r'(\.set\(scope,)"([^"]+)"(\.into\(\),)', r'\1 v8::String::new(scope, "\2").unwrap().into()\3'),
]

# Apply fixes
for pattern, replacement in patterns:
    content = re.sub(pattern, replacement, content)

# Write back
with open('src/testing/v8_bindings.rs', 'w') as f:
    f.write(content)

print("Fixed V8 API issues in v8_bindings.rs")

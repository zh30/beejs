#!/usr/bin/env python3
"""Check for files with missing imports"""

import os
import re

missing_tracing = []
missing_serde = []

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
                has_debug = 'debug!' in content
                has_trace_import = 'use tracing::' in content
                has_info = 'info!' in content
                has_warn = 'warn!' in content
                has_error = 'error!' in content

                if has_debug or has_info or has_warn or has_error:
                    if not has_trace_import:
                        missing_tracing.append(filepath)

                if 'Serialize' in content or 'Deserialize' in content:
                    if 'use serde::' not in content:
                        missing_serde.append(filepath)

print(f"Files missing tracing import: {len(missing_tracing)}")
for f in missing_tracing[:10]:
    print(f"  {f}")

print(f"\nFiles missing serde import: {len(missing_serde)}")
for f in missing_serde[:10]:
    print(f"  {f}")

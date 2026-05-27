#!/usr/bin/env python3
"""Find files using Duration but missing the import"""

import os
import re

duration_errors = []

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
                # Check if using Duration
                if 'Duration::' in content or 'Duration {' in content:
                    # Check if imported
                    if 'use std::time::Duration;' not in content and 'use std::time::{' not in content:
                        duration_errors.append(filepath)

print(f"Files missing Duration import: {len(duration_errors)}")
for f in duration_errors:
    print(f"  {f}")

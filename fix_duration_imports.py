#!/usr/bin/env python3
"""Fix missing Duration imports in all files"""

import os
import re

def fix_duration_import(filepath):
    """Add Duration import if missing"""
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    # Check if already has Duration import
    if any('use std::time::Duration' in line or 'use std::time::{' in line for line in lines):
        return False

    # Find the position to insert (after std:: imports)
    insert_idx = 0
    std_import_idx = -1
    
    for i, line in enumerate(lines):
        if line.startswith('use std::'):
            std_import_idx = i
    
    if std_import_idx >= 0:
        # Insert after the last std:: import
        insert_idx = std_import_idx + 1
        # Add newline after std imports if not present
        if insert_idx < len(lines) and not lines[insert_idx].strip() == '':
            lines.insert(insert_idx, '\n')
            insert_idx += 1
    else:
        # No std imports, insert after other use statements
        for i, line in enumerate(lines):
            if line.startswith('use '):
                insert_idx = i + 1
    
    lines.insert(insert_idx, 'use std::time::Duration;\n')
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.writelines(lines)
    return True

count = 0
for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
                if 'Duration::' in content or 'Duration {' in content:
                    if fix_duration_import(filepath):
                        count += 1
                        print(f"  Fixed {filepath}")

print(f"\nTotal files fixed: {count}")

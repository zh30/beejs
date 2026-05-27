#!/usr/bin/env python3
"""Fix imports by properly merging existing use statements"""

import os
import re

# Find files with Instant errors
files_to_check = set()
with os.popen('cargo check 2>&1') as f:
    for line in f:
        if 'use of undeclared type `Instant`' in line:
            m = re.search(r'--> (.+?):', line)
            if m:
                files_to_check.add(m.group(1))

print(f"Found {len(files_to_check)} files with Instant errors")

# Types and their import modules
TYPE_MODULES = {
    'Instant': ('std::time', ['Instant']),
    'Arc': ('std::sync', ['Arc']),
    'Mutex': ('std::sync', ['Mutex']),
    'HashMap': ('std::collections', ['HashMap']),
    'AtomicUsize': ('std::sync::atomic', ['AtomicUsize']),
    'VecDeque': ('std::collections', ['VecDeque']),
    'AtomicU64': ('std::sync::atomic', ['AtomicU64']),
}

def fix_file(filepath):
    """Fix imports in a single file"""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    lines = content.split('\n')
    std_imports = {}  # module -> [items]
    
    # Parse existing imports
    for i, line in enumerate(lines):
        if line.startswith('use '):
            # Parse std imports
            if line.startswith('use std::'):
                # Extract module and items
                match = re.match(r'use (.+?)::({(.+?)}|(.+?));', line)
                if match:
                    module = match.group(1) + '::'
                    items_str = match.group(3) or match.group(4)
                    items = [item.strip() for item in items_str.split(',')]
                    
                    if module not in std_imports:
                        std_imports[module] = set()
                    std_imports[module].update(items)
    
    # Find what types are used
    types_needed = {}
    for type_name, (module, _) in TYPE_MODULES.items():
        if re.search(r'\b' + type_name + r'\b', content):
            if module not in types_needed:
                types_needed[module] = set()
            types_needed[module].add(type_name)
    
    if not types_needed:
        return False
    
    # Update imports
    modified = False
    for module, needed_items in types_needed.items():
        module_key = module + '::' if not module.endswith('::') else module
        
        if module_key in std_imports:
            # Merge with existing
            existing = std_imports[module_key]
            new_items = needed_items - existing
            if new_items:
                # Find and update the import line
                for i, line in enumerate(lines):
                    if line.startswith(f'use {module}'):
                        # Parse existing items
                        match = re.match(r'(use .+?::)({.+?}|(.+?));', line)
                        if match:
                            prefix = match.group(1)
                            items_str = match.group(3) or match.group(4)
                            existing_items = [item.strip() for item in items_str.split(',')]
                            all_items = existing_items + list(new_items)
                            all_items.sort()
                            new_line = prefix + '{' + ', '.join(all_items) + '};'
                            lines[i] = new_line
                            modified = True
                            break
        else:
            # Add new import
            items_list = list(needed_items)
            items_list.sort()
            new_line = f'use {module}{{{", ".join(items_list)}}};'
            # Find position to insert (after last use statement or after docs)
            insert_idx = 0
            for i, line in enumerate(lines):
                if line.startswith('use '):
                    insert_idx = i + 1
                elif line.startswith('//!') or line.startswith('//'):
                    continue
                elif line.strip() == '':
                    continue
                else:
                    break
            lines.insert(insert_idx, new_line)
            modified = True
    
    if modified:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write('\n'.join(lines))
        return True
    
    return False

count = 0
for filepath in sorted(files_to_check):
    if os.path.exists(filepath):
        if fix_file(filepath):
            count += 1
            print(f"  Fixed {filepath}")

print(f"\nTotal files fixed: {count}")

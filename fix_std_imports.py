#!/usr/bin/env python3
"""Fix missing std imports based on type usage"""

import os
import re

# Map of type to import statement
TYPE_IMPORTS = {
    'Instant': 'use std::time::Instant;',
    'Arc': 'use std::sync::Arc;',
    'Mutex': 'use std::sync::Mutex;',
    'HashMap': 'use std::collections::HashMap;',
    'AtomicUsize': 'use std::sync::atomic::AtomicUsize;',
    'VecDeque': 'use std::collections::VecDeque;',
    'AtomicU64': 'use std::sync::atomic::AtomicU64;',
    'SystemTime': 'use std::time::SystemTime;',
    'File': 'use std::fs::File;',
    'HashSet': 'use std::collections::HashSet;',
    'DefaultHasher': 'use std::collections::hash_map::DefaultHasher;',
    'Ordering': 'use std::sync::atomic::Ordering;',
    'NonNull': 'use std::ptr::NonNull;',
}

def fix_file_imports(filepath):
    """Add missing std imports to a file"""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Check which types are used
    types_used = []
    for type_name in TYPE_IMPORTS.keys():
        # Check if type is used (not just in comments)
        if re.search(r'\b' + type_name + r'\b', content):
            types_used.append(type_name)
    
    if not types_used:
        return False
    
    # Check what's already imported
    lines = content.split('\n')
    use_std_imports = {}
    insert_idx = 0
    
    for i, line in enumerate(lines):
        if line.startswith('use std::'):
            insert_idx = i + 1
            # Parse existing imports
            if 'use std::collections::' in line:
                use_std_imports['collections'] = line
            elif 'use std::sync::' in line:
                use_std_imports['sync'] = line
            elif 'use std::time::' in line:
                use_std_imports['time'] = line
            elif 'use std::fs::' in line:
                use_std_imports['fs'] = line
            elif 'use std::ptr::' in line:
                use_std_imports['ptr'] = line
    
    # Build new imports
    new_imports = []
    
    # Time types
    time_types = [t for t in types_used if t in ['Instant', 'Duration', 'SystemTime']]
    if time_types and 'time' not in use_std_imports:
        if 'Duration' in time_types:
            new_imports.append('use std::time::{Duration, Instant, SystemTime};')
        else:
            new_imports.append('use std::time::{' + ', '.join(time_types) + '};')
    
    # Collections types
    coll_types = [t for t in types_used if t in ['HashMap', 'HashSet', 'VecDeque', 'DefaultHasher']]
    if coll_types and 'collections' not in use_std_imports:
        new_imports.append('use std::collections::{' + ', '.join(coll_types) + '};')
    
    # Sync types
    sync_types = [t for t in types_used if t in ['Arc', 'Mutex', 'AtomicUsize', 'AtomicU64', 'Ordering']]
    if sync_types and 'sync' not in use_std_imports:
        # Split atomic and sync
        atomic_types = [t for t in sync_types if 'Atomic' in t or t == 'Ordering']
        other_sync = [t for t in sync_types if t not in atomic_types]
        
        if atomic_types and other_sync:
            new_imports.append('use std::sync::{' + ', '.join(other_sync) + '};')
            new_imports.append('use std::sync::atomic::{' + ', '.join(atomic_types) + '};')
        elif atomic_types:
            new_imports.append('use std::sync::atomic::{' + ', '.join(atomic_types) + '};')
        elif other_sync:
            new_imports.append('use std::sync::{' + ', '.join(other_sync) + '};')
    
    # FS types
    fs_types = [t for t in types_used if t in ['File']]
    if fs_types and 'fs' not in use_std_imports:
        new_imports.append('use std::fs::File;')
    
    # Ptr types
    ptr_types = [t for t in types_used if t in ['NonNull']]
    if ptr_types and 'ptr' not in use_std_imports:
        new_imports.append('use std::ptr::NonNull;')
    
    if not new_imports:
        return False
    
    # Insert new imports after existing std imports
    for import_line in new_imports:
        lines.insert(insert_idx, import_line)
        insert_idx += 1
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines))
    
    return True

count = 0
for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            if fix_file_imports(filepath):
                count += 1

print(f"Total files fixed: {count}")

#!/usr/bin/env python3
"""
Systematic fix for missing std type imports
Fixes: Duration, Instant, HashMap, Ordering, UNIX_EPOCH, std::io, std::fs, std::sync::mpsc
"""

import os
import re
from pathlib import Path

# Files that need the imports
NEED_STD_TIME = [
    'src/automation/test_runner.rs',
    'src/automation/threshold.rs',
    'src/analysis/bottleneck_detector.rs',
    'src/analysis/optimizer.rs',
    'src/analysis/visualizer.rs',
]

NEED_STD_COLLECTIONS = [
    'src/automation/threshold.rs',
    'src/analysis/visualizer.rs',
]

NEED_STD_SYNC = [
    'src/automation/threshold.rs',
    'src/automation/report_generator.rs',
]

NEED_STD_IO = [
    'src/automation/threshold.rs',
]

def add_imports_to_file(filepath, new_imports):
    """Add imports to a file if they don't already exist"""
    if not os.path.exists(filepath):
        print(f"  File not found: {filepath}")
        return False

    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content
    modified = False

    # Group imports by category
    use_statements = []

    if 'std::time' in new_imports:
        if 'use std::time::' not in content:
            use_statements.append('use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};')
            modified = True

    if 'std::collections' in new_imports:
        if 'use std::collections::' not in content:
            use_statements.append('use std::collections::{HashMap, BTreeMap};')
            modified = True

    if 'std::sync' in new_imports:
        if 'use std::sync::atomic::' not in content:
            use_statements.append('use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};')
            modified = True

    if 'std::io' in new_imports:
        if 'use std::io::' not in content and 'std::io' in new_imports:
            # Check if already using std::io elsewhere
            if 'std::fs' in new_imports:
                use_statements.append('use std::io::{self, Read, Write};')
                use_statements.append('use std::fs;')
            else:
                use_statements.append('use std::io::{self, Read, Write};')
            modified = True

    if 'std::mpsc' in new_imports:
        if 'use std::sync::mpsc::' not in content:
            use_statements.append('use std::sync::mpsc;')
            modified = True

    # Insert use statements after existing use statements
    if use_statements and modified:
        # Find the last use statement in the file
        use_pattern = r'^use .*;$'
        use_matches = list(re.finditer(use_pattern, content, re.MULTILINE))

        if use_matches:
            last_use_end = use_matches[-1].end()
            # Insert after the last use statement
            insert_pos = content.find('\n', last_use_end) + 1
            new_content = content[:insert_pos] + '\n'.join(use_statements) + '\n' + content[insert_pos:]
            content = new_content
        else:
            # No use statements found, add at the beginning after doc comments
            lines = content.split('\n')
            insert_idx = 0
            for i, line in enumerate(lines):
                if line.startswith('//!') or line.startswith('//'):
                    insert_idx = i + 1
                else:
                    break
            lines.insert(insert_idx, '')
            lines.insert(insert_idx + 1, '\n'.join(use_statements))
            content = '\n'.join(lines)

    if modified:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  ✓ Updated {filepath}")
        return True
    else:
        print(f"  - No changes needed for {filepath}")
        return False

def main():
    print("Systematic Fix: Adding Missing Std Imports")
    print("=" * 60)

    total_updated = 0

    # Fix files needing std::time
    print("\n[1] Adding std::time imports...")
    for filepath in NEED_STD_TIME:
        if add_imports_to_file(filepath, ['std::time']):
            total_updated += 1

    # Fix files needing std::collections
    print("\n[2] Adding std::collections imports...")
    for filepath in NEED_STD_COLLECTIONS:
        if add_imports_to_file(filepath, ['std::collections']):
            total_updated += 1

    # Fix files needing std::sync
    print("\n[3] Adding std::sync imports...")
    for filepath in NEED_STD_SYNC:
        if add_imports_to_file(filepath, ['std::sync']):
            total_updated += 1

    # Fix files needing std::io and std::fs
    print("\n[4] Adding std::io and std::fs imports...")
    for filepath in NEED_STD_IO:
        if add_imports_to_file(filepath, ['std::io', 'std::fs']):
            total_updated += 1

    print("\n" + "=" * 60)
    print(f"Total files updated: {total_updated}")
    print("\nNext: Run 'cargo check' to see the improvement!")

if __name__ == '__main__':
    main()

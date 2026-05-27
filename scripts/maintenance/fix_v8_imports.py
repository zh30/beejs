#!/usr/bin/env python3
"""Fix v8 imports by adding use rusty_v8 as v8;"""

import os
import re

count = 0
for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Check if file uses v8:: but doesn't import it
            if 'v8::' in content and 'use rusty_v8' not in content and 'rusty_v8::' not in content:
                lines = content.split('\n')
                
                # Find where to insert the import (after other imports)
                insert_idx = 0
                for i, line in enumerate(lines):
                    if line.startswith('use ') and 'rusty_v8' not in line:
                        insert_idx = i + 1
                    elif line.startswith('//!') or line.startswith('//'):
                        continue
                    elif line.strip() == '':
                        continue
                    else:
                        break
                
                # Insert the import
                lines.insert(insert_idx, 'use rusty_v8 as v8;')
                
                with open(filepath, 'w', encoding='utf-8') as f:
                    f.write('\n'.join(lines))
                
                count += 1
                print(f"Fixed {filepath}")

print(f"\nTotal files fixed: {count}")

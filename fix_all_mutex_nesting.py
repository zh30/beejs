#!/usr/bin/env python3
import re
from pathlib import Path

def fix_mutex_nesting(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original = content
    
    # 修复嵌套的 Mutex 和 Arc
    patterns = [
        (r'Arc::new\(std::sync::Mutex::new\(std::sync::Mutex::new\(', 'Arc::new(Mutex::new('),
        (r'Arc::new\(std::sync::RwLock::new\(std::sync::RwLock::new\(', 'Arc::new(RwLock::new('),
        (r'std::sync::Mutex::new\(std::sync::Mutex::new\(', 'Mutex::new('),
        (r'std::sync::RwLock::new\(std::sync::RwLock::new\(', 'RwLock::new('),
        (r'Mutex::new\(RwLock::new\(RwLock::new\(', 'Mutex::new(RwLock::new('),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    if content != original:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Fixed: {file_path}")
        return True
    return False

for rust_file in Path('src').rglob('*.rs'):
    fix_mutex_nesting(rust_file)

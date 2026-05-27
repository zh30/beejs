#!/usr/bin/env python3
import re
from pathlib import Path

def fix_brackets(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original = content
    
    # 修复嵌套的括号
    patterns = [
        # 修复 Arc::new(Mutex::new(Arc::new( 模式
        (r'Arc::new\(Mutex::new\(Arc::new\(', 'Arc::new(Mutex::new('),
        # 修复额外的右括号
        (r'\)\)\s*\)', '))'),
        (r'\)\s*\)\s*\)', '))'),
        # 修复左括号缺失
        (r'Mutex::new\(\s*\(', 'Mutex::new('),
        (r'Arc::new\(\s*\(', 'Arc::new('),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    # 手动修复已知的问题模式
    lines = content.split('\n')
    fixed_lines = []
    for line in lines:
        # 修复 PerformanceRegressionDetector::new_default() 附近的括号
        if 'PerformanceRegressionDetector::new_default()' in line:
            line = re.sub(r'\)\s*\)\s*;', '));', line)
        
        fixed_lines.append(line)
    
    content = '\n'.join(fixed_lines)
    
    if content != original:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Fixed: {file_path}")
        return True
    return False

for rust_file in Path('src').rglob('*.rs'):
    fix_brackets(rust_file)

#!/usr/bin/env python3
"""
修复剩余的语法错误
"""

import re
import os
from pathlib import Path

def fix_syntax_errors(file_path):
    """修复语法错误"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content

        # 修复多余的 Mutex 包装
        content = re.sub(r'Mutex::new\(Mutex::new\(', 'Mutex::new(', content)
        content = re.sub(r'Arc::new\(Arc::new\(', 'Arc::new(', content)
        
        # 修复括号不匹配
        content = re.sub(r',\s*\)\)\)\s*,', ')),', content)
        
        # 修复缺少闭合括号的结构体初始化
        lines = content.split('\n')
        fixed_lines = []
        i = 0
        while i < len(lines):
            line = lines[i]
            
            # 检查是否是 QueryIndex 初始化
            if 'QueryIndex {' in line and 'time_index: Vec::new()' in line:
                # 查找对应的闭合括号
                brace_count = 0
                j = i
                while j < len(lines):
                    current = lines[j]
                    brace_count += current.count('{') - current.count('}')
                    if brace_count == 0:
                        break
                    j += 1
                
                # 重新构造这个结构体
                if j < len(lines):
                    # 找到所有字段
                    fields = []
                    for k in range(i, j+1):
                        if ':' in lines[k] and not lines[k].strip().startswith('//'):
                            fields.append(lines[k].strip())
                    
                    # 重新构造
                    indent = '            '
                    fixed_lines.append(f'{indent}query_index: Arc::new(Mutex::new(QueryIndex {{')
                    for field in fields:
                        fixed_lines.append(f'{indent}    {field},')
                    fixed_lines.append(f'{indent}}})),')
                    i = j + 1
                    continue
            
            fixed_lines.append(line)
            i += 1
        
        content = '\n'.join(fixed_lines)
        
        if content != original:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed syntax: {file_path}")
            return True
        return False

    except Exception as e:
        print(f"Error: {e}")
        return False

def main():
    src_dir = Path('src')
    fixed = 0
    
    for rust_file in src_dir.rglob('*.rs'):
        if fix_syntax_errors(rust_file):
            fixed += 1
    
    print(f"\nFixed {fixed} files with syntax errors")

if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""
Stage 46.2: Fix scope multiple mutable borrow errors

问题: obj.set(scope, v8::String::new(scope, "x").unwrap().into(), ...)
     同一语句中 scope 被多次可变借用

解决: 拆分为多行
     let key = v8::String::new(scope, "x").unwrap();
     obj.set(scope, key.into(), ...);
"""

import re
import sys
from pathlib import Path

def fix_inline_string_new(content: str) -> tuple[str, int]:
    """修复内联 v8::String::new() 调用"""
    lines = content.split('\n')
    new_lines = []
    changes = 0

    i = 0
    while i < len(lines):
        line = lines[i]

        # 模式1: obj.set(scope, v8::String::new(scope, "...").unwrap().into(), value);
        # 需要变成:
        # let _key = v8::String::new(scope, "...").unwrap();
        # obj.set(scope, _key.into(), value);

        pattern1 = r'^(\s*)(\w+)\.set\(scope,\s*v8::String::new\(scope,\s*([^)]+)\)\.unwrap\(\)\.into\(\),\s*(.+)\);$'
        match1 = re.match(pattern1, line)
        if match1:
            indent = match1.group(1)
            obj = match1.group(2)
            key_str = match1.group(3)
            value = match1.group(4)
            # 生成唯一变量名
            var_name = f"_key_{changes}"
            new_lines.append(f"{indent}let {var_name} = v8::String::new(scope, {key_str}).unwrap();")
            new_lines.append(f"{indent}{obj}.set(scope, {var_name}.into(), {value});")
            changes += 1
            print(f"  修复 set pattern1: {line.strip()[:50]}...")
            i += 1
            continue

        # 模式2: obj.set_index(scope, index, v8::String::new(scope, ...).unwrap().into());
        pattern2 = r'^(\s*)(\w+)\.set_index\(scope,\s*(\w+),\s*v8::String::new\(scope,\s*([^)]+)\)\.unwrap\(\)\.into\(\)\);$'
        match2 = re.match(pattern2, line)
        if match2:
            indent = match2.group(1)
            obj = match2.group(2)
            index = match2.group(3)
            str_content = match2.group(4)
            var_name = f"_val_{changes}"
            new_lines.append(f"{indent}let {var_name} = v8::String::new(scope, {str_content}).unwrap();")
            new_lines.append(f"{indent}{obj}.set_index(scope, {index}, {var_name}.into());")
            changes += 1
            print(f"  修复 set_index: {line.strip()[:50]}...")
            i += 1
            continue

        new_lines.append(line)
        i += 1

    return '\n'.join(new_lines), changes

def process_file(filepath: Path) -> int:
    content = filepath.read_text()
    new_content, changes = fix_inline_string_new(content)
    if changes > 0:
        filepath.write_text(new_content)
    return changes

def main():
    nodejs_core = Path('src/nodejs_core')
    if not nodejs_core.exists():
        print("Error: src/nodejs_core not found")
        sys.exit(1)

    total_files = 0
    total_changes = 0

    for filepath in sorted(nodejs_core.glob('*.rs')):
        changes = process_file(filepath)
        if changes > 0:
            total_files += 1
            total_changes += changes
            print(f"  ✅ {filepath.name}: {changes} 处修复")

    print(f"\n总计: {total_files} 个文件, {total_changes} 处修改")
    return total_changes

if __name__ == '__main__':
    main()

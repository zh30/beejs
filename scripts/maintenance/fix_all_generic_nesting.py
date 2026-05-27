#!/usr/bin/env python3
"""
批量修复所有文件中的泛型嵌套错误
"""

import re
import os

def fix_file_generic_nesting(file_path):
    """修复单个文件中的泛型嵌套错误"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content
        changes = 0

        # 修复模式1: HashMap<String, T, std::collections::HashMap<...>>
        # 匹配任意深度的嵌套 HashMap
        pattern1 = r'HashMap<([^,]+),\s*([^,]+),\s*std::collections::HashMap<[^>]+>>'
        def replace_hashmap(match):
            nonlocal changes
            changes += 1
            key_type = match.group(1).strip()
            value_type = match.group(2).strip()
            return f'HashMap<{key_type}, {value_type}>'

        content = re.sub(pattern1, replace_hashmap, content)

        # 修复模式2: Vec<T, std::collections::...>
        pattern2 = r'Vec<([^,]+),\s*std::collections::[^>]+>'
        def replace_vec(match):
            nonlocal changes
            changes += 1
            inner_type = match.group(1).strip()
            return f'Vec<{inner_type}>'

        content = re.sub(pattern2, replace_vec, content)

        # 修复模式3: Arc<RwLock<...>, 多余的括号
        pattern3 = r'Arc<RwLock<([^>]+)>>,'
        def replace_arc_rwlock(match):
            nonlocal changes
            changes += 1
            inner_type = match.group(1).strip()
            return f'Arc<RwLock<{inner_type}>>,'

        content = re.sub(pattern3, replace_arc_rwlock, content)

        # 修复模式4: Arc<Mutex<...>, 多余的括号
        pattern4 = r'Arc<Mutex<([^>]+)>>,'
        def replace_arc_mutex(match):
            nonlocal changes
            changes += 1
            inner_type = match.group(1).strip()
            return f'Arc<Mutex<{inner_type}>>,'

        content = re.sub(pattern4, replace_arc_mutex, content)

        # 修复模式5: 双逗号
        pattern5 = r',,'
        def replace_double_comma(match):
            nonlocal changes
            changes += 1
            return ','

        content = re.sub(pattern5, replace_double_comma, content)

        if changes > 0:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✓ {file_path}: 修复了 {changes} 个泛型嵌套错误")
            return changes
        else:
            print(f"- {file_path}: 无需修复")
            return 0

    except Exception as e:
        print(f"✗ {file_path}: 修复失败 - {e}")
        return 0

def find_rust_files():
    """查找所有 Rust 源文件"""
    rust_files = []
    for root, dirs, files in os.walk('src'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    return rust_files

def main():
    print("开始批量修复泛型嵌套错误...\n")

    rust_files = find_rust_files()
    total_changes = 0

    for file_path in rust_files:
        changes = fix_file_generic_nesting(file_path)
        total_changes += changes

    print(f"\n✅ 修复完成! 总计修复了 {total_changes} 个错误")
    print(f"检查了 {len(rust_files)} 个文件")

if __name__ == "__main__":
    main()

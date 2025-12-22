#!/usr/bin/env python3
"""
批量修复缺失的导入语句
"""

import os
import re
from pathlib import Path

def fix_missing_imports(file_path):
    """修复单个文件的缺失导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content

        # 检查是否使用了 Arc 或 Mutex 但没有导入
        has_arc_usage = 'Arc<' in content or 'Arc::' in content
        has_mutex_usage = 'Mutex<' in content or 'Mutex::' in content

        if has_arc_usage or has_mutex_usage:
            # 检查是否已经有 std::sync 导入
            has_std_sync_import = 'use std::sync::' in content

            if has_std_sync_import:
                # 更新现有的 std::sync 导入
                content = re.sub(
                    r'use std::sync::\{([^}]+)\};',
                    lambda m: add_missing_sync_types(m.group(1)),
                    content
                )
            else:
                # 在合适位置添加 std::sync 导入
                # 找到最后一个 use 语句的位置
                use_statements = list(re.finditer(r'^use .+;$', content, re.MULTILINE))
                if use_statements:
                    last_use_end = use_statements[-1].end()
                    # 在最后一个 use 语句后插入
                    imports_to_add = []
                    if has_arc_usage and 'Arc' not in content:
                        imports_to_add.append('Arc')
                    if has_mutex_usage and 'Mutex' not in content:
                        imports_to_add.append('Mutex')

                    if imports_to_add:
                        new_import = f"use std::sync::{', '.join(imports_to_add)};\n"
                        content = content[:last_use_end] + '\n' + new_import + content[last_use_end:]

        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def add_missing_sync_types(existing_types):
    """添加缺失的同步类型到现有的导入中"""
    types = [t.strip() for t in existing_types.split(',')]
    if 'Arc' not in types:
        types.append('Arc')
    if 'Mutex' not in types:
        types.append('Mutex')
    return f"use std::sync::{{{', '.join(types)}}};"

def main():
    """主函数"""
    src_dir = Path('/Users/henry/code/beejs/src')

    # 查找所有 Rust 文件
    rust_files = list(src_dir.rglob('*.rs'))

    fixed_count = 0
    for file_path in rust_files:
        if fix_missing_imports(file_path):
            fixed_count += 1

    print(f"\nFixed {fixed_count} files")

if __name__ == '__main__':
    main()

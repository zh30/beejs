#!/usr/bin/env python3
"""
修复重复导入问题
"""

import re
from pathlib import Path

def fix_duplicate_imports(file_path):
    """修复单个文件的重复导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()

        new_lines = []
        i = 0
        while i < len(lines):
            line = lines[i]

            # 检查是否是 std::sync 导入
            if 'use std::sync::' in line and line.strip().endswith(';'):
                # 收集所有连续的 std::sync 导入
                sync_imports = []
                j = i
                while j < len(lines) and 'use std::sync::' in lines[j] and lines[j].strip().endswith(';'):
                    sync_imports.append(lines[j].strip())
                    j += 1

                # 合并导入
                all_types = set()
                for sync_import in sync_imports:
                    # 提取类型
                    match = re.search(r'use std::sync::\{([^}]+)\};', sync_import)
                    if match:
                        types = [t.strip() for t in match.group(1).split(',')]
                        all_types.update(types)

                # 生成合并后的导入
                if all_types:
                    types_str = ', '.join(sorted(all_types))
                    merged_import = f"use std::sync::{{{types_str}}};\n"
                    new_lines.append(merged_import)

                i = j
            else:
                new_lines.append(line)
                i += 1

        # 检查是否有变化
        new_content = ''.join(new_lines)
        with open(file_path, 'r', encoding='utf-8') as f:
            old_content = f.read()

        if new_content != old_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Fixed: {file_path}")
            return True
        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """主函数"""
    src_dir = Path('/Users/henry/code/beejs/src')

    # 查找所有 Rust 文件
    rust_files = list(src_dir.rglob('*.rs'))

    fixed_count = 0
    for file_path in rust_files:
        if fix_duplicate_imports(file_path):
            fixed_count += 1

    print(f"\nFixed {fixed_count} files")

if __name__ == '__main__':
    main()

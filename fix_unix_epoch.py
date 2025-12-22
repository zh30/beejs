#!/usr/bin/env python3
"""
修复 UNIX_EPOCH 导入错误 (E0425)
在使用了 UNIX_EPOCH 但没有导入的文件中添加导入
"""

import re
from pathlib import Path

def fix_file_unix_epoch(file_path):
    """修复文件中的 UNIX_EPOCH 导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # 检查是否使用了 UNIX_EPOCH
        if 'UNIX_EPOCH' not in content:
            return False

        # 检查是否已经导入了 UNIX_EPOCH
        if 'use std::time::{' in content or 'use std::time::UNIX_EPOCH' in content:
            return False

        lines = content.split('\n')
        modified = False

        # 查找 std::time 导入的位置
        time_import_idx = -1
        for i, line in enumerate(lines):
            if line.strip().startswith('use std::time::'):
                time_import_idx = i
                break

        if time_import_idx >= 0:
            # 如果已有 std::time 导入，添加 UNIX_EPOCH
            line = lines[time_import_idx]
            if line.strip().endswith('{'):
                # 格式: use std::time::{
                lines[time_import_idx] = line.rstrip('{') + '{UNIX_EPOCH, '
                modified = True
            elif '}' in line:
                # 格式: use std::time::{...}
                lines[time_import_idx] = line.replace('}', ', UNIX_EPOCH}', 1)
                modified = True
        else:
            # 添加新的导入
            # 找到最后一个 use 语句的位置
            last_use_idx = -1
            for i, line in enumerate(lines):
                if line.strip().startswith('use '):
                    last_use_idx = i

            if last_use_idx >= 0:
                # 在最后一个 use 语句后插入新导入
                new_import = "use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};"
                lines.insert(last_use_idx + 1, new_import)
                modified = True

        if modified:
            new_content = '\n'.join(lines)
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            return True

        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """主函数"""
    print("🔧 修复 UNIX_EPOCH 导入错误...")

    # 查找所有 Rust 文件
    rust_files = list(Path('src').glob('**/*.rs'))
    fixed_count = 0

    for file_path in rust_files:
        if fix_file_unix_epoch(file_path):
            print(f"  ✅ 修复: {file_path}")
            fixed_count += 1

    print(f"\n✨ 修复完成! 共修复 {fixed_count} 个文件")
    print("运行 'cargo check' 查看剩余错误")

if __name__ == '__main__':
    main()

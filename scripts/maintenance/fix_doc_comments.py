#!/usr/bin/env python3
"""
修复 Rust 文档注释位置错误 (E0753)
将内部的 //!/*** 注释转换为外部的 ////** 注释
"""

import re
from pathlib import Path

def fix_file_doc_comments(file_path):
    """修复文件中的文档注释"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content
        lines = content.split('\n')
        modified = False

        # 修复模式：//! -> //
        for i, line in enumerate(lines):
            if line.strip().startswith('//!') or line.strip().startswith('/*!'):
                # 检查是否是内部文档注释（在 use 语句前）
                # 如果下一行是 use 语句，则这是内部注释
                if i + 1 < len(lines) and lines[i + 1].strip().startswith('use '):
                    # 替换为外部注释
                    if line.strip().startswith('//!'):
                        lines[i] = line.replace('//!', '///', 1)
                    else:
                        lines[i] = line.replace('/*!', '/**', 1)
                    modified = True
                else:
                    # 如果不是内部注释，替换为普通注释
                    if line.strip().startswith('//!'):
                        lines[i] = line.replace('//!', '//', 1)
                    elif line.strip().startswith('/*!'):
                        lines[i] = line.replace('/*!', '/*', 1)
                    modified = True

        # 修复模式：/** 内部的 !*/
        for i, line in enumerate(lines):
            if '/*!' in line and '!*/' in line:
                # 简单替换
                lines[i] = line.replace('/*!', '/**', 1).replace('!*/', '*/', 1)
                modified = True
            elif '/*!' in line:
                lines[i] = line.replace('/*!', '/*', 1)
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
    print("🔧 修复 E0753 文档注释位置错误...")

    # 查找所有 Rust 文件
    rust_files = list(Path('src').glob('**/*.rs'))
    fixed_count = 0

    for file_path in rust_files:
        if fix_file_doc_comments(file_path):
            print(f"  ✅ 修复: {file_path}")
            fixed_count += 1

    print(f"\n✨ 修复完成! 共修复 {fixed_count} 个文件")
    print("运行 'cargo check' 查看剩余错误")

if __name__ == '__main__':
    main()

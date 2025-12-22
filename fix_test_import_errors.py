#!/usr/bin/env python3
"""
修复测试文件中的导入语法错误
将 use std::collections{HashMap} 改为 use std::collections::{HashMap}
"""

import os
import re
import glob

def fix_import_syntax(file_path):
    """修复单个文件中的导入语法错误"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content

        # 匹配模式: use 模块名{内容} -> use 模块名::{内容}
        # 匹配各种情况
        patterns = [
            (r'use\s+(\w+(?:::\w+)*)\s*\{([^}]+)\}', r'use \1::{\2}'),
            (r'use\s+(\w+)\s*\{([^{}]+)\}', r'use \1::{\2}'),
        ]

        for pattern, replacement in patterns:
            content = re.sub(pattern, replacement, content)

        # 如果内容有变化，写回文件
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ 修复文件: {file_path}")
            return True
        else:
            return False

    except Exception as e:
        print(f"❌ 修复文件失败 {file_path}: {e}")
        return False

def main():
    """主函数"""
    print("🔧 开始修复测试文件中的导入语法错误...")

    # 获取所有测试文件
    test_files = glob.glob("tests/*.rs")
    fixed_count = 0

    for test_file in test_files:
        if fix_import_syntax(test_file):
            fixed_count += 1

    print(f"🎉 修复完成! 共修复了 {fixed_count} 个文件")

if __name__ == "__main__":
    main()
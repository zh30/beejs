#!/usr/bin/env python3
"""
修复导入语法错误
"""

import re
from pathlib import Path

def fix_import_syntax(file_path: Path) -> bool:
    """修复导入语法错误"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content

        # 修复错误的导入语法: use std::sync::Arc, Mutex, ;
        # 正确的应该是: use std::sync::{Arc, Mutex};
        content = re.sub(
            r'use std::sync::Arc,\s*Mutex,\s*;',
            'use std::sync::{Arc, Mutex};',
            content
        )

        # 修复其他类似的错误
        content = re.sub(
            r'use std::sync::Arc,\s*;',
            'use std::sync::{Arc};',
            content
        )

        if content != original:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    print("🔧 修复导入语法错误...")
    print("=" * 60)

    src_dir = Path("src")
    fixed_count = 0

    for file_path in src_dir.rglob("*.rs"):
        if fix_import_syntax(file_path):
            fixed_count += 1

    print(f"\n✅ 修复了 {fixed_count} 个文件的语法错误")

if __name__ == "__main__":
    main()

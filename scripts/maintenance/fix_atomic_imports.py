#!/usr/bin/env python3
"""
修复未解析的导入错误
主要是 Atomic* 类型的导入问题
"""

import re
from pathlib import Path

def fix_atomic_imports(file_path: Path) -> bool:
    """修复原子类型导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content
        modifications = []

        # 模式1: 如果导入了 AtomicUsize 但没有导入 Ordering，添加 Ordering
        if 'std::sync::AtomicUsize' in content and 'std::sync::Ordering' not in content:
            # 查找 use std::sync::AtomicUsize; 行
            content = re.sub(
                r'(use std::sync::AtomicUsize;)',
                r'\1\nuse std::sync::Ordering;',
                content
            )
            modifications.append("Added missing std::sync::Ordering import")

        # 模式2: 如果导入了 AtomicBool 但没有导入 Ordering，添加 Ordering
        if 'std::sync::AtomicBool' in content and 'std::sync::Ordering' not in content:
            content = re.sub(
                r'(use std::sync::AtomicBool;)',
                r'\1\nuse std::sync::Ordering;',
                content
            )
            modifications.append("Added missing std::sync::Ordering import for AtomicBool")

        # 模式3: 如果同时需要 AtomicUsize, AtomicBool 和 Ordering，合并导入
        if ('std::sync::AtomicUsize' in content or 'std::sync::AtomicBool' in content) and 'std::sync::Ordering' in content:
            # 合并为一行
            atomic_imports = []
            if 'std::sync::AtomicUsize' in content:
                atomic_imports.append('AtomicUsize')
                content = re.sub(r'use std::sync::AtomicUsize;', '', content)
            if 'std::sync::AtomicBool' in content:
                atomic_imports.append('AtomicBool')
                content = re.sub(r'use std::sync::AtomicBool;', '', content)
            if 'std::sync::Ordering' in content:
                atomic_imports.append('Ordering')
                content = re.sub(r'use std::sync::Ordering;', '', content)

            if atomic_imports:
                # 添加合并的导入
                import_line = f"use std::sync::{', '.join(atomic_imports)};"
                # 在最后一个 use 语句后添加
                use_pattern = r'(use [^\n]+;\n?)((?:use [^\n]+;\n?)*)(?=\n(?:[^u]|\Z))'
                content = re.sub(use_pattern, lambda m: m.group(1) + m.group(2) + import_line + '\n', content, count=1)
                modifications.append(f"Merged atomic imports: {', '.join(atomic_imports)}")

        if content != original:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed {file_path}:")
            for mod in modifications:
                print(f"  - {mod}")
            return True
        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    print("🔧 修复未解析的导入错误 (Atomic* 类型)...")
    print("=" * 60)

    src_dir = Path("src")
    fixed_count = 0

    for file_path in src_dir.rglob("*.rs"):
        if fix_atomic_imports(file_path):
            fixed_count += 1

    print(f"\n✅ 修复了 {fixed_count} 个文件的导入错误")

if __name__ == "__main__":
    main()

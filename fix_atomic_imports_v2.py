#!/usr/bin/env python3
"""
修复原子类型导入错误
Ordering 应该在 std::sync::atomic 中，而不是 std::sync 中
"""

import re
from pathlib import Path

def fix_atomic_imports_v2(file_path: Path) -> bool:
    """修复原子类型导入 - 正确的版本"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content
        modifications = []

        # 修复模式: std::sync::{AtomicUsize, Ordering} -> std::sync::atomic::AtomicUsize 和 std::sync::Ordering
        # 或者更好的方式：std::sync::atomic::{AtomicUsize, AtomicBool, Ordering}

        # 模式1: 如果同时导入了 AtomicUsize 和 Ordering 从 std::sync
        if re.search(r'use std::sync::\{[^}]*AtomicUsize[^}]*Ordering[^}]*\};', content):
            # 分离 atomic 类型和 sync 类型
            content = re.sub(
                r'use std::sync::\{([^}]*AtomicUsize[^}]*Ordering[^}]*)\};',
                lambda m: f'use std::sync::atomic::{m.group(1).replace("AtomicUsize", "").replace("Ordering", "").replace(",,", ",").strip(",")};\nuse std::sync::Ordering;',
                content
            )
            modifications.append("Fixed AtomicUsize and Ordering import path")

        # 模式2: 如果只导入了 AtomicUsize 从 std::sync
        if re.search(r'use std::sync::\{[^}]*AtomicUsize[^}]*\};', content) and 'Ordering' not in content:
            content = re.sub(
                r'use std::sync::\{([^}]*AtomicUsize[^}]*)\};',
                lambda m: f'use std::sync::atomic::{m.group(1)};',
                content
            )
            modifications.append("Fixed AtomicUsize import path")

        # 模式3: 如果只导入了 AtomicBool 从 std::sync
        if re.search(r'use std::sync::\{[^}]*AtomicBool[^}]*\};', content):
            content = re.sub(
                r'use std::sync::\{([^}]*AtomicBool[^}]*)\};',
                lambda m: f'use std::sync::atomic::{m.group(1)};',
                content
            )
            modifications.append("Fixed AtomicBool import path")

        # 模式4: 如果同时导入了 AtomicUsize 和 AtomicBool
        if re.search(r'use std::sync::\{[^}]*Atomic(Usize|Bool)[^}]*\};', content):
            # 提取所有 atomic 类型
            atomic_types = []
            if 'AtomicUsize' in content:
                atomic_types.append('AtomicUsize')
            if 'AtomicBool' in content:
                atomic_types.append('AtomicBool')

            # 移除原来的导入
            content = re.sub(r'use std::sync::\{[^}]*Atomic(Usize|Bool)[^}]*\};', '', content)

            # 添加正确的 atomic 导入
            import_line = f"use std::sync::atomic::{', '.join(atomic_types)};"
            content = import_line + '\n' + content

            # 保留其他非 atomic 的导入
            if 'Arc' in original or 'Mutex' in original:
                sync_types = []
                if 'Arc' in original and re.search(r'use std::sync::\{[^}]*Arc[^}]*\};', original):
                    sync_types.append('Arc')
                if 'Mutex' in original and re.search(r'use std::sync::\{[^}]*Mutex[^}]*\};', original):
                    sync_types.append('Mutex')
                if sync_types:
                    sync_import = f"use std::sync::{', '.join(sync_types)};"
                    content = sync_import + '\n' + content

            if 'Ordering' in original:
                content = "use std::sync::Ordering;\n" + content

            modifications.append(f"Fixed atomic imports: {', '.join(atomic_types)}")

        # 模式5: 简单的直接替换错误模式
        if 'std::sync::AtomicUsize' in content and 'std::sync::atomic' not in content:
            content = content.replace('std::sync::AtomicUsize', 'std::sync::atomic::AtomicUsize')
            modifications.append("Direct replace AtomicUsize path")

        if 'std::sync::AtomicBool' in content and 'std::sync::atomic' not in content:
            content = content.replace('std::sync::AtomicBool', 'std::sync::atomic::AtomicBool')
            modifications.append("Direct replace AtomicBool path")

        if 'std::sync::Ordering' in content and 'atomic' in content:
            # 如果已经在 atomic 导入中，确保 Ordering 在正确的位置
            content = re.sub(
                r'use std::sync::atomic::\{([^}]*)\};',
                lambda m: f'use std::sync::atomic::{{{m.group(1)}, Ordering}};',
                content
            )
            modifications.append("Added Ordering to atomic imports")

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
    print("🔧 修复原子类型导入错误 (v2)...")
    print("=" * 60)

    src_dir = Path("src")
    fixed_count = 0

    for file_path in src_dir.rglob("*.rs"):
        if fix_atomic_imports_v2(file_path):
            fixed_count += 1

    print(f"\n✅ 修复了 {fixed_count} 个文件的导入错误")

if __name__ == "__main__":
    main()

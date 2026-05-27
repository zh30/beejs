#!/usr/bin/env python3
"""
全面修复所有名称重复定义错误
"""

import re
from pathlib import Path

def fix_all_name_conflicts(file_path: Path) -> bool:
    """修复所有名称冲突"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content
        modifications = []

        # 修复 Duration 冲突: tokio::time::Duration -> TokioDuration
        if re.search(r'use std::time::Duration;', content) and re.search(r'use tokio::time::Duration', content):
            content = re.sub(r'use tokio::time::Duration', 'use tokio::time::Duration as TokioDuration', content)
            modifications.append("Duration: tokio::time::Duration -> TokioDuration")

        # 修复 Instant 冲突: tokio::time::Instant -> TokioInstant
        if re.search(r'use std::time::Instant;', content) and re.search(r'use tokio::time::Instant', content):
            content = re.sub(r'use tokio::time::Instant', 'use tokio::time::Instant as TokioInstant', content)
            modifications.append("Instant: tokio::time::Instant -> TokioInstant")

        # 修复 HashMap 重复导入 - 移除重复的
        use_lines = content.split('\n')
        new_lines = []
        seen_hashmap = False
        for line in use_lines:
            if 'use std::collections::HashMap' in line:
                if not seen_hashmap:
                    new_lines.append(line)
                    seen_hashmap = True
                # 跳过重复的
            else:
                new_lines.append(line)
        if seen_hashmap and len(new_lines) != len(use_lines):
            content = '\n'.join(new_lines)
            modifications.append("HashMap: removed duplicate import")

        # 修复 Mutex 重复导入 - 移除重复的
        use_lines = content.split('\n')
        new_lines = []
        seen_mutex = False
        for line in use_lines:
            if 'use std::sync::Mutex' in line:
                if not seen_mutex:
                    new_lines.append(line)
                    seen_mutex = True
                # 跳过重复的
            else:
                new_lines.append(line)
        if seen_mutex and len(new_lines) != len(use_lines):
            content = '\n'.join(new_lines)
            modifications.append("Mutex: removed duplicate import")

        # 修复同时导入 std::time::{Duration, Instant} 和 tokio::time::{Duration, Instant} 的情况
        if re.search(r'use std::time::\{[^}]*Duration[^}]*\};', content) and re.search(r'use tokio::time::\{Duration', content):
            # 重命名 tokio 的版本
            content = re.sub(r'use tokio::time::\{Duration', 'use tokio::time::{TokioDuration', content)
            content = re.sub(r', Instant\}', ', TokioInstant}', content)
            modifications.append("Duration/Instant: renamed tokio versions")

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
    print("🔧 修复所有名称重复定义错误...")
    print("=" * 60)

    src_dir = Path("src")
    fixed_count = 0

    for file_path in src_dir.rglob("*.rs"):
        if fix_all_name_conflicts(file_path):
            fixed_count += 1

    print(f"\n✅ 修复了 {fixed_count} 个文件的名称冲突")

if __name__ == "__main__":
    main()

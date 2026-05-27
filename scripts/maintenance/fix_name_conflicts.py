#!/usr/bin/env python3
"""
修复名称重复定义错误
处理以下冲突：
- RwLock: std::sync::RwLock vs tokio::sync::RwLock
- Duration: std::time::Duration vs tokio::time::Duration
- Instant: std::time::Instant vs tokio::time::Instant
- Mutex: 重复导入
- HashMap: 重复导入
"""

import os
import re
from pathlib import Path
from typing import Dict, List, Tuple

def find_rust_files() -> List[Path]:
    """查找所有Rust源文件"""
    rust_files = []
    src_dir = Path("src")
    if src_dir.exists():
        for file_path in src_dir.rglob("*.rs"):
            rust_files.append(file_path)
    return rust_files

def fix_name_conflicts(file_path: Path) -> Tuple[bool, int, List[str]]:
    """
    修复文件中的名称冲突
    返回: (是否修改, 修改次数, 修改列表)
    """
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content
        modifications = []
        modifications_count = 0

        # 修复模式1: RwLock 冲突 - 使用 tokio::sync::RwLock as AsyncRwLock
        # 如果同时导入了 std::sync::RwLock 和 tokio::sync::RwLock
        if re.search(r'use std::sync.*RwLock.*;', content) and re.search(r'use tokio::sync::RwLock;', content):
            # 替换 tokio RwLock 为 AsyncRwLock
            content = re.sub(r'use tokio::sync::RwLock;', 'use tokio::sync::RwLock as AsyncRwLock;', content)
            modifications.append("RwLock: tokio::sync::RwLock -> AsyncRwLock")
            modifications_count += 1

        # 修复模式2: Duration 冲突 - 使用 tokio::time::Duration as TokioDuration
        if re.search(r'use std::time::Duration;', content) and re.search(r'use tokio::time::Duration', content):
            content = re.sub(r'use tokio::time::Duration', 'use tokio::time::Duration as TokioDuration', content)
            modifications.append("Duration: tokio::time::Duration -> TokioDuration")
            modifications_count += 1

        # 修复模式3: Instant 冲突 - 使用 tokio::time::Instant as TokioInstant
        if re.search(r'use std::time::Instant;', content) and re.search(r'use tokio::time::Instant', content):
            content = re.sub(r'use tokio::time::Instant', 'use tokio::time::Instant as TokioInstant', content)
            modifications.append("Instant: tokio::time::Instant -> TokioInstant")
            modifications_count += 1

        # 修复模式4: 修复 HashMap 重复导入
        # 如果在同一行中重复导入 HashMap
        hashmap_pattern = r'use std::collections::HashMap.*?;.*?use std::collections::HashMap.*?;'
        if re.search(hashmap_pattern, content, re.DOTALL):
            # 移除重复的导入，保留第一个
            lines = content.split('\n')
            new_lines = []
            seen_hashmap = False
            for line in lines:
                if 'use std::collections::HashMap' in line:
                    if not seen_hashmap:
                        new_lines.append(line)
                        seen_hashmap = True
                else:
                    new_lines.append(line)
            content = '\n'.join(new_lines)
            modifications.append("HashMap: removed duplicate import")
            modifications_count += 1

        # 修复模式5: 修复 Mutex 重复导入
        mutex_pattern = r'use std::sync::Mutex.*?;.*?use std::sync::Mutex.*?;'
        if re.search(mutex_pattern, content, re.DOTALL):
            lines = content.split('\n')
            new_lines = []
            seen_mutex = False
            for line in lines:
                if 'use std::sync::Mutex' in line:
                    if not seen_mutex:
                        new_lines.append(line)
                        seen_mutex = True
                else:
                    new_lines.append(line)
            content = '\n'.join(new_lines)
            modifications.append("Mutex: removed duplicate import")
            modifications_count += 1

        # 修复模式6: 修复同时导入 std::sync::{Arc, Mutex, RwLock} 和 tokio::sync::RwLock 的情况
        if re.search(r'use std::sync::\{[^}]*RwLock[^}]*\};', content) and re.search(r'use tokio::sync::RwLock', content):
            # 从 std::sync 导入中移除 RwLock，因为 tokio 有异步版本
            content = re.sub(
                r'use std::sync::\{([^}]*)RwLock([^}]*)\};',
                lambda m: f'use std::sync::{m.group(1)}{m.group(2)};',
                content
            )
            # 重命名 tokio RwLock
            content = re.sub(r'use tokio::sync::RwLock;', 'use tokio::sync::RwLock as AsyncRwLock;', content)
            modifications.append("RwLock: separated std and tokio imports")
            modifications_count += 1

        # 写入修改后的内容
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True, modifications_count, modifications
        else:
            return False, 0, []

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False, 0, []

def main():
    print("🔧 开始修复名称重复定义错误...")
    print("=" * 60)

    rust_files = find_rust_files()
    print(f"📁 找到 {len(rust_files)} 个 Rust 文件")

    total_modifications = 0
    modified_files = 0
    all_modifications = []

    for file_path in rust_files:
        modified, count, mods = fix_name_conflicts(file_path)
        if modified:
            modified_files += 1
            total_modifications += count
            print(f"\n✅ {file_path}:")
            for mod in mods:
                print(f"   - {mod}")
            all_modifications.extend([(file_path, mod) for mod in mods])

    print("\n" + "=" * 60)
    print(f"📊 修复总结:")
    print(f"   修改文件数: {modified_files}")
    print(f"   总修改次数: {total_modifications}")
    print(f"   未修改文件: {len(rust_files) - modified_files}")

    if all_modifications:
        print(f"\n📝 所有修改:")
        for file_path, mod in all_modifications[:20]:  # 只显示前20个
            print(f"   - {file_path}: {mod}")
        if len(all_modifications) > 20:
            print(f"   ... 还有 {len(all_modifications) - 20} 个修改")

    print("\n✅ 名称冲突修复完成!")

if __name__ == "__main__":
    main()

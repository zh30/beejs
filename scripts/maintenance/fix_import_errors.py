#!/usr/bin/env python3
"""
自动化编译错误修复脚本
批量修复 Rust 项目中的导入错误
"""

import re
import subprocess
import sys
from pathlib import Path

def get_compilation_errors():
    """获取编译错误"""
    result = subprocess.run(
        ["cargo", "check", "2>&1"],
        capture_output=True,
        text=True
    )
    return result.stdout

def parse_errors(output):
    """解析编译错误"""
    errors = []
    lines = output.split('\n')

    for line in lines:
        if line.startswith('error['):
            # 提取错误类型
            match = re.match(r'error\[([A-Z0-9]+)\]:\s*(.+)', line)
            if match:
                error_type = match.group(1)
                message = match.group(2)
                errors.append({'type': error_type, 'message': message, 'line': line})

    return errors

def fix_file_imports(file_path, content):
    """修复文件中的导入错误"""
    lines = content.split('\n')
    modified = False

    # 常见的缺失导入
    common_imports = {
        'HashMap': 'use std::collections::HashMap;',
        'BTreeMap': 'use std::collections::BTreeMap;',
        'Duration': 'use std::time::Duration;',
        'Instant': 'use std::time::Instant;',
        'SystemTime': 'use std::time::SystemTime;',
        'UNIX_EPOCH': 'use std::time::{SystemTime, UNIX_EPOCH};',
        'Arc': 'use std::sync::Arc;',
        'Mutex': 'use std::sync::Mutex;',
        'RwLock': 'use std::sync::RwLock;',
        'Ordering': 'use std::sync::atomic::{AtomicUsize, Ordering};',
        'AtomicUsize': 'use std::sync::atomic::{AtomicUsize, Ordering};',
        'AtomicBool': 'use std::sync::atomic::{AtomicBool, Ordering};',
    }

    # 检查是否已经导入了这些类型
    existing_imports = set()
    for line in lines:
        if line.strip().startswith('use '):
            existing_imports.add(line.strip())

    # 添加缺失的导入
    new_imports = []
    for type_name, import_stmt in common_imports.items():
        if import_stmt not in existing_imports:
            # 检查是否在文件中使用了这个类型
            usage_pattern = re.compile(rf'\b{type_name}\b')
            if any(usage_pattern.search(line) for line in lines):
                new_imports.append(import_stmt)

    # 在现有导入之后添加新导入
    if new_imports:
        # 找到最后一个 use 语句的位置
        last_use_idx = -1
        for i, line in enumerate(lines):
            if line.strip().startswith('use '):
                last_use_idx = i

        if last_use_idx >= 0:
            # 在最后一个 use 语句后插入新导入
            lines = lines[:last_use_idx+1] + new_imports + lines[last_use_idx+1:]
            modified = True

    return '\n'.join(lines), modified

def main():
    """主函数"""
    print("🔧 开始修复编译错误...")

    # 获取编译错误
    print("📊 分析编译错误...")
    output = get_compilation_errors()
    errors = parse_errors(output)

    # 按错误类型分组
    error_types = {}
    for error in errors:
        error_type = error['type']
        if error_type not in error_types:
            error_types[error_type] = []
        error_types[error_type].append(error)

    print(f"📈 发现 {len(errors)} 个错误:")
    for error_type, errs in error_types.items():
        print(f"  - {error_type}: {len(errs)} 个")

    # 修复导入错误 (E0433)
    if 'E0433' in error_types:
        print("\n🔧 修复 E0433 导入错误...")
        rust_files = list(Path('src').glob('**/*.rs'))

        for file_path in rust_files:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                new_content, modified = fix_file_imports(file_path, content)

                if modified:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(new_content)
                    print(f"  ✅ 修复: {file_path}")
            except Exception as e:
                print(f"  ❌ 错误: {file_path} - {e}")

    print("\n✨ 修复完成!")
    print("运行 'cargo check' 查看剩余错误")

if __name__ == '__main__':
    main()

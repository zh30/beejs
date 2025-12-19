#!/usr/bin/env python3
"""
Stage 46: Fix v8::String::new() Option<Local> 类型转换问题

自动添加 .unwrap() 到缺失的 v8::String::new() 调用
"""

import re
import sys
from pathlib import Path

def fix_file(filepath: Path) -> int:
    """修复单个文件，返回修改次数"""
    content = filepath.read_text()
    original = content
    changes = 0

    # 模式: let xxx = v8::String::new(scope, ...); (没有 .unwrap())
    # 需要排除已经有 .unwrap() 的情况

    lines = content.split('\n')
    new_lines = []

    for line in lines:
        # 检查是否是 v8::String::new 赋值行
        if 'v8::String::new(' in line and '= v8::String::new(' in line:
            # 检查行尾是否缺少 .unwrap()
            stripped = line.rstrip()
            if stripped.endswith(');') and '.unwrap()' not in line and '.unwrap_or' not in line:
                # 需要添加 .unwrap()
                # 找到最后的 ); 位置，在 ) 前插入 .unwrap()
                # e.g., "let x = v8::String::new(scope, &foo);"
                # =>   "let x = v8::String::new(scope, &foo).unwrap();"
                idx = stripped.rfind(');')
                if idx > 0:
                    new_line = stripped[:idx] + ').unwrap();'
                    # 保持原有缩进
                    indent = len(line) - len(line.lstrip())
                    new_line = line[:indent] + new_line.lstrip()
                    new_lines.append(new_line)
                    changes += 1
                    print(f"  修复: {stripped[:60]}...")
                    continue

        new_lines.append(line)

    if changes > 0:
        filepath.write_text('\n'.join(new_lines))

    return changes

def main():
    nodejs_core = Path('src/nodejs_core')
    if not nodejs_core.exists():
        print("Error: src/nodejs_core not found")
        sys.exit(1)

    total_files = 0
    total_changes = 0

    files_to_fix = list(nodejs_core.glob('*.rs'))

    print(f"扫描 {len(files_to_fix)} 个文件...")

    for filepath in sorted(files_to_fix):
        changes = fix_file(filepath)
        if changes > 0:
            total_files += 1
            total_changes += changes
            print(f"  ✅ {filepath.name}: {changes} 处修复")

    print(f"\n总计: {total_files} 个文件, {total_changes} 处修改")
    return total_changes

if __name__ == '__main__':
    changes = main()
    sys.exit(0 if changes >= 0 else 1)

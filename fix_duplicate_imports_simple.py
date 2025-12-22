#!/usr/bin/env python3
"""
简单的重复导入修复脚本
使用更直接的方法查找和修复重复导入
"""

import re
import os
import sys
from pathlib import Path

def fix_file(file_path):
    """修复单个文件的重复导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content
        lines = content.split('\n')
        modified = False

        # 查找所有 use 语句的行号和内容
        use_lines = []
        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('use ') and stripped.endswith(';'):
                use_lines.append((i, stripped))

        # 查找并修复 HashMap 重复导入
        for i, use_line in use_lines:
            # 查找模式: use std::collections::HashMap;
            # 和下一个 use std::collections::{HashMap, ...};

            if 'use std::collections::HashMap;' in use_line:
                # 查找下一个 use std::collections::{
                for j in range(i + 1, min(i + 10, len(lines))):  # 在接下来的10行内查找
                    next_line = lines[j].strip()
                    if next_line.startswith('use std::collections::{') and 'HashMap' in next_line:
                        # 提取花括号内的内容
                        match = re.search(r'\{([^}]+)\}', next_line)
                        if match:
                            inside = match.group(1)
                            # 移除 HashMap，只保留其他类型
                            imports = [imp.strip() for imp in inside.split(',')]
                            filtered = [imp for imp in imports if 'HashMap' not in imp]

                            if filtered:
                                # 重新构造行
                                lines[j] = next_line.replace(inside, ', '.join(filtered))
                            else:
                                # 如果花括号为空，删除整行
                                lines[j] = ''
                            modified = True
                        break

            # 查找并修复 Arc 重复导入
            if 'use std::sync::Arc;' in use_line:
                for j in range(i + 1, min(i + 10, len(lines))):
                    next_line = lines[j].strip()
                    if next_line.startswith('use std::sync::{') and 'Arc' in next_line:
                        match = re.search(r'\{([^}]+)\}', next_line)
                        if match:
                            inside = match.group(1)
                            imports = [imp.strip() for imp in inside.split(',')]
                            filtered = [imp for imp in imports if 'Arc' not in imp]

                            if filtered:
                                lines[j] = next_line.replace(inside, ', '.join(filtered))
                            else:
                                lines[j] = ''
                            modified = True
                        break

            # 查找并修复 Mutex 重复导入
            if 'use std::sync::Mutex;' in use_line:
                for j in range(i + 1, min(i + 10, len(lines))):
                    next_line = lines[j].strip()
                    if next_line.startswith('use std::sync::{') and 'Mutex' in next_line:
                        match = re.search(r'\{([^}]+)\}', next_line)
                        if match:
                            inside = match.group(1)
                            imports = [imp.strip() for imp in inside.split(',')]
                            filtered = [imp for imp in imports if 'Mutex' not in imp]

                            if filtered:
                                lines[j] = next_line.replace(inside, ', '.join(filtered))
                            else:
                                lines[j] = ''
                            modified = True
                        break

        if modified:
            new_content = '\n'.join(lines)
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            return True

        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return False

def main():
    """主函数"""
    src_dir = Path(__file__).parent / 'src'

    if not src_dir.exists():
        print(f"Source directory not found: {src_dir}")
        sys.exit(1)

    fixed_count = 0

    # 递归查找所有 .rs 文件
    for rs_file in src_dir.rglob('*.rs'):
        if fix_file(rs_file):
            fixed_count += 1
            print(f"Fixed: {rs_file}")

    print(f"\n✅ 批量修复完成！")
    print(f"📁 修复文件数: {fixed_count}")

if __name__ == '__main__':
    main()

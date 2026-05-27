#!/usr/bin/env python3
"""
高级重复导入修复脚本
处理花括号对花括号的重复导入模式
"""

import re
import os
import sys
from pathlib import Path
from collections import defaultdict

def extract_use_imports(content):
    """提取所有 use 导入语句中的类型"""
    lines = content.split('\n')
    use_statements = []

    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith('use ') and stripped.endswith(';'):
            # 提取模块路径和类型
            use_statements.append({
                'line_num': i + 1,
                'content': line,
                'module_path': extract_module_path(stripped),
                'types': extract_types(stripped)
            })

    return use_statements

def extract_module_path(use_line):
    """提取模块路径"""
    match = re.search(r'use\s+([^;{]+)', use_line)
    return match.group(1).strip() if match else ''

def extract_types(use_line):
    """提取导入的类型"""
    types = set()

    # 处理简单形式: use std::sync::Arc;
    simple_match = re.search(r'use\s+[^:]+::(\w+);', use_line)
    if simple_match:
        types.add(simple_match.group(1))
        return types

    # 处理花括号形式: use std::sync::{Arc, Mutex};
    brace_match = re.search(r'\{([^}]+)\}', use_line)
    if brace_match:
        inside = brace_match.group(1)
        # 处理嵌套花括号，如 atomic::{AtomicUsize, Ordering}
        nested_pattern = re.findall(r'(\w+)(?:::\w+)*(?:\s*,\s*|$)', inside)
        for t in nested_pattern:
            t = t.strip().rstrip(',')
            if t and not t.startswith('atomic'):
                types.add(t)

    return types

def fix_file(file_path):
    """修复单个文件的重复导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content
        lines = content.split('\n')
        use_statements = extract_use_imports(content)

        modified = False
        types_to_remove = defaultdict(set)  # type_name -> {line_num to clean}

        # 第一步：查找重复的类型并决定保留哪个导入
        for i in range(len(use_statements)):
            for j in range(i + 1, len(use_statements)):
                use1 = use_statements[i]
                use2 = use_statements[j]

                # 只处理相同模块路径的重复
                if use1['module_path'] != use2['module_path']:
                    continue

                # 查找重复的类型
                common_types = use1['types'] & use2['types']

                if common_types:
                    # 决定保留哪个导入（保留类型更多的那个）
                    if len(use1['types']) >= len(use2['types']):
                        # 保留 use1，标记 use2 中的重复类型
                        for t in common_types:
                            types_to_remove[t].add(use2['line_num'])
                    else:
                        # 保留 use2，标记 use1 中的重复类型
                        for t in common_types:
                            types_to_remove[t].add(use1['line_num'])

        # 第二步：实际修改文件
        for type_name, line_nums in types_to_remove.items():
            for line_num in sorted(line_nums, reverse=True):
                actual_line_idx = line_num - 1
                line = lines[actual_line_idx]

                # 从花括号中移除该类型
                if '{' in line and type_name in line:
                    brace_match = re.search(r'\{([^}]+)\}', line)
                    if brace_match:
                        inside = brace_match.group(1)
                        imports = [imp.strip() for imp in inside.split(',')]
                        filtered = [imp for imp in imports if imp != type_name]

                        if filtered:
                            lines[actual_line_idx] = line.replace(inside, ', '.join(filtered))
                        else:
                            # 如果花括号为空，删除整行
                            lines[actual_line_idx] = ''
                        modified = True

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

    print(f"\n✅ 高级批量修复完成！")
    print(f"📁 修复文件数: {fixed_count}")

if __name__ == '__main__':
    main()

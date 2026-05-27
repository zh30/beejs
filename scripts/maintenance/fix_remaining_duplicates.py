#!/usr/bin/env python3
"""
修复剩余的重复导入错误
"""

import re
import os
from pathlib import Path

# 需要修复的文件列表
FILES_TO_FIX = [
    "src/ai_inference/ai_inference_engine.rs",
    "src/ai_inference/gpu_accelerate.rs",
    "src/cli/enhanced_cli.rs",
    "src/cloud/mod.rs",
    "src/debugger/engine.rs",
    "src/distributed/cluster_console.rs",
    "src/distributed/distributed_metrics.rs",
    "src/distributed/distributed_tracer.rs",
    "src/network/async_zero_copy.rs",
    "src/network/batch_io.rs",
    "src/network/io_uring.rs",
    "src/network/stage93_intelligent_prefetch.rs",
    "src/network/zero_copy_network.rs",
    "src/realtime/collaboration.rs",
    "src/runtime_config.rs",
    "src/startup/lazy_init.rs",
    "src/wasm_optimized/cache_manager.rs",
    "src/wasm_optimized/zero_copy_loader.rs",
]

def fix_file(file_path):
    """修复单个文件的重复导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content
        lines = content.split('\n')

        # 收集所有 use 语句
        use_statements = []
        use_line_indices = []

        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('use ') and stripped.endswith(';'):
                use_statements.append(stripped)
                use_line_indices.append(i)

        # 找出重复的类型
        types_used = {}
        for stmt in use_statements:
            # 提取大括号内的类型
            if '{' in stmt and '}' in stmt:
                match = re.search(r'\{([^}]+)\}', stmt)
                if match:
                    inside = match.group(1)
                    for typ in [t.strip() for t in inside.split(',')]:
                        if typ not in types_used:
                            types_used[typ] = []
                        types_used[typ].append(stmt)

        # 合并重复的类型
        modified = False
        new_use_statements = []

        # 先处理单独导入的类型
        for stmt in use_statements:
            if '{' not in stmt:
                # 这是单独导入，检查是否有重复
                typ = stmt.replace('use ', '').replace(';', '')
                if typ in types_used and len(types_used[typ]) > 1:
                    # 有重复，跳过这个单独导入
                    modified = True
                    continue
            new_use_statements.append(stmt)

        # 然后处理花括号导入
        combined_imports = {}
        for stmt in use_statements:
            if '{' in stmt and '}' in stmt:
                # 提取导入前缀和类型列表
                match = re.search(r'use ([^:]+)::\{([^}]+)\}', stmt)
                if match:
                    prefix = match.group(1)
                    types = [t.strip() for t in match.group(2).split(',')]

                    if prefix not in combined_imports:
                        combined_imports[prefix] = set()

                    # 添加类型到集合中（去除重复）
                    for typ in types:
                        combined_imports[prefix].add(typ)

        # 重新构建 use 语句
        final_use_statements = []

        # 添加合并后的花括号导入
        for prefix, types in combined_imports.items():
            if types:
                sorted_types = sorted(list(types))
                stmt = f"use {prefix}::{{{', '.join(sorted_types)}}};"
                final_use_statements.append(stmt)

        # 如果没有花括号导入，添加单独的类型导入
        for typ, stmts in types_used.items():
            if len(stmts) == 1 and '{' not in stmts[0]:
                final_use_statements.append(stmts[0])

        # 重建文件内容
        new_lines = []
        use_idx = 0

        for i, line in enumerate(lines):
            if i in use_line_indices:
                # 替换 use 语句
                if use_idx < len(final_use_statements):
                    new_lines.append(final_use_statements[use_idx])
                    use_idx += 1
                # 跳过原始 use 语句
                continue
            else:
                new_lines.append(line)

        new_content = '\n'.join(new_lines)

        if new_content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Fixed: {file_path}")
            return True
        else:
            return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    fixed_count = 0
    for file_path in FILES_TO_FIX:
        if os.path.exists(file_path):
            if fix_file(file_path):
                fixed_count += 1
        else:
            print(f"File not found: {file_path}")

    print(f"\n✅ 修复完成！")
    print(f"📁 修复文件数: {fixed_count}")

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
修复重复导入错误脚本 v2.0
专门修复 HashMap、Arc、Mutex 等类型的重复导入问题

模式：
    use std::collections::HashMap;  // 第一次导入
    ...
    use std::collections::{HashMap, BTreeMap};  // 重复导入 HashMap

修复为：
    use std::collections::HashMap;
    ...
    use std::collections::BTreeMap;  // 只保留 BTreeMap
"""

import re
import os
import sys
from pathlib import Path

def find_duplicate_imports(content):
    """查找重复导入"""
    lines = content.split('\n')
    use_statements = []
    duplicate_groups = {}

    # 收集所有 use 语句
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith('use ') and stripped.endswith(';'):
            use_statements.append({
                'line_num': i + 1,
                'content': stripped,
                'full_line': line
            })

    # 查找重复的导入 - 改进算法
    imports_by_module = {}

    for use in use_statements:
        # 提取完整的模块路径
        match = re.search(r'use\s+([^;]+);', use['content'])
        if match:
            module_path = match.group(1).strip()
            if module_path not in imports_by_module:
                imports_by_module[module_path] = []
            imports_by_module[module_path].append(use)

    # 现在查找在同一模块内的重复类型
    # 例如: use std::collections::HashMap; 和 use std::collections::{HashMap, BTreeMap};
    all_types_by_module = {}

    for use in use_statements:
        # 提取模块路径和类型
        match = re.search(r'use\s+([^:]+)::([^;{]+)(?:\{([^}]+)\})?;', use['content'])
        if match:
            module_prefix = match.group(1).strip()
            base_type = match.group(2).strip()
            types_in_braces = match.group(3)

            if types_in_braces:
                # 处理花括号形式: use std::collections::{HashMap, BTreeMap}
                for type_name in [t.strip() for t in types_in_braces.split(',')]:
                    key = (module_prefix, type_name)
                    if key not in all_types_by_module:
                        all_types_by_module[key] = []
                    all_types_by_module[key].append(use)
            else:
                # 处理单独形式: use std::collections::HashMap
                key = (module_prefix, base_type)
                if key not in all_types_by_module:
                    all_types_by_module[key] = []
                all_types_by_module[key].append(use)

    # 识别重复的导入组合
    for (module_prefix, type_name), imports in all_types_by_module.items():
        if len(imports) > 1:
            # 寻找单独导入和花括号导入的组合
            single_import = None
            multi_import = None

            for imp in imports:
                content = imp['content']
                if f'::{type_name}' in content and '{' in content and type_name in content:
                    # 这是花括号形式包含该类型
                    multi_import = imp
                elif content.endswith(f'::{type_name};') or content.endswith(f'::{type_name} {{'):
                    # 这是单独导入
                    single_import = imp

            if single_import and multi_import:
                duplicate_groups[type_name] = (single_import, multi_import)

    return duplicate_groups

def fix_duplicate_imports(content, duplicate_groups):
    """修复重复导入"""
    lines = content.split('\n')

    for imported_type, (single_import, multi_import) in duplicate_groups.items():
        # 修复花括号导入，移除重复的类型
        multi_line_idx = multi_import['line_num'] - 1

        # 提取花括号内的内容
        match = re.search(r'\{([^}]+)\}', lines[multi_line_idx])
        if match:
            inside_braces = match.group(1)
            # 分割并清理每个导入
            imports = [imp.strip() for imp in inside_braces.split(',')]
            # 过滤掉重复的类型
            filtered_imports = [imp for imp in imports if not imp.endswith(f'::{imported_type}')]

            if filtered_imports:
                # 重新构造导入语句
                lines[multi_line_idx] = lines[multi_line_idx].replace(
                    inside_braces,
                    ', '.join(filtered_imports)
                )
            else:
                # 如果花括号为空，删除整行
                lines[multi_line_idx] = ''

    return '\n'.join(lines)

def process_file(file_path):
    """处理单个文件"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        duplicate_groups = find_duplicate_imports(content)

        if duplicate_groups:
            fixed_content = fix_duplicate_imports(content, duplicate_groups)

            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)

            return len(duplicate_groups)
        return 0

    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return 0

def main():
    """主函数"""
    src_dir = Path(__file__).parent / 'src'

    if not src_dir.exists():
        print(f"Source directory not found: {src_dir}")
        sys.exit(1)

    total_fixed = 0
    fixed_files = []

    # 递归查找所有 .rs 文件
    for rs_file in src_dir.rglob('*.rs'):
        fixed_count = process_file(rs_file)
        if fixed_count > 0:
            total_fixed += fixed_count
            fixed_files.append(str(rs_file))

    print(f"\n✅ 修复完成！")
    print(f"📁 修复文件数: {len(fixed_files)}")
    print(f"🔧 修复重复导入组: {total_fixed}")
    print(f"\n修复的文件:")
    for file in fixed_files:
        print(f"  - {file}")

if __name__ == '__main__':
    main()

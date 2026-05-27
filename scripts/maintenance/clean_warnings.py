#!/usr/bin/env python3
"""
智能清理编译警告 - Stage 61
自动删除未使用的导入和修复常见警告
"""
import re
import subprocess
import sys
from pathlib import Path

def parse_warnings():
    """解析编译警告"""
    result = subprocess.run(
        ['cargo', 'build', '--lib', '2>&1'],
        capture_output=True,
        text=True
    )

    warnings = {
        'unused_imports': [],
        'non_snake_case': [],
        'dropping_references': [],
        'other': []
    }

    for line in result.stderr.split('\n'):
        # 解析文件位置
        location_match = re.search(r'--> (.+):(\d+):(\d+)', line)
        if not location_match:
            continue

        file_path = location_match.group(1)
        line_num = int(location_match.group(2))

        # 解析警告类型
        if 'unused import' in line or 'unused imports' in line:
            warnings['unused_imports'].append((file_path, line_num, line))
        elif 'non_snake_case' in line:
            warnings['non_snake_case'].append((file_path, line_num, line))
        elif 'dropping_references' in line:
            warnings['dropping_references'].append((file_path, line_num, line))
        else:
            warnings['other'].append((file_path, line_num, line))

    return warnings

def remove_unused_imports(file_path, line_num, import_text):
    """删除单个未使用的导入"""
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    # 查找导入语句（可能是多行）
    # 构建搜索模式
    import_patterns = [
        rf'^\s*use\s+{re.escape(import_text)}\s*;',
        rf'^\s*use\s+{re.escape(import_text.replace("::", r'::'))}\s* as\s+\w+\s*;',
    ]

    for i, line in enumerate(lines):
        if i == line_num - 1:  # line_num 是 1 索引的
            for pattern in import_patterns:
                if re.search(pattern, line):
                    # 删除整行
                    lines[i] = ''
                    print(f"  ✅ 删除未使用导入: {file_path}:{line_num} - {line.strip()}")
                    return True

    return False

def fix_dropping_references(file_path, line_num):
    """修复 dropping_references 警告"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # 替换 drop(...) 为 ...
    lines = content.split('\n')

    for i, line in enumerate(lines):
        if i == line_num - 1:
            # 查找 drop 调用
            if 'drop(' in line:
                # 简单替换：如果有 drop(some_var) -> some_var
                drop_match = re.search(r'drop\s*\(\s*(\w+)\s*\)', line)
                if drop_match:
                    var_name = drop_match.group(1)
                    new_line = line.replace(f'drop({var_name})', var_name)
                    lines[i] = new_line
                    print(f"  ✅ 修复 dropping_references: {file_path}:{line_num}")
                    print(f"    {line.strip()} -> {new_line.strip()}")
                    return True

    return False

def main():
    """主函数"""
    print("🔧 开始清理编译警告...\n")

    warnings = parse_warnings()

    # 1. 清理未使用的导入
    print("1️⃣ 清理未使用的导入:")
    removed_count = 0
    for file_path, line_num, line in warnings['unused_imports']:
        # 提取导入名称
        import_match = re.search(r'unused (?:import|imports?): `(.+?)`', line)
        if import_match:
            import_text = import_match.group(1)
            if remove_unused_imports(file_path, line_num, import_text):
                removed_count += 1

    print(f"   删除 {removed_count} 个未使用的导入\n")

    # 2. 修复 dropping_references
    print("2️⃣ 修复 dropping_references 警告:")
    fixed_count = 0
    for file_path, line_num, _ in warnings['dropping_references']:
        if fix_dropping_references(file_path, line_num):
            fixed_count += 1

    print(f"   修复 {fixed_count} 个 dropping_references 警告\n")

    # 3. 统计其他警告
    print("3️⃣ 其他警告:")
    print(f"   non_snake_case: {len(warnings['non_snake_case'])}")
    print(f"   其他: {len(warnings['other'])}")

    print(f"\n✅ 清理完成! 共修改 {removed_count + fixed_count} 个警告")

if __name__ == '__main__':
    main()

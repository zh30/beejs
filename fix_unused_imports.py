#!/usr/bin/env python3
"""
批量删除未使用的导入 - Stage 61 编译警告清理
"""
import re
import subprocess
import sys

def get_unused_imports():
    """获取所有未使用的导入"""
    result = subprocess.run(
        ['cargo', 'build', '--lib', '2>&1'],
        capture_output=True,
        text=True
    )

    # 解析未使用的导入
    unused_pattern = re.compile(r'warning: unused (?:import|imports?): (.+)')
    unused_imports = []

    for line in result.stderr.split('\n'):
        match = unused_pattern.search(line)
        if match:
            unused = match.group(1)
            unused_imports.append(unused)

    return unused_imports

def clean_unused_imports(rust_file_path):
    """清理单个文件中的未使用导入"""
    with open(rust_file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content

    # 移除未使用的 use 语句
    # 匹配 use 语句并标记是否被使用
    lines = content.split('\n')
    cleaned_lines = []

    for line in lines:
        # 跳过已删除的导入
        if line.strip().startswith('// REMOVED: unused import'):
            continue

        # 检查是否是 use 语句
        if line.strip().startswith('use '):
            # 跳过 if cfg 块内的 use 语句
            if any(cfg in line for cfg in ['#[cfg', '#cfg']):
                cleaned_lines.append(line)
                continue

            # 检查是否被标记为未使用（在注释中）
            if '/* UNUSED' in line or '// UNUSED' in line:
                print(f"  移除: {line.strip()}")
                continue

            cleaned_lines.append(line)
        else:
            cleaned_lines.append(line)

    new_content = '\n'.join(cleaned_lines)

    if new_content != original_content:
        with open(rust_file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False

def main():
    """主函数"""
    print("开始清理未使用的导入...")

    # 获取所有 .rs 文件
    result = subprocess.run(
        ['find', 'src', '-name', '*.rs'],
        capture_output=True,
        text=True
    )

    rust_files = result.stdout.strip().split('\n')

    modified_count = 0
    for rust_file in rust_files:
        print(f"\n处理文件: {rust_file}")
        if clean_unused_imports(rust_file):
            modified_count += 1

    print(f"\n✅ 完成! 修改了 {modified_count} 个文件")

if __name__ == '__main__':
    main()

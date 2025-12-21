#!/usr/bin/env python3
"""
详细修复时间戳相关问题
1. 确保导入 std::time::{SystemTime, UNIX_EPOCH}
2. 修复 elapsed() 方法调用问题
"""

import os
import re
import glob

def fix_timestamp_in_file(file_path):
    """修复单个文件中的时间戳类型和相关问题"""
    with open(file_path, 'r') as f:
        content = f.read()

    original_content = content

    # 1. 确保添加必要的导入
    # 检查是否已经有 std::time 导入
    time_import_pattern = r'use\s+std::time::\{[^}]*\};'
    has_time_import = bool(re.search(time_import_pattern, content))

    if has_time_import:
        # 更新现有的导入，添加 SystemTime 和 UNIX_EPOCH
        def add_imports(match):
            imports = match.group(0)
            # 移除行末的分号和换行
            imports = imports.replace(';', '').strip()
            # 添加 SystemTime 和 UNIX_EPOCH
            if 'SystemTime' not in imports:
                imports += ', SystemTime'
            if 'UNIX_EPOCH' not in imports:
                imports += ', UNIX_EPOCH'
            return imports + ';'

        content = re.sub(time_import_pattern, add_imports, content)
    else:
        # 添加新的导入语句
        content = re.sub(
            r'(use\s+[^;]+;\s*)*',
            r'\1use std::time::{SystemTime, UNIX_EPOCH};\n',
            content,
            count=1
        )

    # 2. 修复 elapsed() 方法调用问题
    # 将 timestamp.elapsed() 改为使用 Duration::from_secs(timestamp)
    content = re.sub(
        r'(\w+)\.elapsed\(\)',
        r'Duration::from_secs(\1)',
        content
    )

    # 3. 修复 Duration 计算
    # 修复 Duration 算术运算
    content = re.sub(
        r'Duration::from_secs\((\w+)\) - Duration::from_secs\((\w+)\)',
        r'Duration::from_secs(\1 - \2)',
        content
    )

    # 4. 修复 as_secs_f64() 调用（Duration 类型）
    content = re.sub(
        r'Duration::from_secs\((\w+)\)\.as_secs_f64\(\)',
        r'(\1 as f64)',
        content
    )

    # 5. 修复算术运算
    # 修复时间差计算
    content = re.sub(
        r'let\s+(\w+)\s*=\s*Duration::from_secs\((\w+)\)\.as_secs_f64\(\)',
        r'let \1 = \2 as f64',
        content
    )

    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"✅ 修复 {file_path}")
        return True
    else:
        print(f"⏭️  跳过 {file_path}: 无需修复")
        return False

def main():
    """主函数"""
    print("🚀 开始详细修复时间戳相关问题...")

    # 修复有问题的测试文件
    problematic_files = [
        'tests/stage42_metaverse_tests.rs',
        'tests/startup_time_benchmark.rs',
        'tests/stage60_performance_monitoring.rs',
        'tests/performance_benchmark_tests.rs',
        'tests/stage78_phase3_matrix_accelerator_tests.rs',
    ]

    fixed_files = []
    for file_path in problematic_files:
        if os.path.exists(file_path):
            if fix_timestamp_in_file(file_path):
                fixed_files.append(file_path)
        else:
            print(f"⚠️  文件不存在: {file_path}")

    print(f"\n📊 修复完成:")
    print(f"   修复文件数: {len(fixed_files)}")

    if fixed_files:
        print(f"\n✅ 已修复的文件:")
        for file_path in fixed_files:
            print(f"   - {file_path}")

    print("\n✨ 修复完成!")

if __name__ == '__main__':
    main()

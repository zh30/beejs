#!/usr/bin/env python3
"""
修复所有 .elapsed() 调用问题
"""

import os
import re
import glob

def fix_elapsed_in_file(file_path):
    """修复单个文件中的 .elapsed() 调用"""
    with open(file_path, 'r') as f:
        content = f.read()

    original_content = content

    # 1. 确保添加 SystemTime 导入（如果还没有）
    if 'use std::time::' in content and 'SystemTime' not in content:
        # 更新现有的 std::time 导入
        content = re.sub(
            r'use std::time::\{([^}]+)\};',
            lambda m: f'use std::time::{{{m.group(1)}, SystemTime, UNIX_EPOCH}};',
            content
        )
    elif 'use std::time::' not in content:
        # 添加新的导入
        content = 'use std::time::{SystemTime, UNIX_EPOCH, Duration};\n' + content

    # 2. 修复 .elapsed() 调用
    # 找到所有 elapsed() 调用并修复
    # 模式：let elapsed = start.elapsed();
    content = re.sub(
        r'(\w+)\.elapsed\(\)',
        r'\1.elapsed().unwrap()',
        content
    )

    # 3. 修复 SystemTime 创建（如果使用了 as_secs() 然后调用 elapsed）
    content = re.sub(
        r'let (\w+) = std::time::SystemTime::now\(\)\.duration_since\(UNIX_EPOCH\)\.unwrap\(\)\.as_secs\(\);',
        r'let \1 = SystemTime::now();',
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
    print("🚀 开始修复 .elapsed() 调用问题...")

    # 获取所有测试文件
    test_files = glob.glob('tests/*.rs')
    fixed_files = []

    for test_file in test_files:
        if fix_elapsed_in_file(test_file):
            fixed_files.append(test_file)

    print(f"\n📊 修复完成:")
    print(f"   总文件数: {len(test_files)}")
    print(f"   修复文件数: {len(fixed_files)}")

    if fixed_files:
        print(f"\n✅ 已修复的文件:")
        for file_path in fixed_files:
            print(f"   - {file_path}")

    print("\n✨ 修复完成!")

if __name__ == '__main__':
    main()

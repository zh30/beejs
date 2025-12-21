#!/usr/bin/env python3
"""
批量修复测试文件中的时间戳类型
将 Instant::now() 替换为 SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
"""

import os
import re
import glob

def fix_timestamp_in_file(file_path):
    """修复单个文件中的时间戳类型"""
    with open(file_path, 'r') as f:
        content = f.read()

    original_content = content

    # 修复各种 Instant::now() 模式
    patterns = [
        # 标准模式：timestamp: std::time::Instant::now(),
        (r'timestamp:\s*std::time::Instant::now\(\)', 'timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()'),

        # 赋值模式：timestamp: Instant::now(),
        (r'timestamp:\s*Instant::now\(\)', 'timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()'),

        # last_updated 字段
        (r'last_updated:\s*std::time::Instant::now\(\)', 'last_updated: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()'),

        # last_heartbeat 字段
        (r'last_heartbeat:\s*std::time::Instant::now\(\)', 'last_heartbeat: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()'),

        # 初始化模式：let timestamp = Instant::now();
        (r'let\s+\w+\s*=\s*std::time::Instant::now\(\)', lambda m: m.group(0).replace('Instant::now()', 'SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()')),

        # 简单模式：Instant::now()
        (r'Instant::now\(\)', 'SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()'),
    ]

    changes = 0
    for pattern, replacement in patterns:
        if callable(replacement):
            new_content, count = re.subn(pattern, replacement, content)
        else:
            new_content, count = re.subn(pattern, replacement, content)

        if count > 0:
            changes += count
            content = new_content

    if changes > 0:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"✅ 修复 {file_path}: {changes} 处更改")
        return True
    else:
        print(f"⏭️  跳过 {file_path}: 无需修复")
        return False

def main():
    """主函数"""
    print("🚀 开始批量修复测试文件中的时间戳类型...")

    # 获取所有测试文件
    test_files = glob.glob('tests/*.rs')
    fixed_files = []

    for test_file in test_files:
        if fix_timestamp_in_file(test_file):
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

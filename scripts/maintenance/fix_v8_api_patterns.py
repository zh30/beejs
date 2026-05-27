#!/usr/bin/env python3
"""
V8 API 兼容性修复脚本
自动修复 rusty_v8 0.22 -> 0.32 的 API 变更
"""

import re
import os
import sys

def fix_to_array_pattern(content):
    """修复 to_array 模式"""
    # 模式1: if let Some(arr) = value.to_array(scope) {
    pattern1 = r'(\s+)if let Some\(arr\) = (\w+)\.to_array\(scope\) \{'
    replacement1 = r'\1if \2.is_array() {\n\1    if let Ok(arr) = v8::Local::<v8::Array>::try_from(\2) {'

    content = re.sub(pattern1, replacement1, content)

    # 模式2: .and_then(|v| v.to_array(scope))
    pattern2 = r'\.and_then\(\|v\| v\.to_array\(scope\)\)'
    replacement2 = '.and_then(|v| {\n                            if v.is_array() {\n                                v8::Local::<v8::Array>::try_from(v).ok()\n                            } else {\n                                None\n                            }\n                        })'

    content = re.sub(pattern2, replacement2, content)

    return content

def fix_buffer_data_pattern(content):
    """修复 buffer().data() 模式"""
    # 模式: buffer().data()
    pattern = r'(\w+)\.buffer\(\)\.data\(\)'
    replacement = r'\1.backing_store().data()'

    return re.sub(pattern, replacement, content)

def process_file(filepath):
    """处理单个文件"""
    print(f"📝 处理文件: {filepath}")

    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content

    # 修复各种模式
    content = fix_to_array_pattern(content)
    content = fix_buffer_data_pattern(content)

    if content != original_content:
        # 备份原文件
        backup_path = filepath + '.backup'
        with open(backup_path, 'w', encoding='utf-8') as f:
            f.write(original_content)
        print(f"  ✅ 已备份到 {backup_path}")

        # 写入修复后的内容
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  ✅ 已修复")
    else:
        print(f"  ℹ️  无需修复")

def main():
    """主函数"""
    print("🔧 V8 API 兼容性修复工具")
    print("=" * 50)

    # 要处理的文件列表
    files_to_process = [
        'src/nodejs_core/util.rs',
        'src/nodejs_core/url.rs',
    ]

    for filepath in files_to_process:
        if os.path.exists(filepath):
            process_file(filepath)
        else:
            print(f"⚠️  文件不存在: {filepath}")

    print("\n✅ 修复完成！")
    print("请运行 'cargo check' 查看剩余错误")

if __name__ == '__main__':
    main()

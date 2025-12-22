#!/usr/bin/env python3
"""
修复剩余的语法解析错误
"""

import re
from pathlib import Path

def fix_generic_syntax_errors(content):
    """修复泛型语法错误"""
    # 修复类似 Vec<_>> 的错误
    patterns = [
        (r'Vec<[^>]*>>', r'Vec<_>'),
        (r'HashMap<[^>]*>>', r'HashMap<String, _>'),
        (r'BTreeMap<[^>]*>>', r'BTreeMap<String, _>'),
    ]

    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)

    return content

def fix_mod_declaration_errors(content):
    """修复模块声明错误"""
    # 修复 mod {} 块内的语法
    lines = content.split('\n')
    fixed_lines = []
    in_mod_block = False
    brace_count = 0

    for line in lines:
        if 'mod {' in line and 'pub mod' not in line:
            in_mod_block = True
            brace_count = 0

        if in_mod_block:
            # 跳过空的 use 语句
            if line.strip().startswith('use std::'):
                continue
            # 修复不正确的语法
            if 'use std::' in line and not line.strip().endswith(';'):
                line = line.rstrip(',') + ';'
            if 'pub use std::' in line and not line.strip().endswith(';'):
                line = line.rstrip(',') + ';'

        fixed_lines.append(line)

        # 统计大括号
        brace_count += line.count('{') - line.count('}')
        if brace_count == 0 and in_mod_block:
            in_mod_block = False

    return '\n'.join(fixed_lines)

def fix_bracket_mismatch(content):
    """修复括号不匹配"""
    # 简单的括号匹配修复
    lines = content.split('\n')
    fixed_lines = []
    for line in lines:
        # 修复不匹配的括号
        line = line.replace('{,', '{')
        line = line.replace(',}', '}')
        # 修复多余的分号
        if 'pub mod {' in line and line.strip().endswith(';'):
            line = line.rstrip(';')
        fixed_lines.append(line)

    return '\n'.join(fixed_lines)

def process_file(filepath):
    """处理单个文件"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content

        # 应用修复
        content = fix_generic_syntax_errors(content)
        content = fix_mod_declaration_errors(content)
        content = fix_bracket_mismatch(content)

        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ 修复语法错误: {filepath}")
            return True

    except Exception as e:
        print(f"❌ 处理失败 {filepath}: {e}")

    return False

def main():
    """主函数"""
    print("🔧 修复剩余语法错误...")

    # 修复关键文件
    key_files = [
        "src/ai/model_interface.rs",
        "src/automation/threshold.rs",
        "src/analysis/mod.rs",
        "src/analysis/optimizer.rs",
        "src/monitor/mod.rs",
        "src/monitor/performance_monitor.rs",
        "src/monitor/data_store.rs",
    ]

    fixed_count = 0
    for file_path in key_files:
        if Path(file_path).exists():
            if process_file(file_path):
                fixed_count += 1

    print(f"\n🎉 语法修复完成! 共修复 {fixed_count} 个文件")

if __name__ == "__main__":
    main()

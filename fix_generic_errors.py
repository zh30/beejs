#!/usr/bin/env python3
"""
修复泛型参数错误
"""

import re
from pathlib import Path

def fix_generic_errors(file_path: Path) -> bool:
    """修复泛型参数错误"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content
        modifications = []

        # 修复模式1: Result<()> -> Result<(), Box<dyn std::error::Error>>
        result_pattern = r'-> Result\(\) \{'
        if re.search(result_pattern, content):
            content = re.sub(result_pattern, '-> Result<(), Box<dyn std::error::Error>> {', content)
            modifications.append("Fixed Result<> generic parameter")

        # 修复模式2: BTreeMap 泛型参数错误
        # 匹配过多的泛型参数
        btreemap_pattern = r'BTreeMap<String,\s*V8Snapshot(?:,\s*String,\s*V8Snapshot)+\s*>>'
        if re.search(btreemap_pattern, content):
            # 简化为 BTreeMap<String, V8Snapshot>
            content = re.sub(
                r'BTreeMap<String,\s*V8Snapshot(?:,\s*String,\s*V8Snapshot)*\s*>>',
                'BTreeMap<String, V8Snapshot>>',
                content
            )
            modifications.append("Fixed BTreeMap generic parameters")

        if content != original:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed {file_path}:")
            for mod in modifications:
                print(f"  - {mod}")
            return True
        return False

    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    print("🔧 修复泛型参数错误...")
    print("=" * 60)

    src_dir = Path("src")
    fixed_count = 0

    for file_path in src_dir.rglob("*.rs"):
        if fix_generic_errors(file_path):
            fixed_count += 1

    print(f"\n✅ 修复了 {fixed_count} 个文件的泛型错误")

if __name__ == "__main__":
    main()

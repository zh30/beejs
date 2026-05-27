#!/usr/bin/env python3
"""批量修复所有文件中的 is_array(scope) 调用"""

import subprocess
import re

# 查找所有包含 is_array(scope) 的源文件
result = subprocess.run(
    ['find', 'src', '-name', '*.rs', '-type', 'f', '-exec', 'grep', '-l', 'is_array(scope)', '{}', ';'],
    capture_output=True,
    text=True
)

files = result.stdout.strip().split('\n') if result.stdout.strip() else []
print(f"找到 {len(files)} 个文件包含 is_array(scope) 调用")

for file_path in files:
    if not file_path:
        continue

    print(f"处理文件: {file_path}")

    # 读取文件
    with open(file_path, 'r') as f:
        content = f.read()

    # 替换 is_array(scope) 为 is_array()
    original_content = content
    content = re.sub(r'\.is_array\(scope\)', '.is_array()', content)

    # 检查是否有变化
    if content != original_content:
        # 写回文件
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"  ✅ 已修复")
    else:
        print(f"  ℹ️  无需修复")

print("\n✅ 完成所有 is_array(scope) 修复")
#!/usr/bin/env python3
"""批量修复 usize: Neg trait bound 错误"""

import re

# 读取文件
with open('src/nodejs_core/buffer.rs', 'r') as f:
    content = f.read()

print("开始修复 usize: Neg trait bound 错误...")

# 修复 1: 将 end 的类型从 usize 改为 isize
content = re.sub(
    r'(\.get\(2\)\s*\n\s*\.to_integer\(scope\)\s*\n\s*\.unwrap_or\(v8::Integer::new\(scope, -1\)\)\s*\n\s*\.value\(\)) as usize',
    r'\1 as isize',
    content
)

# 修复 2: 更新 actual_end 的计算逻辑
content = re.sub(
    r'let actual_end = if end == -1 \{ buffer_length \} else \{ end\.min\(buffer_length) \}',
    'let actual_end = if end == -1 { buffer_length } else { end.min(buffer_length as isize) as usize }',
    content
)

# 写回文件
with open('src/nodejs_core/buffer.rs', 'w') as f:
    f.write(content)

print("✅ 已修复 usize: Neg trait bound 错误")
print("   - 将 end 变量类型从 usize 改为 isize")
print("   - 更新 actual_end 计算逻辑以正确处理 isize")
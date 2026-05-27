#!/usr/bin/env python3
"""
简单安全的 url.rs 修复脚本
一次只替换一个模式
"""

import re

# 读取文件
with open('src/nodejs_core/url.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print(f"📝 共有 {len(lines)} 行")

# 修复计数器
fixed_count = 0

# 新的修复后的行
new_lines = []

i = 0
while i < len(lines):
    line = lines[i]

    # 检查是否是外层的 to_array
    # 模式: if let Some(arr) = params_array.to_array(scope) {
    match = re.match(r'(\s+)if let Some\(arr\) = (\w+)\.to_array\(scope\) \{', line)
    if match:
        indent = match.group(1)
        var_name = match.group(2)

        # 替换这一行
        new_lines.append(f'{indent}if {var_name}.is_array() {{\n')
        new_lines.append(f'{indent}    if let Ok(arr) = v8::Local::<v8::Array>::try_from({var_name}) {{\n')
        fixed_count += 1
        print(f"  ✅ 修复第 {i+1} 行: 外层 to_array")
        i += 1
        continue

    # 检查是否是内层的 to_array 在 and_then 中
    # 模式: .and_then(|v| v.to_array(scope))
    if '.and_then(|v| v.to_array(scope))' in line:
        # 替换为复杂的逻辑
        indent = re.search(r'^(\s+)', line).group(1)
        new_lines.append(f'{indent}.and_then(|v| {{\n')
        new_lines.append(f'{indent}    if v.is_array() {{\n')
        new_lines.append(f'{indent}        v8::Local::<v8::Array>::try_from(v).ok()\n')
        new_lines.append(f'{indent}    }} else {{\n')
        new_lines.append(f'{indent}        None\n')
        new_lines.append(f'{indent}    }}\n')
        new_lines.append(f'{indent}}})\n')
        fixed_count += 1
        print(f"  ✅ 修复第 {i+1} 行: 内层 to_array")
        i += 1
        continue

    # 普通行
    new_lines.append(line)
    i += 1

print(f"\n✅ 总共修复了 {fixed_count} 个模式")

# 备份并写入
with open('src/nodejs_core/url.rs.backup2', 'w', encoding='utf-8') as f:
    f.writelines(lines)

with open('src/nodejs_core/url.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print("✅ 修复完成！")

#!/usr/bin/env python3
"""
修复 url.rs 中的复杂 to_array 嵌套模式
"""

import re

def fix_url_rs_to_array(content):
    """修复 url.rs 中复杂的 to_array 模式"""

    # 模式1: 外层 to_array
    # 从: if let Some(arr) = params_array.to_array(scope) {
    # 到: if params_array.is_array() {
    #         if let Ok(arr) = v8::Local::<v8::Array>::try_from(params_array) {
    pattern1 = r'(\s+)if let Some\(arr\) = (\w+)\.to_array\(scope\) \{'
    replacement1 = r'\1if \2.is_array() {\n\1    if let Ok(arr) = v8::Local::<v8::Array>::try_from(\2) {'
    content = re.sub(pattern1, replacement1, content)

    # 模式2: 内层嵌套的 to_array 在 and_then 中
    # 从: .and_then(|v| v.to_array(scope))
    # 到: 复杂的嵌套逻辑
    # 这需要更仔细的处理

    # 模式3: 简单的直接 to_array
    pattern3 = r'(\s+)if let Some\((\w+)\) = (\w+)\.to_array\(scope\) \{'
    replacement3 = r'\1if \3.is_array() {\n\1    if let Ok(\2) = v8::Local::<v8::Array>::try_from(\3) {'
    content = re.sub(pattern3, replacement3, content)

    # 模式4: 处理 .and_then 中的 to_array
    # 这里需要更复杂的逻辑，因为涉及到链式调用
    # 我们先跳过这些，标记出来手动处理
    lines = content.split('\n')
    new_lines = []
    in_and_then = False

    for i, line in enumerate(lines):
        # 标记需要手动处理的行
        if '.and_then(|v| v.to_array(scope))' in line:
            # 在这行后面添加注释标记
            new_lines.append(line)
            new_lines.append('                            // TODO: 手动修复此行的 to_array 调用')
        else:
            new_lines.append(line)

    return '\n'.join(new_lines)

# 读取文件
with open('src/nodejs_core/url.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 备份原文件
with open('src/nodejs_core/url.rs.backup', 'w', encoding='utf-8') as f:
    f.write(content)

# 修复内容
fixed_content = fix_url_rs_to_array(content)

# 写入修复后的内容
with open('src/nodejs_core/url.rs', 'w', encoding='utf-8') as f:
    f.write(fixed_content)

print("✅ url.rs 修复完成")
print("请检查并手动修复标记为 TODO 的行")

#!/usr/bin/env python3
"""
修复 stream.rs 中的 FunctionCallbackArguments 和 ReturnValue 错误
"""

import re

# 读取文件
with open('src/nodejs_core/stream.rs', 'r', encoding='utf-8') as f:
    original_content = f.read()
    content = original_content

print("🔧 开始修复 stream.rs...")

# 统计修复数量
fixes = 0

# 修复 1: ReturnValue::new() - 在回调函数中，应该使用参数传递的 retval
# 模式: let mut cb_retval = v8::ReturnValue::new();
pattern1 = r'(\s+)let mut cb_retval = v8::ReturnValue::new\(\);'
replacement1 = r'\1// 使用函数签名中的 retval 参数，不需要创建新的'
content = re.sub(pattern1, replacement1, content)
fixes += len(re.findall(pattern1, content))

# 修复 2: FunctionCallbackArguments::from_function_args - 新版本不需要这个
# 模式: let cb_args = v8::FunctionCallbackArguments::from_function_args(scope, &[data_value]);
# 应该直接使用参数中的 args
pattern2 = r'(\s+)let cb_args = v8::FunctionCallbackArguments::from_function_args\(scope, &\[([^\]]+)\]\);'
replacement2 = r'\1// 直接使用参数中的 args，不需要创建新的'
content = re.sub(pattern2, replacement2, content)
fixes += len(re.findall(pattern2, content))

# 修复 3: 空的 FunctionCallbackArguments
# 模式: let cb_args = v8::FunctionCallbackArguments::from_function_args(scope, &[]);
pattern3 = r'(\s+)let cb_args = v8::FunctionCallbackArguments::from_function_args\(scope, &\[\]\);'
replacement3 = r'\1// 直接使用参数中的 args，不需要创建新的'
content = re.sub(pattern3, replacement3, content)
fixes += len(re.findall(pattern3, content))

print(f"✅ 修复了 {fixes} 个模式")

# 备份并写入
with open('src/nodejs_core/stream.rs.backup', 'w', encoding='utf-8') as f:
    f.write(original_content)

with open('src/nodejs_core/stream.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("✅ stream.rs 修复完成！")

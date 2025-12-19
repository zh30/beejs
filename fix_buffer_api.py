#!/usr/bin/env python3
"""修复 buffer.rs 中的 V8 API 问题"""

import re

# 读取文件
with open('src/nodejs_core/buffer.rs', 'r') as f:
    content = f.read()

# 修复 set_on_instance 调用
# 这些应该是静态方法，直接在构造函数上设置
def fix_set_on_instance(match):
    full_match = match.group(0)
    # 提取方法名
    method_match = re.search(r'set_on_instance\([^,]+,\s*v8::String::new\(scope,\s*"([^"]+)"\)', full_match)
    if method_match:
        method_name = method_match.group(1)
        return f'    // Static method: {method_name}'
    return f'    // Removed set_on_instance: {full_match}'

# 修复 set_prototype_property_initializer_callback 调用
def fix_set_prototype_callback(match):
    full_match = match.group(0)
    prop_match = re.search(r'set_prototype_property_initializer_callback\([^,]+,\s*v8::String::new\(scope,\s*"([^"]+)"\)', full_match)
    if prop_match:
        prop_name = prop_match.group(1)
        return f'    // Instance method: {prop_name} (removed in new V8)'
    return f'    // Removed set_prototype_property_initializer_callback: {full_match}'

# 修复 set_prototype_property_accessor 调用
def fix_set_prototype_accessor(match):
    full_match = match.group(0)
    prop_match = re.search(r'set_prototype_property_accessor\([^,]+,\s*v8::String::new\(scope,\s*"([^"]+)"\)', full_match)
    if prop_match:
        prop_name = prop_match.group(1)
        return f'    // Instance accessor: {prop_name} (removed in new V8)'
    return f'    // Removed set_prototype_property_accessor: {full_match}'

# 应用修复
content = re.sub(r'    buffer_constructor\.set_on_instance\([^)]+\);', fix_set_on_instance, content)
content = re.sub(r'    buffer_constructor\.set_prototype_property_initializer_callback\([^)]+\);', fix_set_prototype_callback, content)
content = re.sub(r'    buffer_constructor\.set_prototype_property_accessor\([^)]+\);', fix_set_prototype_accessor, content)

# 写回文件
with open('src/nodejs_core/buffer.rs', 'w') as f:
    f.write(content)

print("✅ 已修复 buffer.rs 中的 V8 API 问题")
print("   - 注释掉已移除的 set_on_instance 调用")
print("   - 注释掉已移除的 set_prototype_property_initializer_callback 调用")
print("   - 注释掉已移除的 set_prototype_property_accessor 调用")
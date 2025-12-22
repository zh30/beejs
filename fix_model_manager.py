#!/usr/bin/env python3
"""
修复 src/ai/model_manager.rs 中的泛型嵌套错误
"""

import re

def fix_model_manager():
    file_path = "src/ai/model_manager.rs"

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    print("修复前文件大小:", len(content), "字符")

    # 修复模式: 多余的括号和泛型嵌套
    # 1. 修复 Arc<RwLock<HashMap<...>>>>>> 为 Arc<RwLock<HashMap<...>>>
    pattern1 = r'Arc<RwLock<HashMap<([^,]+),\s*([^>]+)>>>>>>'
    def replace_arc_rwlock_hashmap(match):
        key_type = match.group(1).strip()
        value_type = match.group(2).strip()
        return f'Arc<RwLock<HashMap<{key_type}, {value_type}>>>,'
    content = re.sub(pattern1, replace_arc_rwlock_hashmap, content)
    print("✓ 修复 Arc<RwLock<HashMap>> 多余括号")

    # 2. 修复 Arc<RwLock<HashMap<...>>>>>> (无逗号结尾)
    pattern2 = r'Arc<RwLock<HashMap<([^,]+),\s*([^>]+)>>>>>'
    def replace_arc_rwlock_hashmap_no_comma(match):
        key_type = match.group(1).strip()
        value_type = match.group(2).strip()
        return f'Arc<RwLock<HashMap<{key_type}, {value_type}>>>'
    content = re.sub(pattern2, replace_arc_rwlock_hashmap_no_comma, content)
    print("✓ 修复 Arc<RwLock<HashMap>> 多余括号 (无逗号)")

    # 3. 修复复杂的泛型嵌套 HashMap<..., std::collections::HashMap<...>>
    pattern3 = r'HashMap<([^,]+),\s*([^,]+),\s*std::collections::HashMap<[^>]+>>'
    def replace_hashmap(match):
        key_type = match.group(1).strip()
        value_type = match.group(2).strip()
        return f'HashMap<{key_type}, {value_type}>'
    content = re.sub(pattern3, replace_hashmap, content)
    print("✓ 修复 HashMap 泛型嵌套")

    # 4. 修复超长的嵌套 HashMap (简单匹配)
    # 匹配类似: HashMap<String, (String, Instant), std::collections::HashMap<...>
    pattern4 = r'HashMap<String,\s*\([^)]+\),\s*std::collections::HashMap<[^>]+>>'
    def replace_complex_hashmap(match):
        return 'HashMap<String, (String, Instant)>'
    content = re.sub(pattern4, replace_complex_hashmap, content)
    print("✓ 修复复杂的 HashMap 嵌套")

    # 写回文件
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

    print("修复后文件大小:", len(content), "字符")
    print("\n✅ model_manager.rs 修复完成!")

if __name__ == "__main__":
    fix_model_manager()

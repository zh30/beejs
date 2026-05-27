#!/usr/bin/env python3
"""
修复 src/ai/predictive_scaler.rs 中的泛型嵌套错误
"""

import re

def fix_predictive_scaler():
    file_path = "src/ai/predictive_scaler.rs"

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    print("修复前文件大小:", len(content), "字符")

    # 修复1: resource_utilization 字段的泛型嵌套
    # 从: HashMap<String, f64, std::collections::HashMap<...>>
    # 到: HashMap<String, f64>
    old_pattern1 = r'pub resource_utilization: HashMap<String, f64, std::collections::HashMap<String, f64, String, f64, std::collections::HashMap<String, f64, std::collections::HashMap<String, f64, String, f64, String, f64, std::collections::HashMap<String, f64, String, f64>>>>'
    new_pattern1 = 'pub resource_utilization: HashMap<String, f64>,'
    content = re.sub(old_pattern1, new_pattern1, content)
    print("✓ 修复 resource_utilization 泛型嵌套")

    # 修复2: historical_data 字段缺少闭合括号
    old_pattern2 = r'historical_data: Arc<RwLock<Vec<Metrics>>,'
    new_pattern2 = 'historical_data: Arc<RwLock<Vec<Metrics>>>,'
    content = re.sub(old_pattern2, new_pattern2, content)
    print("✓ 修复 historical_data 括号不匹配")

    # 修复3: patterns 字段的泛型嵌套
    old_pattern3 = r'patterns: Arc<RwLock<HashMap<String, SeasonalityPattern, std::collections::HashMap<String, SeasonalityPattern, String, SeasonalityPattern, std::collections::HashMap<String, SeasonalityPattern, std::collections::HashMap<String, SeasonalityPattern, String, SeasonalityPattern, String, SeasonalityPattern, std::collections::HashMap<String, SeasonalityPattern, String, SeasonalityPattern>>>>>'
    new_pattern3 = 'patterns: Arc<RwLock<HashMap<String, SeasonalityPattern>>>,'
    content = re.sub(old_pattern3, new_pattern3, content)
    print("✓ 修复 patterns 泛型嵌套")

    # 修复4: scaling_history 字段缺少闭合括号
    old_pattern4 = r'scaling_history: Arc<RwLock<Vec<ScalingAction>>,'
    new_pattern4 = 'scaling_history: Arc<RwLock<Vec<ScalingAction>>>,'
    content = re.sub(old_pattern4, new_pattern4, content)
    print("✓ 修复 scaling_history 括号不匹配")

    # 修复5: parameters 字段的泛型嵌套
    old_pattern5 = r'parameters: HashMap<String, f64, std::collections::HashMap<String, f64, String, f64, std::collections::HashMap<String, f64, std::collections::HashMap<String, f64, String, f64, String, f64, std::collections::HashMap<String, f64, String, f64>>>>'
    new_pattern5 = 'parameters: HashMap<String, f64>,'
    content = re.sub(old_pattern5, new_pattern5, content)
    print("✓ 修复 parameters 泛型嵌套")

    # 写回文件
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

    print("修复后文件大小:", len(content), "字符")
    print("\n✅ predictive_scaler.rs 修复完成!")

if __name__ == "__main__":
    fix_predictive_scaler()

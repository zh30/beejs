#!/usr/bin/env python3
"""
精确修复 use 语句错误
"""

import re
from pathlib import Path

def fix_file(file_path):
    """修复单个文件"""
    try:
        with open(file_path, 'r') as f:
            content = f.read()

        original = content

        # 修复特定错误
        fixes = [
            (r'use serde\{', 'use serde::{'),
            (r'use std::time\{', 'use std::time::{'),
            (r'use std::collections\{', 'use std::collections::{'),
            (r'use crate::benchmarks\{', 'use crate::benchmarks::{'),
        ]

        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content)

        # 修复 module:: 形式的错误
        content = re.sub(r'use ([a-z_]+)::\{', r'use \1::{', content)

        if content != original:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True

        return False

    except Exception as e:
        print(f"Error {file_path}: {e}")
        return False

# 修复错误文件
error_files = [
    'src/performance_analyzer.rs',
    'src/performance_regression.rs',
    'src/performance_comparison/mod.rs',
    'src/automation/mod.rs',
    'src/analysis/mod.rs',
]

for file_path in error_files:
    if Path(file_path).exists():
        fix_file(file_path)

#!/usr/bin/env python3
"""
Stage 46: Fix v8::String::new() Option<Local> 类型转换问题

问题: v8::String::new(scope, ...) 返回 Option<Local<String>>
     但代码中调用 .into() 时没有先 unwrap

模式:
  let val_xxx = v8::String::new(scope, ...);
  obj.set(scope, key.into(), val_xxx.into());  // 错误！

修复为:
  let val_xxx = v8::String::new(scope, ...).unwrap();
  obj.set(scope, key.into(), val_xxx.into());  // 正确
"""

import re
import sys
from pathlib import Path

def fix_v8_string_new(content: str) -> str:
    """修复 v8::String::new() 调用，确保有 .unwrap()"""

    # 模式 1: let val_xxx = v8::String::new(scope, "..."); (没有 unwrap)
    # 修复: 添加 .unwrap()
    pattern1 = r'(let\s+(?:val_)?[\w]+\s*=\s*v8::String::new\([^)]+\))(\s*;)'

    def fix_string_new(match):
        stmt = match.group(1)
        ending = match.group(2)
        # 如果已经有 unwrap，不处理
        if '.unwrap()' in stmt or '.unwrap_or' in stmt:
            return match.group(0)
        return stmt + '.unwrap()' + ending

    content = re.sub(pattern1, fix_string_new, content)

    # 模式 2: v8::String::new(scope, ...).into() 直接调用 (少见)
    # 这种需要变成 v8::String::new(scope, ...).unwrap().into()
    # 但这比较复杂，先忽略

    return content

def fix_v8_integer_new(content: str) -> str:
    """修复 v8::Integer::new() 调用"""
    # v8::Integer::new() 也返回 Option 在某些版本
    # 但实际上在新版本可能不返回 Option
    # 先检查具体错误再修
    return content

def process_file(filepath: Path) -> tuple[bool, int]:
    """处理单个文件，返回 (是否修改, 修改次数)"""
    original = filepath.read_text()
    modified = fix_v8_string_new(original)
    modified = fix_v8_integer_new(modified)

    if modified != original:
        filepath.write_text(modified)
        changes = original.count('v8::String::new') - modified.count('v8::String::new(') + \
                  modified.count('.unwrap()') - original.count('.unwrap()')
        return True, max(1, changes)
    return False, 0

def main():
    nodejs_core = Path('src/nodejs_core')
    if not nodejs_core.exists():
        print("Error: src/nodejs_core not found")
        sys.exit(1)

    total_files = 0
    total_changes = 0

    files_to_fix = [
        'url.rs', 'child_process.rs', 'os.rs', 'buffer.rs',
        'util.rs', 'path.rs', 'querystring.rs', 'stream.rs',
        'events.rs', 'net.rs', 'http.rs', 'crypto.rs', 'fs.rs'
    ]

    for filename in files_to_fix:
        filepath = nodejs_core / filename
        if filepath.exists():
            modified, count = process_file(filepath)
            if modified:
                total_files += 1
                total_changes += count
                print(f"  ✅ {filename}: ~{count} 处修复")

    print(f"\n总计: {total_files} 个文件, ~{total_changes} 处修改")

if __name__ == '__main__':
    main()

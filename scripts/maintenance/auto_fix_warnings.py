#!/usr/bin/env python3
"""
自动修复编译警告 - 批量删除未使用的导入
"""
import re
import subprocess
from pathlib import Path

def get_warning_files():
    """获取有警告的文件"""
    result = subprocess.run(
        ['cargo', 'build', '--lib', '2>&1'],
        capture_output=True,
        text=True
    )

    files_with_warnings = set()

    for line in result.stderr.split('\n'):
        if '-->' in line and '.rs:' in line:
            file_path = line.split('.rs:')[0] + '.rs'
            file_path = file_path.replace('--> ', '').strip()
            files_with_warnings.add(file_path)

    return files_with_warnings

def clean_file_warnings(file_path):
    """清理单个文件的警告"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except:
        return False

    original = content
    lines = content.split('\n')

    # 查找并删除未使用的导入
    cleaned_lines = []
    i = 0
    modified = False

    while i < len(lines):
        line = lines[i]

        # 检查是否是 use 语句
        if line.strip().startswith('use '):
            # 检查下一行是否也是导入（多行导入）
            use_line = line
            j = i + 1

            # 收集多行导入
            while j < len(lines) and (lines[j].strip().startswith('use ') or lines[j].strip().endswith('{')):
                if lines[j].strip().endswith('{'):
                    # 这是多行导入的开始
                    use_line += '\n' + lines[j]
                    j += 1
                    # 收集导入项
                    while j < len(lines):
                        import_line = lines[j]
                        use_line += '\n' + import_line
                        if '};' in import_line or import_line.strip().endswith(';'):
                            j += 1
                            break
                        j += 1
                    break
                else:
                    use_line += '\n' + lines[j]
                    j += 1

            # 检查这个导入是否在文件中被使用
            # 简单检查：提取导入的名称并搜索
            import_items = re.findall(r'(\w+)(?:,|\s*::|\s*as|\s*\{|$)', use_line)
            used = False

            for item in import_items[:5]:  # 检查前5个
                if len(item) > 2 and item not in ['std', 'crate', 'super']:
                    # 在剩余内容中搜索
                    remaining = '\n'.join(lines[j:])
                    if item in remaining:
                        used = True
                        break

            if not used:
                # 删除这个导入
                print(f"  🗑️  删除未使用导入: {file_path}:{i+1}")
                modified = True
                i = j
                continue

            cleaned_lines.append(use_line)
            i = j
        else:
            cleaned_lines.append(line)
            i += 1

    new_content = '\n'.join(cleaned_lines)

    if new_content != original:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True

    return False

def main():
    """主函数"""
    print("🔧 开始自动清理编译警告...\n")

    files = get_warning_files()
    print(f"📁 发现 {len(files)} 个文件有警告\n")

    modified_count = 0
    for file_path in sorted(files):
        if 'src/' not in file_path:
            continue

        print(f"处理: {file_path}")
        if clean_file_warnings(file_path):
            modified_count += 1

    print(f"\n✅ 完成! 修改了 {modified_count} 个文件")

if __name__ == '__main__':
    main()

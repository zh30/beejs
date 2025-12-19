#!/bin/bash
# 批量修复 to_array 错误脚本

echo "🔧 开始批量修复 to_array 错误..."
echo "=================================="

# 文件列表
FILES=(
    "src/nodejs_core/util.rs"
    "src/nodejs_core/url.rs"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "📝 处理文件: $file"

        # 统计 to_array 错误数量
        count=$(grep -c "to_array" "$file" || echo "0")
        echo "  发现 $count 个 to_array 错误"

        if [ "$count" -gt 0 ]; then
            # 备份文件
            cp "$file" "$file.backup"
            echo "  已备份到 $file.backup"
        fi
    fi
done

echo ""
echo "✅ 批量修复准备完成"
echo "📋 接下来需要手动修复以下文件中的 to_array 错误:"
echo "  - src/nodejs_core/util.rs"
echo "  - src/nodejs_core/url.rs"
echo ""
echo "修复模式:"
echo "  旧: if let Some(arr) = value.to_array(scope) {"
echo "  新: if value.is_array() {"
echo "          if let Ok(arr) = v8::Local::<v8::Array>::try_from(value) {"

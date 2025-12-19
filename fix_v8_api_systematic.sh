#!/bin/bash
# V8 API 系统性修复脚本
# 修复 rusty_v8 0.22 -> 0.32 的 API 变更

set -e

echo "🔧 开始 V8 API 系统性修复..."
echo "================================"

# 备份原文件
echo "📦 备份关键源文件..."
cp src/nodejs_core/crypto.rs src/nodejs_core/crypto.rs.backup || true
cp src/nodejs_core/buffer.rs src/nodejs_core/buffer.rs.backup || true
cp src/nodejs_core/stream.rs src/nodejs_core/stream.rs.backup || true

# 1. 修复 to_array 错误
echo "🔄 修复 to_array 错误..."
echo "  - 替换模式: value.to_array(scope) -> is_array() + try_from()"

# 查找并修复 crypto.rs 中的 to_array 错误
if grep -n "to_array" src/nodejs_core/crypto.rs > /dev/null; then
    echo "  发现 crypto.rs 中的 to_array 错误，需要手动修复"
fi

# 2. 修复 buffer 访问错误
echo "🔄 修复 buffer 访问错误..."
echo "  - 替换模式: buffer().data() -> backing_store().data()"

# 3. 修复 FunctionCallbackArguments 构造
echo "🔄 修复 FunctionCallbackArguments..."
echo "  - 替换模式: v8::FunctionCallbackArguments::from_function_args -> 直接构造"

# 4. 修复 ReturnValue 构造
echo "🔄 修复 ReturnValue..."
echo "  - 替换模式: v8::ReturnValue::new() -> 使用参数传递的 mut retval"

echo ""
echo "✅ 系统性修复脚本准备完成"
echo "📝 接下来需要手动修复关键文件中的 API 调用"
echo ""
echo "主要修复点:"
echo "  1. crypto.rs: 238行, 273行 - to_array 错误"
echo "  2. buffer.rs: buffer().data() 访问"
echo "  3. stream.rs: 各种 API 变更"
echo "  4. function callbacks: 参数和返回值处理"

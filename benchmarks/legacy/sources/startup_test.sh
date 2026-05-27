#!/bin/bash
# 启动时间基准测试

echo "=== Beejs 启动时间优化验证 ==="
echo ""

echo "测试 1: 数字字面量 (超级快路径)"
for i in {1..3}; do
    echo -n "  第 $i 次: "
    time ./target/release/beejs -e '123' 2>&1 | grep total
done
echo ""

echo "测试 2: 布尔值 (超级快路径)"
for i in {1..3}; do
    echo -n "  第 $i 次: "
    time ./target/release/beejs -e 'true' 2>&1 | grep total
done
echo ""

echo "测试 3: 字符串字面量 (超级快路径)"
for i in {1..3}; do
    echo -n "  第 $i 次: "
    time ./target/release/beejs -e '"hello"' 2>&1 | grep total
done
echo ""

echo "测试 4: 简单算术 (超级快路径)"
for i in {1..3}; do
    echo -n "  第 $i 次: "
    time ./target/release/beejs -e '2+2' 2>&1 | grep total
done
echo ""

echo "测试 5: 复杂脚本 (需要 V8)"
time ./target/release/beejs --eval 'console.log("Hello World"); 1+1' 2>&1 | grep total
echo ""

echo "测试 6: 简单脚本 (需要 V8)"
time ./target/release/beejs --eval 'let x = 1; let y = 2; x + y' 2>&1 | grep total
echo ""

echo "✅ 启动时间测试完成！"

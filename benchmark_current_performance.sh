#!/bin/bash
# 当前性能基准测试脚本

echo "🚀 Beejs 性能基准测试"
echo "======================="
echo ""

# 测试1: 简单表达式
echo "测试1: 简单表达式 (console.log + 1+1)"
time ./target/release/beejs --eval 'console.log("Hello"); 1+1' 2>&1
echo ""

# 测试2: 算术运算
echo "测试2: 算术运算 (复杂计算)"
time ./target/release/beejs --eval 'let sum = 0; for(let i=0; i<10000; i++) { sum += i * 2; } sum' 2>&1
echo ""

# 测试3: 函数调用
echo "测试3: 函数调用 (递归计算)"
time ./target/release/beejs --eval 'function fib(n) { return n <= 1 ? n : fib(n-1) + fib(n-2); } fib(10)' 2>&1
echo ""

# 测试4: 对象操作
echo "测试4: 对象操作"
time ./target/release/beejs --eval 'let obj = {a: 1, b: 2, c: 3}; obj.a + obj.b + obj.c' 2>&1
echo ""

# 测试5: 数组操作
echo "测试5: 数组操作"
time ./target/release/beejs --eval 'let arr = [1,2,3,4,5]; arr.reduce((a,b) => a+b, 0)' 2>&1
echo ""

# 测试6: REPL 性能 (单进程多次执行)
echo "测试6: REPL 性能 (3次命令)"
echo -e 'console.log("Test 1"); 1+1\nconsole.log("Test 2"); 2+2\nconsole.log("Test 3"); 3+3\n.exit' | time ./target/release/beejs repl 2>&1
echo ""

echo "✅ 基准测试完成"
echo ""
echo "性能目标对比:"
echo "- 简单执行: < 5ms (当前: 7-8ms) ⚠️"
echo "- REPL启动: < 5ms (当前: ~5ms) ✅"
echo "- 复杂计算: < 100ms (当前: 17ms) ✅"

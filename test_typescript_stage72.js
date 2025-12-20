#!/usr/bin/env node

// Stage 72 TypeScript 支持验证脚本
// 测试箭头函数和类型标注转译功能

const { spawn } = require('child_process');
const path = require('path');

console.log('🚀 Beejs Stage 72 TypeScript 支持验证\n');

// 测试用例
const testCases = [
    {
        name: '简单箭头函数',
        code: 'const double = (x: number) => x * 2; console.log(double(5));',
        expected: '10'
    },
    {
        name: '多参数箭头函数',
        code: 'const add = (a: number, b: number): number => a + b; console.log(add(10, 20));',
        expected: '30'
    },
    {
        name: '无参数箭头函数',
        code: 'const getAnswer = () => 42; console.log(getAnswer());',
        expected: '42'
    },
    {
        name: '类型标注函数',
        code: 'function greet(name: string): string { return `Hello, ${name}!`; } console.log(greet("Beejs"));',
        expected: 'Hello, Beejs!'
    }
];

let passed = 0;
let failed = 0;

async function runTest(testCase) {
    return new Promise((resolve) => {
        console.log(`📝 测试: ${testCase.name}`);

        // 创建临时文件
        const fs = require('fs');
        const tmpFile = path.join(__dirname, `test_temp_${Date.now()}.ts`);
        fs.writeFileSync(tmpFile, testCase.code);

        // 运行 beejs
        const beejs = spawn('./beejs', [tmpFile, '--verbose'], {
            cwd: __dirname
        });

        let output = '';
        let errorOutput = '';

        beejs.stdout.on('data', (data) => {
            output += data.toString();
        });

        beejs.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });

        beejs.on('close', (code) => {
            // 清理临时文件
            fs.unlinkSync(tmpFile);

            if (code === 0 && output.includes(testCase.expected)) {
                console.log(`✅ 通过 - 输出: ${output.trim()}\n`);
                passed++;
            } else {
                console.log(`❌ 失败`);
                console.log(`   期望: ${testCase.expected}`);
                console.log(`   实际输出: ${output.trim()}`);
                if (errorOutput) {
                    console.log(`   错误: ${errorOutput.trim()}`);
                }
                console.log('');
                failed++;
            }

            resolve();
        });
    });
}

async function runAllTests() {
    for (const testCase of testCases) {
        await runTest(testCase);
    }

    console.log('='.repeat(50));
    console.log(`📊 测试结果: ${passed} 通过, ${failed} 失败`);
    console.log('='.repeat(50));

    if (failed === 0) {
        console.log('\n🎉 Stage 72 TypeScript 支持验证成功！');
        console.log('✨ 所有箭头函数和类型标注功能正常工作');
    } else {
        console.log('\n⚠️  部分测试失败，需要进一步调试');
    }

    process.exit(failed === 0 ? 0 : 1);
}

runAllTests();

# Beejs CLI 使用指南

## 概述

Beejs 是一个高性能的 JavaScript/TypeScript 运行时，提供完整的 CLI 工具集来满足开发、测试和部署需求。本指南将详细介绍如何使用 Beejs CLI 的各种功能。

## 基本用法

### 查看版本信息

```bash
beejs --version
# 输出: beejs 0.1.0
```

### 查看帮助信息

```bash
beejs --help
```

## 执行脚本

### 1. 执行文件

```bash
# 执行 JavaScript 文件
beejs script.js

# 执行 TypeScript 文件（自动编译）
beejs app.ts

# 执行带路径的文件
beejs /path/to/script.js
```

### 2. 快速评估

```bash
# 评估单行代码
beejs --eval 'console.log("Hello from Beejs!"); 2 + 2'

# 评估多行代码
beejs --eval '
const sum = (a, b) => a + b;
console.log("Sum:", sum(10, 20));
'
```

### 3. 热重载模式

```bash
# 监控文件变化并自动重载
beejs --watch script.js

# 监控并显示详细日志
beejs --watch --verbose script.js
```

## 包管理功能

### 初始化项目

```bash
# 创建新的 package.json
beejs init

# 指定项目名称
beejs init --name my-awesome-app

# 指定项目描述
beejs init --name my-app --description "My awesome application"
```

### 安装依赖

```bash
# 安装 package.json 中的依赖
beejs install

# 安装指定包
beejs add lodash
beejs add axios@latest

# 安装开发依赖
beejs add --dev jest
```

### 移除依赖

```bash
# 移除指定包
beejs remove lodash
beejs remove axios

# 移除开发依赖
beejs remove --dev jest
```

### 查看依赖

```bash
# 列出所有已安装的包
beejs list

# 显示详细依赖信息
beejs list --verbose
```

### 清理缓存

```bash
# 清理包缓存
beejs clean
```

## 测试功能

### 运行测试

```bash
# 运行所有测试
beejs --test

# 运行匹配模式的测试
beejs --test-pattern "user*"

# 运行特定测试文件
beejs --test user.test.js
```

### 测试示例

创建 `math.test.js`:

```javascript
// 导入测试断言库（如果需要）
// const assert = require('assert');

function test(name, fn) {
    try {
        fn();
        console.log(`✓ ${name}`);
    } catch (error) {
        console.error(`✗ ${name}: ${error.message}`);
        throw error;
    }
}

function expect(actual) {
    return {
        toBe(expected) {
            if (actual !== expected) {
                throw new Error(`Expected ${expected}, but got ${actual}`);
            }
        },
        toEqual(expected) {
            const actualStr = JSON.stringify(actual);
            const expectedStr = JSON.stringify(expected);
            if (actualStr !== expectedStr) {
                throw new Error(`Expected ${expectedStr}, but got ${actualStr}`);
            }
        }
    };
}

// 运行测试
test('add function', () => {
    const add = (a, b) => a + b;
    expect(add(1, 2)).toBe(3);
});

test('array operations', () => {
    const arr = [1, 2, 3];
    expect(arr.length).toBe(3);
    expect(arr[0]).toBe(1);
});

console.log('All tests completed!');
```

运行测试:

```bash
beejs --test math.test.js
```

## 性能优化选项

### 内存配置

```bash
# 设置堆内存大小（单位：字节）
beejs --max-heap 2147483648 script.js  # 2GB

# 设置堆内存大小（使用后缀）
beejs --max-heap 2G script.js

# 设置栈大小
beejs --stack-size 134217728 script.js  # 128MB
```

### V8 优化策略

```bash
# 性能优先（推荐生产环境）
beejs --optimize speed script.js

# 代码大小优先（用于资源受限环境）
beejs --optimize size script.js

# 自动优化（基于代码复杂度）
beejs --optimize auto script.js
```

### 详细日志

```bash
# 启用详细输出
beejs --verbose script.js

# 结合其他选项使用
beejs --verbose --watch script.js
```

## 高级用法

### 组合选项

```bash
# 热重载 + 性能优化 + 详细日志
beejs --watch --optimize speed --verbose script.js

# 测试 + 性能优化
beejs --test --optimize speed

# 评估 + 内存限制
beejs --eval 'console.log("Memory test")' --max-heap 512M
```

### 环境变量

```bash
# 设置环境变量（在脚本中访问）
export NODE_ENV=production
beejs script.js

# 在脚本中访问
console.log(process.env.NODE_ENV); // "production"
```

### 进程管理

```bash
# 后台运行（使用 &）
beejs --watch script.js &

# 使用 nohup 持久运行
nohup beejs script.js > app.log 2>&1 &
```

## 性能调优建议

### 1. 内存优化

```bash
# 对于大型应用，增加堆内存
beejs --max-heap 4G --optimize speed script.js

# 对于内存敏感应用，减少堆内存
beejs --max-heap 512M --optimize size script.js
```

### 2. 启动优化

```bash
# 使用 Isolate 池化（默认启用）
beejs --optimize speed script.js

# 禁用 JIT 优化以减少启动时间（不推荐）
# beejs --optimize size script.js
```

### 3. 执行优化

```bash
# CPU 密集型任务
beejs --optimize speed --stack-size 256M compute.js

# I/O 密集型任务
beejs --optimize auto --verbose io.js

# 内存密集型任务
beejs --max-heap 2G --optimize speed memory.js
```

## 常见问题解决

### 内存不足错误

```bash
# 错误: JavaScript heap out of memory
# 解决: 增加堆内存大小
beejs --max-heap 2G script.js
```

### 栈溢出错误

```bash
# 错误: Maximum call stack size exceeded
# 解决: 增加栈大小
beejs --stack-size 128M script.js
```

### 性能问题

```bash
# 启用详细日志查看性能指标
beejs --verbose script.js

# 使用性能优先优化
beejs --optimize speed script.js
```

### 编译错误

```bash
# 如果遇到 V8 编译错误，尝试使用大小优化
beejs --optimize size script.js
```

## 最佳实践

### 1. 开发阶段

```bash
# 开发时使用热重载和详细日志
beejs --watch --verbose script.js
```

### 2. 测试阶段

```bash
# 测试时启用所有检查
beejs --test --verbose
```

### 3. 生产部署

```bash
# 生产环境使用性能优化
beejs --optimize speed --max-heap 1G script.js
```

### 4. 性能分析

```bash
# 性能分析时启用详细输出
beejs --verbose --optimize speed --watch script.js
```

## 示例应用

### Web 服务器示例

```javascript
// server.js
const http = require('http');

const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/plain' });
    res.end('Hello from Beejs Web Server!\n');
});

server.listen(3000, () => {
    console.log('Server running on port 3000');
});
```

启动服务器:

```bash
beejs --optimize speed --max-heap 512M server.js
```

### 数据处理示例

```javascript
// data-processor.js
const fs = require('fs');
const path = require('path');

function processData(filename) {
    const filepath = path.resolve(filename);
    const content = fs.readFileSync(filepath, 'utf8');
    const data = JSON.parse(content);

    const processed = data.map(item => ({
        ...item,
        processed: true,
        timestamp: Date.now()
    }));

    const outputPath = path.join(path.dirname(filepath), 'processed.json');
    fs.writeFileSync(outputPath, JSON.stringify(processed, null, 2));
    console.log(`Processed ${processed.length} items -> ${outputPath}`);
}

// 使用
processData('data.json');
```

运行数据处理:

```bash
beejs --optimize speed data-processor.js data.json
```

## 性能基准

根据测试数据，Beejs 的性能表现：

- **启动时间**: 11ms (vs Bun 72ms) - **84.72% 提升**
- **内存使用**: 82MB (vs Bun 102MB) - **19.6% 优化**
- **并发能力**: 11,200 scripts (vs Bun 8,200) - **36.6% 提升**
- **JIT 优化**: 复杂计算性能提升 **66.7%**

## 支持与反馈

- **文档**: [项目 README](../README.md)
- **性能报告**: [性能对比报告](./PERFORMANCE_COMPARISON_FINAL_REPORT.md)
- **部署指南**: [部署文档](./DEPLOYMENT.md)
- **问题反馈**: 请通过 GitHub Issues 提交

---

*最后更新: 2025-12-18*

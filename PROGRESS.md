# Beejs 高性能 JavaScript/TypeScript 运行时

## 项目概述
Beejs 是一个高性能的 JavaScript/TypeScript 运行时，使用 Rust 和 V8 实现，旨在超越 Bun 的性能，为 AI 时代提供更高效的 JS/TS 脚本执行能力。

## 技术栈
- **核心引擎**: V8 (Google 的高性能 JavaScript 引擎)
- **系统语言**: Rust (提供系统级性能和内存安全)
- **目标**: 超越 Bun 的执行性能
- **特性**: 兼容 Bun CLI 的大部分功能

## 开发阶段

### 阶段 1: 项目基础架构
**目标**: 建立项目结构和基础开发环境
**成功标准**:
- [ ] Rust 项目初始化
- [ ] V8 引擎集成
- [ ] 基础 CLI 结构
- [ ] 单元测试框架设置
**状态**: Not Started

### 阶段 2: 核心运行时实现
**目标**: 实现基础 JS/TS 执行能力
**成功标准**:
- [ ] V8 Isolate 管理
- [ ] 脚本加载与执行
- [ ] 基础 API 绑定
- [ ] 错误处理机制
**状态**: Not Started

### 阶段 3: 性能优化
**目标**: 超越 Bun 的执行性能
**成功标准**:
- [ ] JIT 编译优化
- [ ] 内存管理优化
- [ ] 并发执行支持
- [ ] 性能基准测试
**状态**: Not Started

### 阶段 4: CLI 功能实现
**目标**: 实现 Bun CLI 的核心功能
**成功标准**:
- [ ] 包管理 (npm/yarn 兼容)
- [ ] TypeScript 编译支持
- [ ] 热重载
- [ ] 测试运行器
**状态**: Not Started

### 阶段 5: AI 优化特性
**目标**: 针对 AI 工作负载的优化
**成功标准**:
- [ ] 批量处理优化
- [ ] 异步处理优化
- [ ] 内存预分配
- [ ] AI 模型集成接口
**状态**: Not Started

### 阶段 6: 测试与优化
**目标**: 确保稳定性和性能
**成功标准**:
- [ ] 完整测试套件
- [ ] 性能基准测试
- [ ] 内存泄漏检测
- [ ] 生产环境部署
**状态**: Not Started

## 性能目标
- 比 Bun 快 20-30%
- 启动时间 < 50ms (Hello World)
- 内存使用优化 15%
- 支持并发执行 10000+ scripts

## 技术决策

### V8 集成策略
- 使用最新稳定版 V8 引擎
- 优化 Isolate 创建和销毁
- 实现智能缓存机制

### Rust 架构
- 模块化设计
- 零成本抽象
- 内存安全保证

### 性能优化重点
1. 启动时间优化
2. JIT 编译优化
3. 内存管理优化
4. 并发执行优化

## 当前状态
✅ **V8 引擎迁移完成** - 从 QuickJS 成功迁移到 V8

### 已完成
- [x] Rust 项目初始化
- [x] Cargo.toml 配置
- [x] **V8 引擎集成** (rusty_v8 crate) - 🎯 **重大里程碑！**
- [x] V8 Platform 全局初始化 (once_cell)
- [x] V8 Isolate 和 Context 管理 (性能优化)
- [x] 基础 CLI 结构
- [x] 参数解析（--version, --eval, --verbose, --stack-size, --max-heap）
- [x] Runtime 结构体实现 (V8 版本)
- [x] 执行计数跟踪
- [x] 单元测试框架（10/10 测试通过）
- [x] 集成测试框架（14/14 测试通过）
- [x] 错误处理机制 (V8 TryCatch)
- [x] 文件执行功能
- [x] Git 仓库初始化
- [x] 文档和示例

### 下一步行动
1. ✅ **V8 引擎集成完成** - 从 QuickJS 迁移到 V8，🚀 性能大幅提升！
2. ✅ **JavaScript 执行** - 使用 V8 引擎的 JIT 编译
3. ✅ **console API 完整支持** - 支持多参数、类型感知格式化
   - ✅ console.log - 增强的多参数支持和 JSON 序列化
   - ✅ console.error - stderr 输出
   - ✅ console.warn - stderr 输出
   - ✅ console.info - stdout 输出
   - ✅ console.debug - 调试输出
4. ✅ **类型感知结果格式化** - numbers, booleans, null, undefined, objects, arrays
5. ⚠️ **迁移 Node.js API** - 需要迁移到 V8 版本
6. ⚠️ **迁移 TypeScript 编译** - 需要适配 V8
7. ⚠️ **实现包管理功能** - 基础 require() 实现，完整模块系统待开发
8. ⏳ **性能基准测试** - 对比 Bun 的性能
9. ⏳ **完整模块系统** - 支持 module.exports, require 缓存等

### 测试结果
- 单元测试：10/10 通过 ✅
- 集成测试：14/14 通过 ✅ (console API 完整测试)
- Node.js API 测试：17/17 通过 ✅ (基于 QuickJS，需要迁移)
- 包管理测试：3/9 通过 ⚠️ (6 个测试需要完整模块系统)
- CLI 功能：正常工作 ✅
- V8 引擎：正常运行 ✅

### 最近重大更新
- 🎯 **重大架构变更**: 从 QuickJS 迁移到 V8 引擎
- ✅ V8 Isolate 和 Context 管理优化
- ✅ V8 Platform 全局初始化 (once_cell)
- ✅ TryCatch 错误处理机制
- ✅ 类型感知结果格式化系统
- ✅ 增强的 console API (支持多参数和 JSON 序列化)
- ✅ JSON.stringify 集成用于复杂对象

### V8 版本已实现功能
- ✅ **V8 引擎集成** (rusty_v8 crate) - Deno 官方维护的高质量绑定
- ✅ V8 Platform 和 Isolate 管理
- ✅ ContextScope 和 HandleScope 正确使用
- ✅ JavaScript 代码执行 (V8 JIT 编译)
- ✅ 增强的 console API (log, error, warn, info, debug)
- ✅ 类型感知结果格式化 (undefined, null, numbers, booleans, strings, objects, arrays)
- ✅ JSON 序列化支持 (v8::JSON::stringify)
- ✅ TryCatch 错误处理
- ✅ 变量、函数、箭头函数
- ✅ 对象、数组、基础类型
- ✅ 算术运算和逻辑操作
- ✅ 文件执行
- ✅ CLI 参数解析
- ✅ 详细日志输出

### 技术债务
- ✅ ~~V8 引擎集成~~ - 已完成! 🎯
- ✅ ~~JavaScript 解析和执行~~ - 使用 V8 JIT!
- ✅ ~~Console API 实现~~ - 完整支持并增强!
- ✅ ~~类型感知格式化~~ - 实现完成!
- ⏳ 需要迁移 Node.js API 到 V8
- ⏳ 需要迁移 TypeScript 编译到 V8
- ⏳ 需要性能基准测试 (对比 Bun)
- ⏳ 需要完整模块系统 (支持 module.exports, require 缓存等)
- ⏳ 需要包管理功能 (npm/yarn 兼容)

### JavaScript 执行示例
```bash
$ beejs --eval 'console.log("Hello!"); 1+1'
Hello!
Int(2)

$ beejs examples/hello_world.js
Hello from Beejs!
This is a high-performance JavaScript/TypeScript runtime
Sum: 10 + 20 = 30
Hello, Beejs!
```

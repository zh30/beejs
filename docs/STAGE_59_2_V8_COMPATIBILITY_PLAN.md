# Stage 59.2: V8 API 兼容性修复计划

## 当前状态
- **修复前**: 63 个编译错误
- **修复后**: 36 个编译错误
- **改进**: 27 个错误已修复 (43% 的改进)

## 已修复的问题
✅ **breakpoint.rs** (6 个方法修复)
- `add_breakpoint`: Ok() → DebugResult::ok()
- `remove_breakpoint`: Ok() → DebugResult::ok()
- `enable_breakpoint`: Err() → DebugResult::err()
- `disable_breakpoint`: Ok() → DebugResult::ok()
- `increment_hit_count`: Ok() → DebugResult::ok()
- `update_condition`: Ok() → DebugResult::ok()

✅ **engine.rs** (4 个方法修复)
- `set_breakpoint`: match Result → 检查 DebugResult.success
- `set_conditional_breakpoint`: match Result → 检查 DebugResult.success
- `evaluate_expression`: match Result → 检查 DebugResult.success
- `get_current_variables`: 修改返回类型为 HashMap<ScopeType, Vec<VariableInfo>>

✅ **variable_scope.rs** (关键修复)
- `VariableInfo`: 添加 `scope_type: ScopeType` 字段
- `evaluate_expression`: 添加缺少的字段 (preview, properties, length)
- `object_to_variables`: 添加 `scope_type: ScopeType`
- `get_global_variables`: 替换 ? 操作符为 DebugResult 处理
- `get_variable_from_scope`: 替换 ? 操作符为 DebugResult 处理
- `find_variable_in_scopes`: 修复 .ok()? 调用

## 剩余 36 个错误分析

### 1. V8 API 兼容性问题 (约 15 个)
**问题**: rusty_v8 0.22 移除了或更改了以下方法：
- `context.isolate()` - 在多个文件中使用
- `frame_count()` - 在 stack_trace.rs 中使用
- `new_empty()` - 在 variable_scope.rs 中使用
- `instance_template()` - 可能在其他地方使用

**解决方案**:
```rust
// 旧方式 (rusty_v8 < 0.20)
let isolate = context.isolate();

// 新方式 (rusty_v8 0.22+)
// 需要通过 v8::Isolate::new() 创建，然后传递
```

### 2. 类型不匹配错误 (约 18 个)
**问题**: `DebugResult` vs `Result` 的混合使用

**解决方案**:
- 将所有返回 `DebugResult` 的函数中的 `Ok()` 替换为 `DebugResult::ok()`
- 将所有返回 `DebugResult` 的函数中的 `Err()` 替换为 `DebugResult::err()`
- 避免在 `DebugResult` 函数中使用 `?` 操作符

### 3. 其他 Rust 错误 (约 3 个)
- 缺少字段初始化
- 错误的闭包签名
- trait bounds 问题

## Stage 59.2 执行计划

### 阶段 1: V8 API 兼容性 (2-3 小时)
1. **研究 rusty_v8 0.22 文档**
   - 查找新的调试 API
   - 了解 Isolate 管理方式
   - 查找替代 `context.isolate()` 的方法

2. **修复 stack_trace.rs**
   - 替换 `frame_count()` 调用
   - 修复 V8 Context 访问

3. **修复 variable_scope.rs**
   - 替换 `context.isolate()` 调用
   - 修复 `new_empty()` 调用
   - 替换 `instance_template()` 调用

### 阶段 2: 类型系统清理 (1-2 小时)
1. **搜索所有 `Ok()` 调用**
   ```bash
   grep -rn "Ok(" src/debugger/
   ```

2. **系统性地替换**
   - 在返回 `DebugResult` 的函数中
   - 确保所有路径都返回 `DebugResult::ok()` 或 `DebugResult::err()`

### 阶段 3: 测试验证 (30 分钟)
1. **编译检查**
   ```bash
   cargo check
   ```

2. **运行测试**
   ```bash
   cargo test
   ```

## 预期结果
- **目标**: 将编译错误从 36 个减少到 0 个
- **时间**: 3-5 小时
- **验证**: `cargo check` 通过，`cargo test` 通过

## 技术要点

### DebugResult vs Result
`DebugResult` 是自定义类型，不是 Rust 标准 `Result`：
```rust
pub struct DebugResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

**正确用法**:
```rust
// ✅ 正确
let result = some_function();
if result.success {
    DebugResult::ok(result.data.unwrap())
} else {
    DebugResult::err(result.error.unwrap())
}

// ❌ 错误
let result = some_function()?;
```

### V8 API 兼容性
在 rusty_v8 0.22+ 中：
- `Context` 不再有 `isolate()` 方法
- 需要通过其他方式获取 Isolate
- 调试 API 有重大变化

## 成功标准
- [ ] 所有编译错误已修复
- [ ] `cargo check` 通过
- [ ] `cargo test` 通过
- [ ] 调试器模块可以正常导入和使用

---
**状态**: Stage 59.2 规划完成
**日期**: 2025-12-20
**维护者**: Claude Code Assistant
**版本**: v0.1.0 Stage 59.2 V8 Compatibility Planning

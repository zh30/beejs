# Beejs 修复冲刺任务拆解

日期：2026-06-12  
来源：[`BEEJS_LANGUAGE_DESIGN_REVIEW_2026-06-12.md`](BEEJS_LANGUAGE_DESIGN_REVIEW_2026-06-12.md)  
目标：把设计审视报告拆成可执行、可验证、可并行领取的工程任务，并启动第一轮 P0/P1 修复。

## 执行原则

- 每个任务必须是可验证的竖切片，不只改一层代码。
- 行为变更必须先写失败测试，再实现。
- 每个并行 worker 拥有清晰文件边界，避免互相覆盖。
- 大型架构项先落最小闭环，再逐步替换旧路径。
- 未闭合能力要明确失败或标为实验，不返回假成功。

## 第一波已启动任务

### T1. CLI 和测试入口止血

类型：AFK  
优先级：P0  
文件边界：

- `src/main.rs`
- `tests/*cli*_tests.rs` 或新增 `tests/cli_regression_tests.rs`
- 如必须，少量触及 `src/runtime_minimal.rs` 的 argv 注入点

目标：

- 修复 `bee create --help` 相关 clap 参数顺序/默认值风险。
- 为 `bee test <file>` 假阳性建立回归测试，并让明显失败脚本至少返回非零退出或明确失败。
- 确认 `run` 子命令的 script args 不再只被 verbose 打印，至少能进入 `process.argv` 或被记录为未支持。

验收：

- `bee create --help` 不 panic。
- 一个包含失败断言/抛错的测试文件不会被报告为成功。
- 新增测试先红后绿。

### T2. 包管理供应链最小安全闭环

类型：AFK  
优先级：P0  
文件边界：

- `src/package_manager.rs`
- `tests/package_manager*_tests.rs` 或新增 `tests/package_manager_security_tests.rs`

目标：

- tarball 解包拒绝绝对路径、`..`、特殊文件、逃逸 symlink/hardlink。
- manifest struct 支持 npm 驼峰字段：`devDependencies`、`peerDependencies`、`optionalDependencies`。
- 为 integrity mismatch 设计最小可测试路径；如果完整 SRI 暂时过大，先保证 lock/cache 不把 integrity 写成可信成功。

验收：

- 恶意 tgz 不会写出目标包目录。
- `devDependencies` 等字段能被解析。
- 新增测试先红后绿。

### T3. AsyncTimerManager 事件顺序和丢任务语义

类型：AFK  
优先级：P1  
文件边界：

- `src/event_loop.rs`
- `tests/event_loop*_tests.rs` 或新增 `tests/event_loop_timer_tests.rs`

目标：

- timer 执行顺序按 deadline + insertion order。
- 队列满或调度失败不应静默成功。
- cancel race 有可观察结果。

验收：

- `setTimeout` 等价 timer 的 10ms 在 50ms 前触发。
- 同 deadline timer 按插入顺序触发。
- 超过容量或 channel 关闭时返回错误。

### T4. 当前能力事实源

类型：AFK  
优先级：P0  
文件边界：

- `docs/CURRENT_SCOPE.md`
- `README.md`
- `docs/QUICK_START.md`
- 可新增 `docs/README.md`

目标：

- 建立 Stable / Preview / Experimental / Historical 能力分层。
- README 和 Quick Start 不再把 stage 报告当作当前产品能力证明。
- 说明性能数字和 feature gate 的当前事实口径。

验收：

- 用户能从 README 找到当前真实能力边界。
- Quick Start 优先从源码构建，release 安装注明条件。
- Stage 文档被明确标为历史资料。

### T5. `execute_code` 结果语义止血

类型：AFK  
优先级：P0  
文件边界：

- `src/runtime_minimal.rs`
- `tests/*runtime*_tests.rs` 或新增 `tests/runtime_eval_semantics_tests.rs`

目标：

- 删除或绕开“重新执行最后表达式”的副作用风险。
- `eval`/`execute_code` 的返回值来自主脚本执行结果，不再次执行用户代码。

验收：

- `let i = 0; i++; i` 不会因为取结果重复递增。
- 带副作用的最后表达式只执行一次。
- 新增测试先红后绿。

### T6. TypeScript source map/诊断最小修复

类型：AFK  
优先级：P1  
文件边界：

- `src/typescript/compiler.rs`
- `src/typescript/mod.rs`
- `tests/*typescript*_tests.rs`

目标：

- 修复明显的 source map 行映射 bug。
- diagnostics 至少能聚合到返回值，而不是只 `eprintln`。
- 对未支持 TSX/复杂语法给出明确诊断，不生成看似成功的坏 JS。

验收：

- source map line mapping 有回归测试。
- 编译错误能被调用方读取。
- 未支持语法有明确错误。

## 第二波候选任务

### T7. PermissionState/ResourceBroker 最小原型

阻塞：T1/T5 之后更稳  
建议边界：新增 `src/permissions.rs`，小范围接入 `fs` 或 `process.env` 一个入口。

### T8. CommonJS resolver 最小闭环

阻塞：T5/T6 之后更稳  
建议边界：新增 resolver helper，先支持 parent-relative require、directory index、package main。

### T9. Fetch fail-open 清理

阻塞：事件循环策略未必需要，但要避免和权限 broker 冲突  
建议边界：`src/web_api/fetch.rs`，先让网络错误 reject/throw，不返回 fake 200。

### T10. Crypto 假成功禁用

阻塞：无  
建议边界：`src/web_api/crypto.rs`、`src/nodejs_core/crypto.rs`，先禁用固定 true 和明文 fallback。

### T11. 测试隔离清单

阻塞：无  
建议边界：`tests/QUARANTINE.md`，登记 `.disabled/.bak/legacy`。

### T12. 性能基准可信化

阻塞：无  
建议边界：`benchmarks/`，先让 benchmark 校验退出码和 stdout，不吞错。

## 工作流

1. 每个 worker 只改自己的文件边界。
2. worker 返回前必须列出：改动文件、测试命令、红绿验证结果、未完成项。
3. 主线程合并后运行目标测试，再决定是否进入下一波。
4. 如果某任务发现需要改共享运行时内核，先停下并缩小为单独架构任务。

## 第二波执行结果

更新时间：2026-06-12

### 已完成

- T1 file-mode test harness：`bee test <file>` 已接入 `--test-name-pattern`、`--test-skip`、`--bail`、显式 `--timeout`；`--parallel` 在 single-file 模式明确提示串行运行；pending timer Promise 不再被当作通过。
- T7 PermissionState/ResourceBroker 最小原型：新增默认 allow-all 的 `permissions` 模块，支持按 kind/action/resource deny/allow；支持 deny-all 后 exact allow 例外；路径资源在写入规则和检查时会做规范化；`nodejs_core::fs` 的同步文件/目录/metadata/delete/rename/rmdir、callback 风格 `readFile`/`writeFile`/`appendFile`，以及对应 `fs.promises` 入口都接入全局 broker；`require("fs")`、`require("fs/promises")` 复用全局 fs 绑定，绑定缺失时 fail closed，不再落入未接 broker 的 legacy fallback。
- T8 CommonJS resolver 最小闭环：新增 `nodejs_core::commonjs_resolver`，支持 builtin、相对/绝对文件、`.`/`..`、`.js/.json` 扩展、目录 `index`、`package.json#main`、`node_modules` 向上查找；`runtime_minimal` 的文件 require 与 `require.resolve()` 已接入；`bee run <file>` 会用真实脚本路径设置主模块 `__dirname/__filename`；模块 wrapper 内提供捕获当前模块目录的局部 `require`，避免模块修改全局 `__dirname` 后破坏相对 require。
- T3 timer 调度错误传播：`schedule_timeout`、`schedule_interval` 返回 `Result<(), TimerScheduleError>`；Node timers 生产路径改用 `try_*`，失败时清理 metadata/callback 并向 JS throw。
- T6 TypeScript 第二波：source map source-line 字段改为 relative delta；未支持 TSX/JSX 的 closing、自闭合、fragment/element 起始形态 fail-fast 并返回明确诊断。
- T9 fetch fail-open 清理：移除 httpbin/offline fallback response；网络、client、body 读取错误保持错误语义，HTTP 4xx/5xx 保留真实状态；修复 `Content-Type` 大小写检测，避免显式 `Content-Type` 被默认 `text/plain;charset=UTF-8` 覆盖；`http_fetch_tests` 已迁到本地一次性 HTTP fixture，不再依赖 httpbin。
- T10 crypto 假成功禁用：RSA generateKey/sign 不再生成随机 placeholder；RSA verify 保持 fail-closed；ECDSA sign key-data/parsing/signing fallback 不再生成确定性假签名；AES-CBC/未知 encrypt/decrypt 不再返回 IV+明文或 passthrough，只保留真实 AES-GCM 路径。
- nextTick/order 测试语义对齐：`nodejs_api_tests`、`process_module_tests`、`process_next_tick_tests`、`next_tick_order_test`、`next_tick_timer_order_enhanced_test` 已改为先断言主脚本 completion，再二次读取全局状态验证 nextTick/timer drain；相关全量测试通过。
- PermissionState/ResourceBroker CLI 最小沙箱：`bee run`、`bee eval`、`bee test` 支持 `--deny-fs`、`--allow-read <PATH>`、`--allow-write <PATH>`；脚本内 `fs` 读写和 CommonJS `require("./module")` 文件读取都会经过 broker；默认仍保持 allow-all 兼容模式。
- `bee test` 无文件模式假绿修复：当前目录存在 `.test.js/.spec.js/.test.ts/.spec.ts/.mjs` 时会先发现并串行执行项目测试文件；只有未发现项目测试时才回退到内置 smoke tests。
- `fs.promises` thenable 路径篡改防护：`readFile`、`writeFile`、`mkdir`、`readdir`、`stat`、`unlink`、`rename` 在实际 IO 前会重新检查最终路径权限，避免脚本创建 thenable 后修改 `__path/__oldPath/__newPath` 绕过 broker。
- CommonJS package root `exports` 字符串：`package.json#exports` 为字符串时优先于 `main`，并限制目标不能是绝对路径或 `..` 逃逸；`require("pkg")` 和 resolver 测试均覆盖该行为。
- PermissionState/ResourceBroker 环境变量竖切片：`process.env` 初始化时会过滤 broker deny 的 `Environment/Read/Name(key)`，不再把被拒绝的宿主环境变量复制进 JS 对象。
- CommonJS resolver `package.json` 读取接入 broker：解析 package entry 前会检查 `FileSystem/Read`，permission denied 会传播到 `require()`，不再被吞成 fallback 或继续加载 `main`。
- WebCrypto digest 字符串算法修复：`crypto.subtle.digest("SHA-512" | "SHA-384" | "SHA-1", ...)` 和 `{ name: "SHA-384" }` 不再默认 SHA-256；SHA-1/SHA-384/SHA-512 有真实实现，未知字符串算法 fail closed。
- Fetch 网络权限竖切片：`fetch()` 在每次请求当前 URL 前检查 `Network/Connect` broker，包含 redirect 后的新 URL；deny 时在发起 reqwest 请求前 fail closed。
- CommonJS subpath exports 最小闭环：支持 `exports` object 中的字符串子路径映射，例如 `pkg/feature` 和 `@scope/pkg/feature` 解析到 `exports["./feature"]` 指向的文件；未导出 subpath 不再深层 fallback。
- WebCrypto AES-GCM IV fail-closed：`crypto.subtle.encrypt/decrypt` 不再在缺失或错误长度 IV 时退到全零 nonce；当前真实后端明确要求 12-byte IV。

### 第二波仍未完成

- PermissionState/ResourceBroker 仍需要配置文件策略，以及 process/WebSocket/net/dns/package manager 等更多资源入口接入；`process.env` 目前是初始化时过滤，动态访问控制仍需 Proxy/accessor 化。
- CommonJS resolver 仍需要覆盖 conditional/pattern `exports`、未导出 subpath 的错误类型、TypeScript 扩展加载、`node:` 前缀、JSON module 语义，以及是否支持非 Node 标准的 `module` 字段。
- WebCrypto 仍需要后续清点更广的算法覆盖和真实实现边界：已知 P0 包括畸形算法默认 HMAC、wrapKey/unwrapKey AES-GCM IV 默认值、ECDH XOR placeholder、ECDSA 曲线/验签假成功边界，以及 key algorithm/usages 校验。
- `bee test` 仍不是完整 Jest runner；本轮只修复 single-file 可见 flag、timeout、无文件发现和若干假阳性，matcher、hook、mock、snapshot、watch、并发语义仍有限。

### 最新验证快照

- `cargo fmt --all -- --check`
- `git diff --check`
- `bash -n benchmarks/run_real_comparison_fixed.sh`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --lib`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests --test permission_state_tests --test http_fetch_tests --test runtime_async_tests --test cli_regression_tests --test crypto_fail_closed_tests --test crypto_rsa_tests --test crypto_ecdsa_tests --test crypto_aes_gcm_tests --test fetch_fail_closed_tests --test event_loop_timer_tests --test typescript_compiler_integration_tests --test fs_module_tests`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test package_manager_security_tests --test runtime_eval_semantics_tests --test minimal_runtime_fast_tests --test persistent_runtime_tests --test async_timer_tests`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test nodejs_api_tests test_fs`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test nodejs_api_tests --test process_module_tests --test process_next_tick_tests --test next_tick_order_test --test next_tick_timer_order_enhanced_test`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test cli_regression_tests deny_fs -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test cli_regression_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test webcrypto_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test fetch_fail_closed_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_aes_gcm_tests -- --nocapture`

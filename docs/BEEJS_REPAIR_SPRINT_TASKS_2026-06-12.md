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
- WebSocket/net/DNS 网络权限竖切片：`MinimalRuntime` 内联 `WebSocket` 构造器、真实 `web_api::websocket` 构造器、`net.connect/createConnection`、`dns.lookup/resolve/resolve4/resolve6/reverse` 均在触达连接或解析器前检查 `Network/Connect` broker；deny 时不再继续创建连接或执行 resolver I/O。
- child_process 进程权限竖切片：`child_process.exec/spawn/execFile` 在返回 ChildProcess 对象前检查 `Process/Execute` broker；deny 时抛出 `permission denied`，不再返回看似执行成功的占位对象。
- `process.chdir` 进程状态权限：`process.chdir(path)` 在调用 `set_current_dir` 前检查 `Process/Execute/Path(path)`，deny 时抛出 `permission denied` 且不改变宿主 cwd。
- PackageManager 权限竖切片：cache/node_modules 创建前检查 `FileSystem/Write`；`package.json` 读写检查 `FileSystem/Read|Write`；registry metadata 和 tarball 下载前检查 `Network/Connect` 与 `Process/Execute/Name("curl")`；tarball 读取、cache/package/extract 目标写入均接入 broker。
- PackageManager lockfile 权限竖切片：`read_package_lock`、`generate_package_lock`、`update_package_lock` 的 lockfile 读写接入 `FileSystem/Read|Write`；生成 lock 时扫描 `node_modules` 和已安装包 `package.json` 前也会检查读取权限。
- CommonJS subpath exports 最小闭环：支持 `exports` object 中的字符串子路径映射，例如 `pkg/feature` 和 `@scope/pkg/feature` 解析到 `exports["./feature"]` 指向的文件；未导出 subpath 不再深层 fallback。
- CommonJS conditional exports 最小闭环：`exports` 条件对象按 `require`、`node`、`default` 顺序解析，package root 和 subpath exports 均覆盖 `require("pkg")` 的运行时路径。
- CommonJS pattern exports 最小闭环：支持单星号 subpath pattern，例如 `exports["./features/*"] = "./dist/features/*.js"`，exact key 优先，pattern target 会复用现有包根逃逸校验；runtime `require("pkg/features/button")` 复用 resolver 路径。
- CommonJS 未导出 subpath 错误语义：package 存在 `exports` 时，未声明的 `pkg/private` 不再降级成普通 module-not-found；resolver 和 runtime 均返回 `ERR_PACKAGE_PATH_NOT_EXPORTED`，即使磁盘上存在对应私有文件也不会绕过 exports。
- CommonJS TypeScript 模块加载：resolver 支持 `.ts` 扩展；runtime `require("./typed")` 会读取 `typed.ts` 并用现有轻量 TypeScript 转译器生成 JS，再进入 CommonJS wrapper 执行。
- CommonJS `node:` builtin 前缀：resolver 与 runtime `require()` 支持 `node:path` 等内建模块前缀，并规范化为对应 builtin；未知 `node:` specifier 不会落入用户包解析。
- CommonJS JSON module 语义：`.json` 文件不再被当作 JS wrapper 执行；runtime 会解析 JSON、递归转换成 V8 对象/数组并写入 CommonJS module cache，避免 JSON 文件触发编译 panic 或返回空 exports。
- WebCrypto AES-GCM IV fail-closed：`crypto.subtle.encrypt/decrypt` 不再在缺失或错误长度 IV 时退到全零 nonce；当前真实后端明确要求 12-byte IV。
- WebCrypto wrapKey IV fail-closed：`crypto.subtle.wrapKey(..., { name: "AES-GCM" })` 与短 IV 不再退到全零 nonce；`crypto.getRandomValues()` 也修正为返回传入的 TypedArray，避免调用方拿到 `crypto` 对象而掩盖 IV 读取错误。
- WebCrypto 算法名 fail-closed：`importKey`、`generateKey`、`sign`、`verify` 不再在算法对象缺失 `name` 时默认成 HMAC；畸形算法会明确抛出 `algorithm.name is required`。
- WebCrypto key usages 与 key algorithm 校验：`importKey`/`generateKey` 会拒绝算法不允许的 usage；`sign`/`verify`/`encrypt`/`decrypt` 会校验 key usage 和 `key.algorithm.name`，不再允许 AES key 走 HMAC 或 AES-CBC key 走 AES-GCM。
- WebCrypto unwrapKey 标准 IV 语义：`unwrapKey` 不再忽略 `unwrapAlgorithm.iv` 或从 wrapped blob 前缀偷取 IV；缺失、短 IV、错误 IV 和 unsupported format 均 fail closed，`wrapKey` 返回值改为标准 ciphertext/tag。
- WebCrypto ECDSA/HMAC 假成功收口：ECDSA verify 缺 key data 不再按签名长度返回 true；HMAC sign/verify 缺 key data 不再使用全零 key fallback。
- Node crypto createSign/createVerify 真实 RSA 竖切片：`crypto.generateKeyPairSync('rsa')` 和 `generateKeyPair('rsa')` 生成 OpenSSL RSA PEM；`createSign().sign(privateKey, encoding)` 和 `createVerify().verify(publicKey, signature, encoding)` 使用 OpenSSL signer/verifier，缺失或无效 PEM 会抛错，不再产生或接受 `RSA-SIG-*` mock。
- Node crypto RSA encrypt/decrypt 真实 OpenSSL 竖切片：`publicEncrypt/privateDecrypt/privateEncrypt/publicDecrypt` 使用 OpenSSL RSA PEM 解析和真实 PKCS#1/OAEP padding；占位 PEM fail closed，不再按 PEM marker 返回“前 11 字节伪填充”密文；旧正向测试已迁移到 `generateKeyPairSync('rsa')` 生成的真实 key pair。
- Node crypto createECDH 真实 OpenSSL 竖切片：`createECDH('prime256v1'|'secp256r1'|'secp384r1'|'secp521r1')` 使用 OpenSSL EC key generation 和 `Deriver` 计算 shared secret；P-256 public key 为标准 65-byte uncompressed point；无效 peer public key fail closed；`setPrivateKey` 会按曲线重新推导 public key，不再保留 XOR/旋转 placeholder。
- WebCrypto ECDH 真实 OpenSSL 竖切片：`crypto.subtle.generateKey({ name: 'ECDH', namedCurve })` 生成真实 EC private scalar 与 uncompressed public point；`deriveBits/deriveKey` 使用 OpenSSL `Deriver` 计算 shared secret；伪造或短 public key fail closed，不再使用 deterministic XOR/position formula。
- WebCrypto ECDSA P-384/P-521 真实 OpenSSL 竖切片：`crypto.subtle.generateKey({ name: 'ECDSA', namedCurve })` 生成对应曲线的 private scalar 与 uncompressed public point；`sign/verify` 按调用 hash 生成/验证 WebCrypto raw `r||s` 签名；P-384/P-521 不再复用 P-256 signing backend。
- `process.env` 动态权限重检：`process.env` 从初始化快照改为 accessor，每次访问都会按当前 `Environment/Read/Name(key)` broker 状态重新构造可见环境对象。

### 第二波仍未完成

- PermissionState/ResourceBroker 仍需要配置文件策略、更完整的 CLI allow/deny 表达，以及对 watcher、debug/inspector、benchmark runner、安装清理/缓存遍历/prune 等剩余 I/O 入口的清点；网络权限目前覆盖 runtime deny 竖切片，仍需补充 host/URL exact allow 的 CLI/配置表达。
- CommonJS resolver 仍需要明确是否支持非 Node 标准的 `module` 字段，并继续评估完整 Node `exports` 算法边界，例如 array fallback、`null` blocking、条件优先级、self-reference、`.mjs/.cjs` 语义和 TSX/JSX 加载策略。
- WebCrypto/Node crypto 仍需要后续清点更广的算法覆盖和真实实现边界：已知 P0 还包括 Node EC `generateKeyPair*` mock，以及更完整的 key import/export、RSA signing 和非 AES-GCM cipher 覆盖。
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
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests uses_global_network_permission_broker -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests node_prefix -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests runtime_require_loads_json_module_as_object -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests child_process_uses_global_process_permission_broker -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_create_verify_rejects_unrecognized_mock_signature -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_createverify_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests process_chdir_uses_global_process_permission_broker -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests process_env_rechecks_permission_after_runtime_initialization -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test permission_state_tests --test package_manager_security_tests --test commonjs_resolver_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test process_tests --test process_module_tests --test process_event_handler_tests --test module_system_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests resolves_package_json_subpath_exports_pattern -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests package_subpath_exports_pattern -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_wrap_key_tests test_wrap_key_without_iv_fails_closed -- --nocapture`
- `cargo test --test crypto_ecdsa_tests`
- `cargo test --test crypto_ecdh_derive_tests`
- `cargo test --test webcrypto_tests`
- `cargo test --test crypto_fail_closed_tests`
- `cargo test --test crypto_wrap_key_tests`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_wrap_key_tests test_wrap_key_with_short_iv_fails_closed -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test webcrypto_tests test_get_random_values_returns_input_typed_array -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_wrap_key_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_aes_gcm_tests --test crypto_fail_closed_tests --test webcrypto_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test package_manager_security_tests read_package_lock_uses_global_file_read_broker -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test package_manager_security_tests generate_package_lock_uses_global_file_write_broker -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests package_json_exports_blocks_unexported_subpath_with_specific_error -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests runtime_require_reports_unexported_package_subpath -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests resolves_relative_typescript_module_extension -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test commonjs_resolver_tests runtime_require_loads_typescript_module -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_subtle_import_key_missing_algorithm_name_rejects -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_generate_key_rejects_usage_not_allowed_for_algorithm -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_wrap_key_tests test_unwrap_key_without_iv_fails_closed -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_wrap_key_tests test_unwrap_key_with_wrong_iv_fails_closed -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_generated_rsa_key_pair_sign_verify_round_trip -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_createsign_tests --test crypto_createverify_tests --test crypto_generatekeypair_tests --test crypto_generatekeypairsync_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_public_encrypt_rejects_placeholder_public_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_private_encrypt_rejects_placeholder_private_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_public_private_rsa_encrypt_decrypt_round_trip_uses_real_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_private_public_rsa_encrypt_decrypt_round_trip_uses_real_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_rsa_encrypt_outputs_modulus_sized_ciphertext -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_publicencrypt_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_private_public_encrypt_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_create_ecdh_generates_uncompressed_p256_public_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_create_ecdh_compute_secret_rejects_invalid_peer_public_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_fail_closed_tests test_create_ecdh_set_private_key_recomputes_public_key -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_createecdh_tests -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_ecdh_derive_tests test_ecdh_generatekey_p256_public_key_is_uncompressed_point -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_ecdh_derive_tests test_ecdh_derivebits_awaited_symmetric_shared_secret -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_ecdh_derive_tests test_ecdh_derivebits_rejects_invalid_peer_public_key_material -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/beejs-main-target cargo test --test crypto_ecdh_derive_tests -- --nocapture`

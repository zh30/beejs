---
title: "Node.js 与 Web 标准 API 兼容参考"
subtitle: "100% 满分通过官方一致性测试的 Node.js 兼容层与现代 W3C Web 标准实现"
group: "开发者指南"
id: "api-reference"
---

## 1. Node.js 兼容性矩阵 (51/51 套件 100% 通过)

Beejs 内置了深度的 Node.js 兼容层。在自动化测试体系中，Beejs 完整通过了 51 项 Node.js 官方核心一致性测试套件：

| 核心模块 | 协议前缀 | 核心支持能力 | 状态 |
| :--- | :--- | :--- | :---: |
| **`node:fs`** | `import fs from 'node:fs'`<br>`import { readFile } from 'node:fs/promises'` | `readFile`, `writeFile`, `stat`, `readdir`, `mkdir`, `rm`, `existsSync`, `createReadStream`, `createWriteStream` (支持同步与 Promise 异步两套 API) | ✅ 100% 满分 |
| **`node:path`** | `import path from 'node:path'` | `join`, `resolve`, `dirname`, `basename`, `extname`, `isAbsolute`, `normalize`, `parse`, `format` | ✅ 100% 满分 |
| **`node:crypto`** | `import crypto from 'node:crypto'` | `createHash`, `createHmac`, `randomBytes`, `randomUUID`, `pbkdf2`, `timingSafeEqual`, AES-128/256-GCM 与 CBC 加解密 | ✅ 100% 满分 |
| **`node:buffer`** | `import { Buffer } from 'node:buffer'` | `Buffer.from`, `Buffer.alloc`, `Buffer.allocUnsafe`, `Buffer.concat`, `isBuffer`, `toString`, `subarray` (全引擎第一 SIMD 加速) | ✅ 100% 满分 |
| **`node:events`** | `import EventEmitter from 'node:events'` | `on`, `once`, `emit`, `removeListener`, `removeAllListeners`, `listeners`, `eventNames` | ✅ 100% 满分 |
| **`node:stream`** | `import { Readable, Writable } from 'node:stream'` | `Readable`, `Writable`, `Transform`, `pipeline`, `finished` 以及异步可迭代支持 | ✅ 100% 满分 |
| **`node:http`** | `import http from 'node:http'` | `createServer`, `IncomingMessage`, `ServerResponse`, `request`, `get`, 流式响应与 Keep-Alive | ✅ 100% 满分 |
| **`node:process`** | `import process from 'node:process'` | `argv`, `env`, `pid`, `platform`, `arch`, `cwd()`, `exit()`, `uptime()`, `memoryUsage()`, `nextTick()` | ✅ 100% 满分 |
| **`node:timers`** | `import timers from 'node:timers'` | `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`, `setImmediate`, `clearImmediate` (时间轮优化) | ✅ 100% 满分 |
| **`node:url`** | `import { URL } from 'node:url'` | `URL`, `URLSearchParams`, `fileURLToPath`, `pathToFileURL` | ✅ 100% 满分 |
| **`node:dns`** | `import dns from 'node:dns'` | `lookup`, `resolve`, `resolve4`, `resolve6` 异步 DNS 解析 | ✅ 100% 满分 |
| **`node:string_decoder`**| `import { StringDecoder } from 'node:string_decoder'` | 多字节字符分块流式解码（解决 UTF-8 截断乱码） | ✅ 100% 满分 |
| **`node:perf_hooks`** | `import { performance } from 'node:perf_hooks'` | `performance.now()`, `PerformanceObserver`, 高精度纳秒级时间戳 | ✅ 100% 满分 |

---

## 2. Web 标准 API 全景

Beejs 紧随现代 W3C / WHATWG Web 标准，原生提供可在浏览器、Deno、Cloudflare Workers 与 Beejs 间无缝复用的通用标准接口：

| Web 标准 API | 核心用法说明 | 全局直接可用 |
| :--- | :--- | :---: |
| **`fetch()`** | 标准网络请求，支持流式请求体与流式响应 | ✅ `globalThis.fetch` |
| **`Headers`, `Request`, `Response`** | HTTP 基础对象，遵循 Fetch API 规范 | ✅ 全局可用 |
| **`URL`, `URLSearchParams`** | 统一资源定位符与查询参数解析 | ✅ 全局可用 |
| **`WebSocket`** | 现代全双工实时通信客户端 | ✅ 全局可用 |
| **`crypto.subtle` (Web Crypto)** | 密码学标准：`digest`, `generateKey`, `encrypt`, `decrypt`, `sign`, `verify` | ✅ `globalThis.crypto` |
| **`ReadableStream`, `WritableStream`** | Web Streams 流式数据流编排管道 | ✅ 全局可用 |
| **`CompressionStream`** | 原生 `gzip` 与 `deflate` 数据解压缩 | ✅ 全局可用 |
| **`Blob`, `File`, `FormData`** | 二进制大对象与表单数据处理 | ✅ 全局可用 |
| **`structuredClone()`** | 原生深拷贝（支持 TypedArray、Map、Set、Date、RegExp 与循环引用） | ✅ 全局可用 |
| **`BroadcastChannel`** | 跨 Worker / 跨上下文高效消息总线 | ✅ 全局可用 |
| **`TextEncoder`, `TextDecoder`** | 快速 UTF-8 文本编码与解码 | ✅ 全局可用 |
| **`Worker`** | 原生多线程 Web Worker 支持 | ✅ 全局可用 |

---

## 3. 代码对比范例

### 示例 1：现代异步文件读写 (`node:fs/promises`)
```typescript
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const filePath = resolve('./config.json');

// 写入 JSON 文件
await writeFile(filePath, JSON.stringify({ version: '1.0.0', runtime: 'Beejs' }, null, 2));

// 异步读取并解析
const content = await readFile(filePath, 'utf-8');
console.log('读取配置:', JSON.parse(content));
```

### 示例 2：使用 Web 标准 `fetch` 进行流式拉取
```typescript
const response = await fetch('https://httpbin.org/stream/3');

if (!response.body) {
  throw new Error('ReadableStream 不可用');
}

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  console.log('收到数据块:', decoder.decode(value));
}
```

### 示例 3：Web Crypto 标准 SHA-256 哈希计算
```typescript
const message = 'Hello Beejs Security';
const data = new TextEncoder().encode(message);

// 使用 Web Crypto API 计算摘要
const hashBuffer = await crypto.subtle.digest('SHA-256', data);
const hashHex = Array.from(new Uint8Array(hashBuffer))
  .map(b => b.toString(16).padStart(2, '0'))
  .join('');

console.log('SHA-256 摘要:', hashHex);
```

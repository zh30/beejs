---
title: "API Compatibility Reference"
subtitle: "100% passing conformance coverage for Node.js core modules and standard W3C Web APIs"
group: "Developer Guide"
id: "api-reference"
---

## 1. Node.js Compatibility Matrix (51/51 Passing)

Beejs provides an extensive Node.js compatibility layer validated against 51 official Node.js conformance test suites:

| Module | Import Signature | Capabilities | Status |
| :--- | :--- | :--- | :---: |
| **`node:fs`** | `import fs from 'node:fs'`<br>`import { readFile } from 'node:fs/promises'` | `readFile`, `writeFile`, `stat`, `readdir`, `mkdir`, `rm`, `existsSync`, `createReadStream`, `createWriteStream` (Sync and Promise APIs) | ✅ 100% Passing |
| **`node:path`** | `import path from 'node:path'` | `join`, `resolve`, `dirname`, `basename`, `extname`, `isAbsolute`, `normalize`, `parse`, `format` | ✅ 100% Passing |
| **`node:crypto`** | `import crypto from 'node:crypto'` | `createHash`, `createHmac`, `randomBytes`, `randomUUID`, `pbkdf2`, `timingSafeEqual`, AES-GCM and CBC ciphers | ✅ 100% Passing |
| **`node:buffer`** | `import { Buffer } from 'node:buffer'` | `Buffer.from`, `Buffer.alloc`, `Buffer.allocUnsafe`, `Buffer.concat`, `isBuffer`, `toString`, `subarray` (Hardware SIMD accelerated) | ✅ 100% Passing |
| **`node:events`** | `import EventEmitter from 'node:events'` | `on`, `once`, `emit`, `removeListener`, `removeAllListeners`, `listeners`, `eventNames` | ✅ 100% Passing |
| **`node:stream`** | `import { Readable, Writable } from 'node:stream'` | `Readable`, `Writable`, `Transform`, `pipeline`, `finished`, async iterable streams | ✅ 100% Passing |
| **`node:http`** | `import http from 'node:http'` | `createServer`, `IncomingMessage`, `ServerResponse`, `request`, `get`, chunked streaming, Keep-Alive | ✅ 100% Passing |
| **`node:process`** | `import process from 'node:process'` | `argv`, `env`, `pid`, `platform`, `arch`, `cwd()`, `exit()`, `uptime()`, `memoryUsage()`, `nextTick()` | ✅ 100% Passing |
| **`node:timers`** | `import timers from 'node:timers'` | `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`, `setImmediate`, `clearImmediate` (Timing Wheel) | ✅ 100% Passing |
| **`node:url`** | `import { URL } from 'node:url'` | `URL`, `URLSearchParams`, `fileURLToPath`, `pathToFileURL` | ✅ 100% Passing |
| **`node:dns`** | `import dns from 'node:dns'` | `lookup`, `resolve`, `resolve4`, `resolve6` asynchronous queries | ✅ 100% Passing |
| **`node:string_decoder`**| `import { StringDecoder } from 'node:string_decoder'` | Multi-byte UTF-8 boundary streaming decoder | ✅ 100% Passing |
| **`node:perf_hooks`** | `import { performance } from 'node:perf_hooks'` | `performance.now()`, `PerformanceObserver`, high-resolution nanosecond metrics | ✅ 100% Passing |

---

## 2. Web Standards API Surface

Beejs implements universal Web APIs standardized by W3C and WHATWG for cross-runtime code reuse:

| Web Standard API | Description | Global Availability |
| :--- | :--- | :---: |
| **`fetch()`** | Universal network request interface with streaming Request/Response bodies | ✅ `globalThis.fetch` |
| **`Headers`, `Request`, `Response`** | Fetch API foundation primitives | ✅ Available globally |
| **`URL`, `URLSearchParams`** | WHATWG URL parser and query string manager | ✅ Available globally |
| **`WebSocket`** | Standard real-time full-duplex client socket | ✅ Available globally |
| **`crypto.subtle` (Web Crypto)**| Standard cryptography: `digest`, `generateKey`, `encrypt`, `decrypt`, `sign` | ✅ `globalThis.crypto` |
| **`ReadableStream`, `WritableStream`** | WHATWG Streams standard for data pipelines | ✅ Available globally |
| **`CompressionStream`** | Native streaming `gzip` and `deflate` compression | ✅ Available globally |
| **`Blob`, `File`, `FormData`** | Binary data and multipart form payload containers | ✅ Available globally |
| **`structuredClone()`** | Deep-cloning for objects, TypedArrays, Sets, Maps, and circular graphs | ✅ Available globally |
| **`BroadcastChannel`** | Inter-context message bus | ✅ Available globally |
| **`TextEncoder`, `TextDecoder`** | High-performance UTF-8 byte stream string conversion | ✅ Available globally |
| **`Worker`** | Multi-threaded Web Worker execution | ✅ Available globally |

---

## 3. Practical Code Examples

### 1. Asynchronous File I/O (`node:fs/promises`)
```typescript
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const filePath = resolve('./config.json');

// Write structured JSON
await writeFile(filePath, JSON.stringify({ version: '1.0.0', runtime: 'Beejs' }, null, 2));

// Read back asynchronously
const content = await readFile(filePath, 'utf-8');
console.log('Parsed config:', JSON.parse(content));
```

### 2. Streaming Fetch Request
```typescript
const response = await fetch('https://httpbin.org/stream/3');

if (!response.body) {
  throw new Error('ReadableStream unavailable');
}

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  console.log('Stream chunk:', decoder.decode(value));
}
```

### 3. Web Crypto Digest (SHA-256)
```typescript
const message = 'Hello Beejs Security';
const data = new TextEncoder().encode(message);

const hashBuffer = await crypto.subtle.digest('SHA-256', data);
const hashHex = Array.from(new Uint8Array(hashBuffer))
  .map(b => b.toString(16).padStart(2, '0'))
  .join('');

console.log('SHA-256 Digest:', hashHex);
```

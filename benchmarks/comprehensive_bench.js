/**
 * Comprehensive JavaScript Runtime Benchmark Suite
 * Runs the EXACT same benchmark workload across Beejs, Node.js, and Bun.
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const EventEmitter = require('events');

const WARMUP_ITERS = 2;
const BENCH_ITERS = 5;

function runBench(name, fn) {
    // Warmup
    for (let i = 0; i < WARMUP_ITERS; i++) {
        fn();
    }
    
    // Benchmark
    const times = [];
    for (let i = 0; i < BENCH_ITERS; i++) {
        const start = performance.now();
        fn();
        const elapsed = performance.now() - start;
        times.push(elapsed);
    }
    
    // Calculate stats
    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);
    const opsSec = 1000 / avg;
    
    return { name, avgMs: avg, minMs: min, maxMs: max, opsSec };
}

// 1. Core Computation & JIT
function benchFibonacci() {
    function fib(n) {
        if (n <= 1) return n;
        let a = 0, b = 1;
        for (let i = 2; i <= n; i++) {
            let temp = a + b;
            a = b;
            b = temp;
        }
        return b;
    }
    let sum = 0;
    for (let i = 0; i < 500000; i++) {
        sum += fib(40);
    }
    return sum;
}

function benchPrimes() {
    const max = 100000;
    const flags = new Uint8Array(max + 1);
    let count = 0;
    for (let i = 2; i <= max; i++) {
        if (!flags[i]) {
            count++;
            for (let j = i * 2; j <= max; j += i) {
                flags[j] = 1;
            }
        }
    }
    return count;
}

function benchMatrixMultiply() {
    const N = 80;
    const A = new Float64Array(N * N);
    const B = new Float64Array(N * N);
    const C = new Float64Array(N * N);
    for (let i = 0; i < N * N; i++) {
        A[i] = i * 0.1;
        B[i] = i * 0.2;
    }
    for (let i = 0; i < N; i++) {
        for (let k = 0; k < N; k++) {
            const aik = A[i * N + k];
            for (let j = 0; j < N; j++) {
                C[i * N + j] += aik * B[k * N + j];
            }
        }
    }
    return C[0];
}

// 2. Object & Array Operations
function benchObjectCreation() {
    const arr = [];
    for (let i = 0; i < 50000; i++) {
        arr.push({
            id: i,
            name: 'user_' + i,
            active: (i % 2 === 0),
            meta: { role: 'admin', score: i * 1.5 },
            tags: ['tag1', 'tag2']
        });
    }
    let totalScore = 0;
    for (let i = 0; i < arr.length; i++) {
        totalScore += arr[i].meta.score;
    }
    return totalScore;
}

function benchArrayTransforms() {
    const data = [];
    for (let i = 0; i < 20000; i++) data.push(i);
    return data
        .filter(x => x % 2 === 0)
        .map(x => x * 3)
        .reduce((acc, x) => (acc ^ x), 0);
}

// 3. String & JSON Operations
function benchJsonSerialization() {
    const obj = {
        title: 'Benchmark Dataset',
        count: 1000,
        items: []
    };
    for (let i = 0; i < 200; i++) {
        obj.items.push({
            id: i,
            title: 'Item number ' + i,
            price: 19.99 + (i * 0.5),
            inStock: i % 3 !== 0,
            attributes: { color: 'blue', size: 'M', weight: 1.2 }
        });
    }
    let totalLen = 0;
    for (let i = 0; i < 50; i++) {
        const json = JSON.stringify(obj);
        totalLen += json.length;
        const parsed = JSON.parse(json);
        totalLen += parsed.items.length;
    }
    return totalLen;
}

function benchStringRegex() {
    const text = 'The quick brown fox jumps over the lazy dog. HTTP/1.1 200 OK. Content-Type: application/json; charset=utf-8\r\n\r\n';
    let count = 0;
    const regex = /[a-zA-Z0-9_-]+:\s*[^\r\n]+/g;
    for (let i = 0; i < 10000; i++) {
        const matches = text.match(regex);
        if (matches) count += matches.length;
        const replaced = text.replace(/quick/, 'slow').replace(/brown/, 'red');
        count += replaced.length;
    }
    return count;
}

// 4. Buffer Operations
function benchBufferOperations() {
    let total = 0;
    for (let i = 0; i < 1000; i++) {
        const buf = Buffer.alloc(16 * 1024);
        buf.fill(0xAA);
        const slice = buf.slice(100, 500);
        total += slice[0];
        const strBuf = Buffer.from('hello world from buffer benchmark ' + i);
        total += strBuf.length;
    }
    return total;
}

// 5. Crypto
function benchCryptoSha256() {
    const data = Buffer.alloc(16 * 1024, 'a');
    let hashLen = 0;
    for (let i = 0; i < 500; i++) {
        const hash = crypto.createHash('sha256').update(data).digest('hex');
        hashLen += hash.length;
    }
    return hashLen;
}

function benchCryptoRandomBytes() {
    let totalLen = 0;
    for (let i = 0; i < 500; i++) {
        const bytes = crypto.randomBytes(1024);
        totalLen += bytes.length;
    }
    return totalLen;
}

// 6. EventEmitter
function benchEventEmitter() {
    const ee = new EventEmitter();
    let counter = 0;
    const handler = (val) => { counter += val; };
    ee.on('event', handler);
    for (let i = 0; i < 50000; i++) {
        ee.emit('event', 1);
    }
    ee.removeListener('event', handler);
    return counter;
}

// 7. File System Sync I/O
function benchFsSync() {
    const tmpFile = path.join('/tmp', 'bench_io_' + process.pid + '.tmp');
    const content = 'X'.repeat(32 * 1024); // 32 KB
    for (let i = 0; i < 100; i++) {
        fs.writeFileSync(tmpFile, content);
        const readBack = fs.readFileSync(tmpFile);
        if (readBack.length !== content.length) {
            throw new Error('I/O mismatch');
        }
    }
    try { fs.unlinkSync(tmpFile); } catch (_) {}
}

const benchmarks = [
    { name: '1. JIT / Fibonacci (500k ops)', fn: benchFibonacci },
    { name: '2. JIT / Primes Sieve (100k)', fn: benchPrimes },
    { name: '3. JIT / Matrix Multiply (80x80)', fn: benchMatrixMultiply },
    { name: '4. Objects / Alloc & Property Access (50k)', fn: benchObjectCreation },
    { name: '5. Arrays / Filter-Map-Reduce (20k)', fn: benchArrayTransforms },
    { name: '6. JSON / Stringify & Parse (50 iters)', fn: benchJsonSerialization },
    { name: '7. String & RegExp (10k iters)', fn: benchStringRegex },
    { name: '8. Buffer / Alloc, Fill, Slice (1k x 16KB)', fn: benchBufferOperations },
    { name: '9. Crypto / SHA-256 (500 x 16KB)', fn: benchCryptoSha256 },
    { name: '10. Crypto / randomBytes (500 x 1KB)', fn: benchCryptoRandomBytes },
    { name: '11. EventEmitter / emit & listen (50k)', fn: benchEventEmitter },
    { name: '12. File System / Sync Read & Write (100 x 32KB)', fn: benchFsSync }
];

console.log('Runtime Platform:', process.platform, process.arch);
console.log('Runtime Version:', typeof process.versions === 'object' ? JSON.stringify(process.versions) : 'unknown');
console.log('Starting Benchmark Suite (' + benchmarks.length + ' workloads, ' + BENCH_ITERS + ' samples each)...\n');

const results = [];
for (const b of benchmarks) {
    const res = runBench(b.name, b.fn);
    results.push(res);
    console.log(res.name.padEnd(50) + ': ' + res.avgMs.toFixed(2).padStart(8) + ' ms  (' + res.opsSec.toFixed(1).padStart(7) + ' ops/s)');
}

console.log('\nSummary JSON:');
console.log(JSON.stringify(results, null, 2));

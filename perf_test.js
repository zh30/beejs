let start = Date.now();
let count = 0;
for (let i = 0; i < 100000; i++) {
    count += i * 2;
}
let duration = Date.now() - start;
console.log(`执行 ${count} 次运算，耗时 ${duration}ms`);
console.log(`性能: ${(100000 / duration * 1000).toFixed(0)} ops/sec`);

const iterations = 1000000;
console.log(`Running ${iterations} iterations...`);
const start = Date.now();
let sum = 0;
for (let i = 0; i < iterations; i++) {
    sum += Math.sqrt(i);
}
const end = Date.now();
const duration = end - start;
console.log(`Duration: ${duration}ms`);
console.log(`Ops/sec: ${(iterations / duration * 1000).toFixed(0)}`);

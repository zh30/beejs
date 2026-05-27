// Performance test for Beejs
const iterations = 100000;

console.log(`Running ${iterations} iterations...`);

let sum = 0;
const start = Date.now();

for (let i = 0; i < iterations; i++) {
    sum += Math.sqrt(i);
}

const end = Date.now();
const duration = end - start;
const opsPerSecond = (iterations / duration) * 1000;

console.log(`Completed ${iterations} iterations in ${duration}ms`);
console.log(`Operations per second: ${opsPerSecond.toFixed(2)}`);
console.log(`Sum: ${sum}`);

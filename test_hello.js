console.log('Hello from Beejs! 🎉');
console.log('Performance test starting...');

// Simple performance test
const iterations = 1000000;
let sum = 0;

const start = Date.now();
for (let i = 0; i < iterations; i++) {
    sum += Math.sqrt(i);
}
const end = Date.now();

console.log(`Completed ${iterations} iterations in ${end - start}ms`);
console.log(`Sum: ${sum}`);
console.log('Performance test completed! ✅');

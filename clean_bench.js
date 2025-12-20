// Clean benchmark - no console output
const iterations = 10000000;
let sum = 0;
const start = Date.now();

for (let i = 0; i < iterations; i++) {
    sum += Math.sqrt(i);
}

const end = Date.now();
const duration = end - start;
const opsPerSecond = (iterations / duration) * 1000;

// Only print at the end
console.log(duration + "," + Math.floor(opsPerSecond));

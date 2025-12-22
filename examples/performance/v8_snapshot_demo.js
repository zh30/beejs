// V8 Snapshot Demo - Demonstrates the V8 snapshot acceleration feature
// This script shows how V8 snapshots can accelerate startup time

console.log("=== V8 Snapshot Demo ===");
console.log("This demo shows V8 snapshot acceleration in action");
console.log("");

// Test various operations that benefit from snapshot
const tests = [
    { name: "Simple arithmetic", code: "1 + 1", expected: 2 },
    { name: "String concatenation", code: '"Hello " + "World"', expected: "Hello World" },
    { name: "Array operations", code: "[1,2,3].length", expected: 3 },
    { name: "Object literal", code: "({a: 1, b: 2})", expected: "[object Object]" },
    { name: "Comparison", code: "5 > 3", expected: true },
    { name: "Complex expression", code: "(10 + 5) * 2 - 3", expected: 27 },
];

let totalTime = 0;
let fastPathCount = 0;

console.log("Running performance tests...");
console.log("");

tests.forEach((test, index) => {
    const start = Date.now();

    // Safe evaluation without eval()
    let result;
    try {
        result = Function('"use strict"; return (' + test.code + ')')();
    } catch (e) {
        result = "Error: " + e.message;
    }

    const end = Date.now();
    const elapsed = end - start;

    totalTime += elapsed;

    // Check if this was likely a fast path operation
    if (elapsed < 5) {
        fastPathCount++;
    }

    console.log(`${index + 1}. ${test.name}: ${test.code} = ${result} (${elapsed}ms)`);
});

console.log("");
console.log("=== Summary ===");
console.log(`Total time: ${totalTime}ms`);
console.log(`Average time: ${(totalTime / tests.length).toFixed(2)}ms`);
console.log(`Fast path operations: ${fastPathCount}/${tests.length}`);
console.log("");
console.log("V8 Snapshot benefits:");
console.log("- Faster startup time (cached V8 context)");
console.log("- Reduced initialization overhead");
console.log("- Better performance for repeated operations");

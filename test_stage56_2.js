#!/usr/bin/env beejs
// Stage 56.2 Test Script - Execution Context

console.log("=== Beejs Stage 56.2 Test ===");
console.log("");

// Test __dirname and __filename
console.log("📂 __dirname:", typeof __dirname !== 'undefined' ? __dirname : 'NOT SET');
console.log("📄 __filename:", typeof __filename !== 'undefined' ? __filename : 'NOT SET');

// Test process.argv
if (typeof process !== 'undefined' && process.argv) {
    console.log("🔧 process.argv:", JSON.stringify(process.argv));
    console.log("   - Executable:", process.argv[0]);
    console.log("   - Script:", process.argv[1]);
    if (process.argv.length > 2) {
        console.log("   - Args:", process.argv.slice(2).join(", "));
    }
} else {
    console.log("❌ process.argv not available");
}

// Test process.cwd()
if (typeof process !== 'undefined' && typeof process.cwd === 'function') {
    console.log("📁 process.cwd():", process.cwd());
} else {
    console.log("❌ process.cwd() not available");
}

console.log("");
console.log("✅ Stage 56.2 Execution Context Test Complete!");

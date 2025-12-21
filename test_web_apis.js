// Test Blob API
console.log("=== Testing Blob API ===");

try {
    const blob = new Blob(['hello world']);
    console.log("✓ Blob created - size:", blob.size, "type:", blob.type);
} catch (e) {
    console.log("✗ Blob test failed:", e.message);
}

try {
    const file = new File(['content'], 'test.txt', { type: 'text/plain' });
    console.log("✓ File created - name:", file.name, "size:", file.size);
} catch (e) {
    console.log("✗ File test failed:", e.message);
}

// Test FormData API
console.log("\n=== Testing FormData API ===");

try {
    const form = new FormData();
    console.log("✓ FormData created");
    console.log("  Type:", typeof form);
    console.log("  Is object:", form instanceof Object);
} catch (e) {
    console.log("✗ FormData test failed:", e.message);
}

console.log("\n=== All tests completed ===");

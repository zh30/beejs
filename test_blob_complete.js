// Test Blob and File APIs

console.log("Testing Blob API...");

// Test 1: Create a basic Blob
try {
    const blob = new Blob(['hello world']);
    console.log("✓ Test 1 passed: Basic Blob created");
    console.log("  Size:", blob.size);
    console.log("  Type:", blob.type);
} catch (e) {
    console.log("✗ Test 1 failed:", e.message);
}

// Test 2: Create a Blob with MIME type
try {
    const blob = new Blob(['{"name": "test"}'], { type: 'application/json' });
    console.log("✓ Test 2 passed: Blob with MIME type created");
    console.log("  Size:", blob.size);
    console.log("  Type:", blob.type);
} catch (e) {
    console.log("✗ Test 2 failed:", e.message);
}

// Test 3: Create a File
try {
    const file = new File(['file content'], 'test.txt', { type: 'text/plain' });
    console.log("✓ Test 3 passed: File created");
    console.log("  Name:", file.name);
    console.log("  Size:", file.size);
    console.log("  Type:", file.type);
    console.log("  Last Modified:", file.lastModified);
} catch (e) {
    console.log("✗ Test 3 failed:", e.message);
}

// Test 4: Test Blob methods
try {
    const blob = new Blob(['Hello World']);
    const text = blob.text();
    console.log("✓ Test 4 passed: Blob.text() method works");
    console.log("  Text:", text);
} catch (e) {
    console.log("✗ Test 4 failed:", e.message);
}

console.log("\nAll tests completed!");

// Comprehensive Web API Test - Stage 53
console.log("=== Beejs Web API Comprehensive Test ===\n");

// Test 1: URL API
console.log("Test 1: URL API");
try {
    const url = new URL("https://example.com:8080/path/to/resource?query=value#section");
    console.log("✅ URL created successfully");
    console.log("  - href:", url.href);
    console.log("  - protocol:", url.protocol);
    console.log("  - host:", url.host);
    console.log("  - hostname:", url.hostname);
    console.log("  - port:", url.port);
    console.log("  - pathname:", url.pathname);
    console.log("  - search:", url.search);
    console.log("  - hash:", url.hash);
    console.log("  - origin:", url.origin);
} catch (e) {
    console.log("❌ URL API failed:", e.message);
}

console.log("\n" + "=".repeat(50) + "\n");

// Test 2: URLSearchParams API
console.log("Test 2: URLSearchParams API");
try {
    const params = new URLSearchParams("name=john&age=30&city=NYC");
    console.log("✅ URLSearchParams created successfully");
    console.log("  - name:", params.get("name"));
    console.log("  - age:", params.get("age"));
    console.log("  - city:", params.get("city"));
} catch (e) {
    console.log("❌ URLSearchParams API failed:", e.message);
}

console.log("\n" + "=".repeat(50) + "\n");

// Test 3: Fetch API
console.log("Test 3: Fetch API");
try {
    const response = fetch("https://httpbin.org/json");
    console.log("✅ Fetch API exists and callable");
    console.log("  - Response object:", typeof response);
    console.log("  - Response.ok:", response.ok);
    console.log("  - Response.status:", response.status);
} catch (e) {
    console.log("❌ Fetch API failed:", e.message);
}

console.log("\n" + "=".repeat(50) + "\n");

// Test 4: WebSocket API
console.log("Test 4: WebSocket API");
try {
    const ws = new WebSocket("ws://localhost:8080");
    console.log("✅ WebSocket created successfully");
    console.log("  - WebSocket readyState:", ws.readyState);

    // Test WebSocket methods
    ws.send("test message");
    console.log("  - send() method available");

    ws.close();
    console.log("  - close() method available");
} catch (e) {
    console.log("❌ WebSocket API failed:", e.message);
}

console.log("\n" + "=".repeat(50) + "\n");

// Test 5: Global Objects Check
console.log("Test 5: Global Objects");
console.log("✅ URL:", typeof URL);
console.log("✅ URLSearchParams:", typeof URLSearchParams);
console.log("✅ fetch:", typeof fetch);
console.log("✅ WebSocket:", typeof WebSocket);
console.log("✅ Headers:", typeof Headers);
console.log("✅ Request:", typeof Request);
console.log("✅ Response:", typeof Response);

console.log("\n" + "=".repeat(50) + "\n");
console.log("=== All Web API Tests Completed ===");

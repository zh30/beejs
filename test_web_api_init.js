// Test Web API initialization - Force standard path
// This code should trigger standard execution path

console.log("=== Testing Web API Initialization ===");

// Test 1: URL API
try {
    const url = new URL("https://api.example.com:8080/data/users?limit=10&offset=0#details");
    console.log("✅ URL API: " + url.href + " | Host: " + url.host + " | Protocol: " + url.protocol);
} catch (e) {
    console.log("❌ URL API failed: " + e.message);
}

// Test 2: URLSearchParams
try {
    const params = new URLSearchParams("name=Alice&age=25&city=San Francisco");
    console.log("✅ URLSearchParams: " + params.get("name") + " is " + params.get("age") + " years old");
} catch (e) {
    console.log("❌ URLSearchParams failed: " + e.message);
}

// Test 3: Fetch API
try {
    const fetchExists = typeof fetch === "function";
    console.log(fetchExists ? "✅ Fetch API: Available" : "❌ Fetch API: Missing");
} catch (e) {
    console.log("❌ Fetch API check failed: " + e.message);
}

// Test 4: WebSocket API
try {
    const wsExists = typeof WebSocket === "function";
    console.log(wsExists ? "✅ WebSocket API: Available" : "❌ WebSocket API: Missing");
} catch (e) {
    console.log("❌ WebSocket API check failed: " + e.message);
}

console.log("=== Web API Tests Completed ===");

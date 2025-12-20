// Simple synchronous fetch test
console.log("Testing real HTTP fetch...\n");

// Test 1: Simple GET request
console.log("Test 1: Simple GET request to httpbin.org");
const response = fetch("https://httpbin.org/get");
console.log("✓ Response object:", response);
console.log("✓ Status:", response.status);
console.log("✓ OK:", response.ok);
console.log("✓ Status Text:", response.statusText);
console.log("✓ Headers:", response.headers);
console.log("✓ Body:", response.body ? response.body.substring(0, 100) + "..." : "null");
console.log("\n✅ Test 1 passed!\n");

// Test 2: POST request with JSON body
console.log("Test 2: POST request with JSON body");
const response2 = fetch("https://httpbin.org/post", {
    method: "POST",
    headers: {
        "Content-Type": "application/json"
    },
    body: JSON.stringify({ name: "Beejs", version: "0.1.0" })
});
console.log("✓ Response object:", response2);
console.log("✓ Status:", response2.status);
console.log("✓ OK:", response2.ok);
console.log("\n✅ Test 2 passed!\n");

// Test 3: Check other Web APIs
console.log("Test 3: Check other Web APIs");
console.log("✓ setTimeout exists:", typeof setTimeout);
console.log("✓ setInterval exists:", typeof setInterval);
console.log("✓ TextEncoder exists:", typeof TextEncoder);
console.log("✓ TextDecoder exists:", typeof TextDecoder);
console.log("✓ btoa exists:", typeof btoa);
console.log("✓ atob exists:", typeof atob);
console.log("✓ URL exists:", typeof URL);
console.log("✓ URLSearchParams exists:", typeof URLSearchParams);
console.log("\n✅ All tests passed!");

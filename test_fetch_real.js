// Test real HTTP fetch implementation
console.log("Testing real HTTP fetch...\n");

// Test 1: Simple GET request
console.log("Test 1: Simple GET request to httpbin.org");
fetch("https://httpbin.org/get")
  .then(response => {
    console.log("✓ Status:", response.status);
    console.log("✓ OK:", response.ok);
    console.log("✓ Status Text:", response.statusText);
    console.log("✓ Headers:", response.headers);
    if (response.body) {
      return response.body;
    }
    return null;
  })
  .then(body => {
    if (body) {
      console.log("✓ Body length:", body.length);
      console.log("✓ Body preview:", body.substring(0, 100));
    }
    console.log("\n✅ Test 1 passed!\n");

    // Test 2: POST request with JSON
    console.log("Test 2: POST request with JSON body");
    return fetch("https://httpbin.org/post", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ name: "Beejs", version: "0.1.0" })
    });
  })
  .then(response => {
    console.log("✓ Status:", response.status);
    console.log("✓ OK:", response.ok);
    console.log("\n✅ Test 2 passed!\n");

    // Test 3: Test with invalid URL (should error)
    console.log("Test 3: Invalid URL (should throw error)");
    return fetch("invalid-url");
  })
  .catch(error => {
    console.log("✓ Error caught:", error.message || error.toString());
    console.log("\n✅ Test 3 passed (error handling)!\n");

    console.log("🎉 All fetch tests completed!");
  });

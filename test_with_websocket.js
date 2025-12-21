console.log("Testing WebSocket...");

try {
    // This should trigger WebSocket initialization
    const ws = new WebSocket('ws://example.com');
    console.log("WebSocket created:", ws);
    console.log("WebSocket URL:", ws.url);
    console.log("WebSocket readyState:", ws.readyState);
} catch (e) {
    console.log("Error:", e.message);
}

console.log("Test complete");

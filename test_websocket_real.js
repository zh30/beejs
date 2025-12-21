// WebSocket 真实网络连接测试
// 使用公共 WebSocket echo 服务器测试

console.log("=== WebSocket Real Network Connection Test ===\n");

// 测试 1: 构造函数检查
console.log("1. Testing WebSocket constructor...");
try {
    // 使用公共的 WebSocket echo 服务器
    const ws = new WebSocket("wss://echo.websocket.org");
    console.log("   ✅ WebSocket created");
    console.log("   URL:", ws.url);
    console.log("   Initial readyState:", ws.readyState, "(should be 0 = CONNECTING)");

    // 检查方法存在
    console.log("   send method:", typeof ws.send);
    console.log("   close method:", typeof ws.close);
    console.log("   _pollEvents method:", typeof ws._pollEvents);
    console.log("   _updateReadyState method:", typeof ws._updateReadyState);

    // 等待连接
    console.log("\n2. Waiting for connection...");
    let attempts = 0;
    const maxAttempts = 50; // 5 seconds max

    while (attempts < maxAttempts) {
        // 更新并获取 readyState
        const state = ws._updateReadyState();

        // 检查事件
        const events = ws._pollEvents();
        for (const event of events) {
            console.log("   Event received:", event.type);
            if (event.type === "open") {
                console.log("   ✅ Connection opened!");
            } else if (event.type === "message") {
                console.log("   ✅ Message received:", event.data);
            } else if (event.type === "error") {
                console.log("   ❌ Error:", event.message);
            } else if (event.type === "close") {
                console.log("   Connection closed, code:", event.code);
            }
        }

        if (state === 1) { // OPEN
            console.log("   ✅ WebSocket is OPEN (state=" + state + ")");
            break;
        } else if (state === 3) { // CLOSED
            console.log("   ❌ WebSocket CLOSED (state=" + state + ")");
            break;
        }

        // 简单的同步延迟
        const start = Date.now();
        while (Date.now() - start < 100) {
            // busy wait
        }
        attempts++;
    }

    if (ws._updateReadyState() === 1) {
        console.log("\n3. Sending message...");
        try {
            ws.send("Hello from Beejs!");
            console.log("   ✅ Message sent");

            // 等待 echo 响应
            console.log("\n4. Waiting for echo response...");
            let responseAttempts = 0;
            while (responseAttempts < 30) { // 3 seconds max
                const events = ws._pollEvents();
                for (const event of events) {
                    if (event.type === "message") {
                        console.log("   ✅ Echo received:", event.data);
                        if (event.data === "Hello from Beejs!") {
                            console.log("   ✅ Echo matches original message!");
                        }
                    }
                }

                // 简单的同步延迟
                const start = Date.now();
                while (Date.now() - start < 100) {
                    // busy wait
                }
                responseAttempts++;
            }
        } catch (e) {
            console.log("   ❌ Send failed:", e.message);
        }

        console.log("\n5. Closing connection...");
        try {
            ws.close();
            console.log("   ✅ Close called");

            // 等待关闭
            let closeAttempts = 0;
            while (closeAttempts < 10) {
                ws._updateReadyState();
                const events = ws._pollEvents();
                for (const event of events) {
                    if (event.type === "close") {
                        console.log("   ✅ Close event received");
                    }
                }

                if (ws.readyState === 3) {
                    console.log("   ✅ WebSocket CLOSED");
                    break;
                }

                const start = Date.now();
                while (Date.now() - start < 100) { }
                closeAttempts++;
            }
        } catch (e) {
            console.log("   ❌ Close failed:", e.message);
        }
    }

} catch (e) {
    console.log("   ❌ Error:", e.message || e);
}

console.log("\n=== Test Complete ===");

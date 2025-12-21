// WebSocket 调试脚本 - 检查内部状态

console.log("=== WebSocket 调试信息 ===\n");

// 检查构造函数
console.log("1. 检查 WebSocket 构造函数:");
console.log("  WebSocket:", WebSocket);
console.log("  typeof WebSocket:", typeof WebSocket);
console.log("  WebSocket.name:", WebSocket.name);

// 检查原型
console.log("\n2. 检查 WebSocket.prototype:");
console.log("  WebSocket.prototype:", WebSocket.prototype);
console.log("  typeof WebSocket.prototype:", typeof WebSocket.prototype);

// 检查实例
console.log("\n3. 创建 WebSocket 实例:");
const ws = new WebSocket('ws://echo.websocket.org');
console.log("  ws:", ws);
console.log("  ws.constructor:", ws.constructor);
console.log("  ws.__proto__:", ws.__proto__);
console.log("  Object.getPrototypeOf(ws):", Object.getPrototypeOf(ws));

// 详细检查属性
console.log("\n4. 检查实例属性:");
const properties = Object.getOwnPropertyNames(ws);
console.log("  自身属性:", properties);

const protoProperties = Object.getOwnPropertyNames(WebSocket.prototype);
console.log("  原型属性:", protoProperties);

// 检查方法
console.log("\n5. 检查方法:");
console.log("  ws.send:", ws.send);
console.log("  ws.close:", ws.close);
console.log("  ws.addEventListener:", ws.addEventListener);
console.log("  ws.removeEventListener:", ws.removeEventListener);

// 检查常量
console.log("\n6. 检查常量:");
console.log("  WebSocket.CONNECTING:", WebSocket.CONNECTING);
console.log("  WebSocket.OPEN:", WebSocket.OPEN);
console.log("  WebSocket.CLOSING:", WebSocket.CLOSING);
console.log("  WebSocket.CLOSED:", WebSocket.CLOSED);

// 尝试调用方法
console.log("\n7. 尝试调用 send 方法:");
try {
    ws.send("test");
    console.log("  ✅ send 调用成功");
} catch (e) {
    console.log("  ❌ send 调用失败:", e.message);
}

console.log("\n=== 调试完成 ===");

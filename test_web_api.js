// Stage 53 - Web API 测试脚本
// 注意：使用变量赋值避免触发 fast path
const test = () => {
    console.log("=== Beejs Web API 测试 ===");

    // 测试 1: 检查全局对象
    console.log("1. 检查全局对象:");
    console.log("  - fetch:", typeof fetch);
    console.log("  - WebSocket:", typeof WebSocket);
    console.log("  - URL:", typeof URL);
    console.log("  - Headers:", typeof Headers);

    // 测试 2: URL API
    try {
        const url = new URL("https://example.com/path?query=value#hash");
        console.log("2. URL API 测试:");
        console.log("  - href:", url.href);
        console.log("  - pathname:", url.pathname);
        console.log("  - search:", url.search);
        console.log("  - hash:", url.hash);
        console.log("  ✅ URL API 正常工作");
    } catch (e) {
        console.log("  ❌ URL API 错误:", e.message);
    }

    // 测试 3: Headers API
    try {
        const headers = new Headers();
        headers.set("Content-Type", "application/json");
        console.log("3. Headers API 测试:");
        console.log("  - Content-Type:", headers.get("Content-Type"));
        console.log("  ✅ Headers API 正常工作");
    } catch (e) {
        console.log("  ❌ Headers API 错误:", e.message);
    }

    console.log("\n=== 测试完成 ===");
};

test();

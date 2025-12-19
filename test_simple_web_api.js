// 简单的 Web API 测试
const test = () => {
    console.log("Testing URL API...");
    try {
        const url = new URL("https://example.com/path");
        console.log("URL created:", url.href);
        console.log("✅ URL API works!");
    } catch (e) {
        console.log("❌ URL API failed:", e.message);
    }
};

test();

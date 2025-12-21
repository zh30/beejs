// WebSocket 集成测试 - 真实网络连接测试
// 测试与真实 WebSocket 服务器的通信

console.log('=== WebSocket 集成测试开始 ===\n');

// 测试 1: 创建 WebSocket 实例并连接
console.log('测试 1: 连接到 WebSocket 服务器');
const ws = new WebSocket('ws://echo.websocket.org');

console.log('WebSocket 实例创建成功');
console.log('- URL:', ws.url);
console.log('- readyState:', ws.readyState, '(0=CONNECTING)');

// 测试 2: 设置事件监听器
console.log('\n测试 2: 设置事件监听器');

let openCount = 0;
let messageCount = 0;
let closeCount = 0;

ws.onopen = function() {
    openCount++;
    console.log('✓ onopen 事件触发 (第', openCount, '次)');
    console.log('  - readyState:', ws.readyState, '(1=OPEN)');

    // 连接成功后发送测试消息
    setTimeout(() => {
        console.log('\n发送测试消息...');
        ws.send('Hello from Beejs WebSocket!');
    }, 100);
};

ws.onmessage = function(event) {
    messageCount++;
    console.log('✓ onmessage 事件触发 (第', messageCount, '次)');
    console.log('  - 数据:', event.data);
    console.log('  - 数据类型:', typeof event.data);
};

ws.onclose = function(event) {
    closeCount++;
    console.log('✓ onclose 事件触发 (第', closeCount, '次)');
    console.log('  - code:', event.code);
    console.log('  - reason:', event.reason);
    console.log('  - wasClean:', event.wasClean);
};

ws.onerror = function(error) {
    console.log('⚠ onerror 事件触发:', error);
};

// 测试 3: 检查连接状态变化
console.log('\n测试 3: 监控连接状态变化');
setTimeout(() => {
    console.log('当前 readyState:', ws.readyState);
    if (ws.readyState === 1) { // OPEN
        console.log('✓ 连接已建立');
    }
}, 500);

// 测试 4: 发送消息并等待响应
setTimeout(() => {
    console.log('\n测试 4: 发送更多消息');
    ws.send('Message 1: ' + new Date().toISOString());
    ws.send('Message 2: Testing WebSocket');
    ws.send('Message 3: End');
}, 1000);

// 测试 5: 关闭连接
setTimeout(() => {
    console.log('\n测试 5: 关闭连接');
    ws.close(1000, 'Test complete');
}, 3000);

// 测试 6: 最终状态检查
setTimeout(() => {
    console.log('\n测试 6: 最终状态');
    console.log('- 最终 readyState:', ws.readyState, '(3=CLOSED)');
    console.log('- 事件触发统计:');
    console.log('  - onopen:', openCount, '次');
    console.log('  - onmessage:', messageCount, '次');
    console.log('  - onclose:', closeCount, '次');

    console.log('\n=== WebSocket 集成测试完成 ===');
    console.log('如果看到以上事件触发，说明 WebSocket 集成工作正常！');
}, 4000);

// 备用错误处理
setTimeout(() => {
    console.log('\n⚠ 测试超时，如果未看到连接事件，请检查网络连接');
    console.log('注意: 需要网络连接才能完成此测试');
}, 6000);

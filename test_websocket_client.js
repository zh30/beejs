// WebSocket 客户端功能测试
// 按照 TDD 原则，先编写测试，再实现功能

// 测试 1: WebSocket 构造函数存在
console.log('测试 1: WebSocket 构造函数存在');
console.log('typeof WebSocket:', typeof WebSocket);
if (typeof WebSocket === 'function') {
    console.log('✅ WebSocket 构造函数存在');
} else {
    console.error('❌ WebSocket 应该是函数');
}

// 测试 2: 创建 WebSocket 实例
console.log('\n测试 2: 创建 WebSocket 实例');
const ws = new WebSocket('ws://echo.websocket.org');
console.log('ws 实例创建成功:', ws);
console.log('ws.url:', ws.url);
console.log('ws.readyState:', ws.readyState); // 0 = CONNECTING

if (ws.url === 'ws://echo.websocket.org') {
    console.log('✅ URL 正确设置');
} else {
    console.error('❌ URL 应该正确设置');
}

if (ws.readyState === 0) {
    console.log('✅ 初始 readyState 正确 (CONNECTING = 0)');
} else {
    console.error('❌ 初始 readyState 应该是 CONNECTING (0)');
}

// 测试 3: WebSocket 属性检查
console.log('\n测试 3: WebSocket 属性检查');
console.log('ws.bufferedAmount:', ws.bufferedAmount);
console.log('ws.extensions:', ws.extensions);
console.log('ws.protocol:', ws.protocol);
console.log('ws.binaryType:', ws.binaryType);

if (ws.bufferedAmount === 0) {
    console.log('✅ bufferedAmount 初始为 0');
} else {
    console.error('❌ bufferedAmount 初始应该为 0');
}

if (ws.extensions === '') {
    console.log('✅ extensions 初始为空字符串');
} else {
    console.error('❌ extensions 初始应该为空字符串');
}

if (ws.protocol === '') {
    console.log('✅ protocol 初始为空字符串');
} else {
    console.error('❌ protocol 初始应该为空字符串');
}

if (ws.binaryType === 'arraybuffer') {
    console.log('✅ binaryType 初始为 "arraybuffer"');
} else {
    console.error('❌ binaryType 初始应该为 "arraybuffer"');
}

// 测试 4: 事件处理器属性存在
console.log('\n测试 4: 事件处理器属性存在');
console.log('ws.onopen:', ws.onopen);
console.log('ws.onmessage:', ws.onmessage);
console.log('ws.onclose:', ws.onclose);
console.log('ws.onerror:', ws.onerror);

if (ws.onopen === null || typeof ws.onopen === 'function') {
    console.log('✅ onopen 属性存在');
} else {
    console.error('❌ onopen 应该是函数或 null, 但得到:', ws.onopen);
}

if (ws.onmessage === null || typeof ws.onmessage === 'function') {
    console.log('✅ onmessage 属性存在');
} else {
    console.error('❌ onmessage 应该是函数或 null, 但得到:', ws.onmessage);
}

if (ws.onclose === null || typeof ws.onclose === 'function') {
    console.log('✅ onclose 属性存在');
} else {
    console.error('❌ onclose 应该是函数或 null, 但得到:', ws.onclose);
}

if (ws.onerror === null || typeof ws.onerror === 'function') {
    console.log('✅ onerror 属性存在');
} else {
    console.error('❌ onerror 应该是函数或 null, 但得到:', ws.onerror);
}

// 测试 5: 方法存在
console.log('\n测试 5: WebSocket 方法存在');
console.log('typeof ws.send:', typeof ws.send);
console.log('typeof ws.close:', typeof ws.close);
console.log('typeof ws.addEventListener:', typeof ws.addEventListener);
console.log('typeof ws.removeEventListener:', typeof ws.removeEventListener);

if (typeof ws.send === 'function') {
    console.log('✅ send 方法存在');
} else {
    console.error('❌ send 应该是函数');
}

if (typeof ws.close === 'function') {
    console.log('✅ close 方法存在');
} else {
    console.error('❌ close 应该是函数');
}

if (typeof ws.addEventListener === 'function') {
    console.log('✅ addEventListener 方法存在');
} else {
    console.error('❌ addEventListener 应该是函数');
}

if (typeof ws.removeEventListener === 'function') {
    console.log('✅ removeEventListener 方法存在');
} else {
    console.error('❌ removeEventListener 应该是函数');
}

// 测试 6: 带有协议的 WebSocket
console.log('\n测试 6: 带有协议的 WebSocket');
const wsWithProtocol = new WebSocket('ws://echo.websocket.org', 'chat');
console.log('wsWithProtocol 实例创建成功:', wsWithProtocol);
console.log('wsWithProtocol.url:', wsWithProtocol.url);

if (wsWithProtocol.url === 'ws://echo.websocket.org') {
    console.log('✅ 带有协议的 WebSocket 创建成功');
} else {
    console.error('❌ URL 应该正确设置');
}

// 测试 7: ReadyState 常量（如果存在）
console.log('\n测试 7: ReadyState 常量');
if (typeof WebSocket !== 'undefined') {
    // WebSocket 常量可能在构造函数上
    console.log('WebSocket.CONNECTING:', WebSocket.CONNECTING);
    console.log('WebSocket.OPEN:', WebSocket.OPEN);
    console.log('WebSocket.CLOSING:', WebSocket.CLOSING);
    console.log('WebSocket.CLOSED:', WebSocket.CLOSED);

    if (WebSocket.CONNECTING !== undefined) {
        if (WebSocket.CONNECTING === 0 && WebSocket.OPEN === 1 &&
            WebSocket.CLOSING === 2 && WebSocket.CLOSED === 3) {
            console.log('✅ ReadyState 常量正确');
        } else {
            console.error('❌ ReadyState 常量值不正确');
        }
    } else {
        console.log('⚠️ ReadyState 常量未定义（需要在构造函数上）');
    }
} else {
    console.error('❌ WebSocket 未定义');
}

// 测试 8: 错误处理 - 无效 URL
console.log('\n测试 8: 错误处理 - 无效 URL');
try {
    const wsInvalid = new WebSocket('');
    console.log('⚠️ 无效 URL 没有抛出错误（可能是预期的）');
} catch (e) {
    console.log('✅ 捕获到错误:', e.message);
}

// 测试 9: 事件监听器
console.log('\n测试 9: 事件监听器');
let openEventFired = false;
ws.onopen = function() {
    console.log('onopen 事件触发');
    openEventFired = true;
};

// 测试 10: send 方法（基础验证）
console.log('\n测试 10: send 方法基础验证');
try {
    // 注意：这里可能失败，因为连接尚未建立
    ws.send('test message');
    console.log('✅ send 方法调用成功');
} catch (e) {
    console.log('⚠️ send 调用失败（预期，因为没有真实连接）:', e.message);
}

console.log('\n=== WebSocket 客户端测试完成 ===');
console.log('注意: 由于没有真实服务器，部分网络相关测试可能失败');

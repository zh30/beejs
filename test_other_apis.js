// Test if other Web APIs work
try {
    const url = new URL('https://example.com');
    console.log('✅ URL API works:', url.href);
} catch (e) {
    console.log('❌ URL API failed:', e.message);
}

try {
    const encoder = new TextEncoder();
    console.log('✅ TextEncoder API works');
} catch (e) {
    console.log('❌ TextEncoder API failed:', e.message);
}

try {
    const ws = new WebSocket('ws://echo.websocket.org');
    console.log('✅ WebSocket API works');
} catch (e) {
    console.log('❌ WebSocket API failed:', e.message);
}

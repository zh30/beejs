// Debug Blob API initialization
console.log('=== Debugging Blob API ===\n');

// Check if Web API context exists
console.log('Checking global object...');
const global_keys = Object.keys(globalThis);
console.log('Global keys count:', global_keys.length);

// Check for specific APIs that should be initialized
const apis = ['fetch', 'WebSocket', 'URL', 'TextEncoder', 'Blob', 'File', 'FormData'];
console.log('\nAPI Availability:');
apis.forEach(api => {
    const exists = typeof globalThis[api] !== 'undefined';
    console.log(`  ${api}: ${exists ? '✅' : '❌'}`);
});

// Check performance object (another Stage 74 API)
console.log('\nChecking Performance API:');
if (typeof performance !== 'undefined') {
    console.log('✅ Performance API exists');
    try {
        const now = performance.now();
        console.log('  performance.now():', now);
    } catch (e) {
        console.log('  ❌ Error calling performance.now():', e.message);
    }
} else {
    console.log('❌ Performance API does not exist');
}

// Try to access Blob directly
console.log('\nTrying to access Blob:');
try {
    console.log('typeof Blob:', typeof Blob);
    console.log('Blob:', Blob);
} catch (e) {
    console.log('Error:', e.message);
}

// Try to access File directly
console.log('\nTrying to access File:');
try {
    console.log('typeof File:', typeof File);
    console.log('File:', File);
} catch (e) {
    console.log('Error:', e.message);
}

console.log('\n=== Debug Complete ===');

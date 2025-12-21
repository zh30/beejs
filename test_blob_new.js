// Test with new Blob to trigger Web API path
console.log('=== Testing new Blob ===');

try {
    const blob = new Blob(['Hello']);
    console.log('✅ Blob created:', blob.size, blob.type);
} catch (e) {
    console.log('❌ Error:', e.message);
}

console.log('=== Test Complete ===');

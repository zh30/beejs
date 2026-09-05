const assert = require('node:assert');
const http2 = require('node:http2');

assert.ok(http2.constants);
assert.strictEqual(http2.constants.HTTP2_HEADER_STATUS, ':status');
assert.strictEqual(http2.constants.HTTP2_HEADER_METHOD, ':method');

assert.strictEqual(typeof http2.getDefaultSettings, 'function');
assert.strictEqual(typeof http2.createServer, 'function');
assert.strictEqual(typeof http2.createSecureServer, 'function');

const server = http2.createServer();
assert.ok(server);

console.log('CONFORMANCE_PASS');

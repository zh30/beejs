const assert = require('node:assert');
const http = require('node:http');

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('ok');
});

assert.strictEqual(typeof server.setTimeout, 'function');
assert.strictEqual(typeof server.ref, 'function');
assert.strictEqual(typeof server.unref, 'function');

server.listen({ port: 0, host: '127.0.0.1' }, () => {
  const addr = server.address();
  assert.ok(addr);
  assert.strictEqual(typeof addr.port, 'number');
  assert.ok(addr.port > 0);
  assert.strictEqual(addr.address, '127.0.0.1');

  if (typeof server.close === 'function') {
    server.close();
  }
  console.log('CONFORMANCE_PASS');
});

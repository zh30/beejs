const net = require('net');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof net.createServer === 'function', 'net.createServer exists');

const server = net.createServer();
let denied = false;
try {
  server.listen({ port: 0, host: '127.0.0.1' });
} catch (error) {
  const text = String(error && error.message ? error.message : error).toLowerCase();
  denied = text.includes('permission') || text.includes('denied');
}

assert(denied, 'net.Server.listen must be denied under --deny-net');
console.log('CONFORMANCE_PASS');

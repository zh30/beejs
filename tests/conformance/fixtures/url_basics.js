// url / WHATWG URL conformance smoke fixture
const url = require('url');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof url.URL === 'function' || typeof URL === 'function', 'URL ctor');
const U = url.URL || URL;
const u = new U('https://example.com:8080/path?q=1#hash');
assert(u.hostname === 'example.com', 'hostname');
assert(u.pathname === '/path', 'pathname');
assert(typeof url.fileURLToPath === 'function', 'fileURLToPath');
assert(typeof url.pathToFileURL === 'function', 'pathToFileURL');

const p = url.fileURLToPath(url.pathToFileURL('/tmp/x').href);
assert(typeof p === 'string' && p.includes('x'), 'roundtrip path');

console.log('CONFORMANCE_PASS');

// Buffer conformance smoke fixture
const { Buffer } = require('buffer');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof Buffer === 'function' || typeof Buffer === 'object', 'Buffer exists');
assert(typeof Buffer.from === 'function', 'Buffer.from');
assert(typeof Buffer.alloc === 'function', 'Buffer.alloc');

const b = Buffer.from('hello');
assert(b.length === 5, 'length');
const s = b.toString('utf8');
assert(s === 'hello', 'toString utf8, got: ' + JSON.stringify(s));
assert(Buffer.isBuffer(b) === true, 'isBuffer');

console.log('CONFORMANCE_PASS');

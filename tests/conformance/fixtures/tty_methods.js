const assert = require('node:assert');
const tty = require('node:tty');

assert.strictEqual(typeof tty.isatty, 'function');
assert.strictEqual(tty.isatty(-1), false);

assert.strictEqual(typeof tty.WriteStream, 'function');
assert.strictEqual(typeof tty.ReadStream, 'function');

console.log('CONFORMANCE_PASS');

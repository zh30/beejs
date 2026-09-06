const assert = require('assert');
const { performance } = require('perf_hooks');

assert.strictEqual(typeof performance.now, 'function');
const t1 = performance.now();
assert(typeof t1 === 'number');
assert(t1 >= 0);

assert.strictEqual(typeof performance.mark, 'function');
assert.strictEqual(typeof performance.measure, 'function');

performance.mark('start');
performance.mark('end');
performance.measure('test-measure', 'start', 'end');

console.log('CONFORMANCE_PASS');

// assert module conformance smoke fixture
let assert;
try {
  assert = require('assert');
} catch (e) {
  throw new Error('assert builtin missing: ' + e);
}

assert.strictEqual(1 + 1, 2);
assert.ok(true);
assert.deepStrictEqual({ a: 1 }, { a: 1 });

let threw = false;
try {
  assert.strictEqual(1, 2);
} catch (e) {
  threw = true;
}
if (!threw) throw new Error('assert.strictEqual should throw');

console.log('CONFORMANCE_PASS');

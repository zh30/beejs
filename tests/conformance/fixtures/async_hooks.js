const assert = require('node:assert');
const ah = require('node:async_hooks');

// 1. AsyncResource
const resource = new ah.AsyncResource('TestResource');
assert.strictEqual(typeof resource.asyncId(), 'number');
assert.strictEqual(typeof resource.triggerAsyncId(), 'number');

let executed = false;
resource.runInAsyncScope(() => {
  executed = true;
});
assert.strictEqual(executed, true);

const fn = resource.bind((a, b) => a * b);
assert.strictEqual(fn(6, 7), 42);

// 2. AsyncLocalStorage
const als = new ah.AsyncLocalStorage();
assert.strictEqual(als.getStore(), undefined);

als.run({ user: 'alice' }, () => {
  assert.deepStrictEqual(als.getStore(), { user: 'alice' });
  als.run({ user: 'bob' }, () => {
    assert.deepStrictEqual(als.getStore(), { user: 'bob' });
  });
  assert.deepStrictEqual(als.getStore(), { user: 'alice' });
});

assert.strictEqual(als.getStore(), undefined);

// 3. executionAsyncId & triggerAsyncId
assert.strictEqual(typeof ah.executionAsyncId(), 'number');
assert.strictEqual(typeof ah.triggerAsyncId(), 'number');

console.log('CONFORMANCE_PASS');

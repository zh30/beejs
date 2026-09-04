const assert = require('assert');
const util = require('util');

// 1. util.format
const formatted = util.format('Hello %s, count: %d', 'world', 42);
assert.strictEqual(formatted, 'Hello world, count: 42');

// 2. util.inspect
const inspected = util.inspect({ a: 1, b: 'two' });
assert.strictEqual(typeof inspected, 'string');
assert(inspected.includes('a'));

// 3. util.types
assert.strictEqual(util.types.isDate(new Date()), true);
assert.strictEqual(util.types.isDate('not a date'), false);

// 4. type checks
assert.strictEqual(util.isArray([1, 2]), true);
assert.strictEqual(util.isArray({}), false);
assert.strictEqual(util.isBoolean(true), true);
assert.strictEqual(util.isNull(null), true);
assert.strictEqual(util.isNumber(123), true);
assert.strictEqual(util.isString('abc'), true);

console.log('CONFORMANCE_PASS');

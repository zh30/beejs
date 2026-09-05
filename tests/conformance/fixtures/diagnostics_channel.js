const assert = require('node:assert');
const dc = require('node:diagnostics_channel');

// 1. channel and pub/sub
const ch = dc.channel('test.channel');
assert.strictEqual(ch.name, 'test.channel');
assert.strictEqual(ch.hasSubscribers, false);

let received = null;
function sub(msg, name) {
  received = { msg, name };
}

ch.subscribe(sub);
assert.strictEqual(ch.hasSubscribers, true);
assert.strictEqual(dc.hasSubscribers('test.channel'), true);

ch.publish({ foo: 'bar' });
assert.deepStrictEqual(received, { msg: { foo: 'bar' }, name: 'test.channel' });

ch.unsubscribe(sub);
assert.strictEqual(ch.hasSubscribers, false);

// 2. tracingChannel
const tc = dc.tracingChannel('test.trace');
assert.ok(tc.start);
assert.ok(tc.end);
assert.ok(tc.error);

let traced = false;
const res = tc.traceSync((x, y) => {
  traced = true;
  return x + y;
}, {}, null, 10, 20);

assert.strictEqual(traced, true);
assert.strictEqual(res, 30);

console.log('CONFORMANCE_PASS');

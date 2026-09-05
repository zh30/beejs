const assert = require('node:assert');
const events = require('node:events');

async function test() {
  const ee = new events.EventEmitter();

  setTimeout(() => {
    ee.emit('ping', 123, 'hello');
  }, 10);

  const [num, str] = await events.once(ee, 'ping');
  assert.strictEqual(num, 123);
  assert.strictEqual(str, 'hello');

  console.log('CONFORMANCE_PASS');
}

test().catch(err => {
  console.error(err);
  process.exit(1);
});

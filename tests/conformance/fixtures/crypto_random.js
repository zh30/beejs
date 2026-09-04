// crypto.randomBytes and crypto.randomUUID conformance fixture
const crypto = require('crypto');
const assert = require('assert');

const bytes = crypto.randomBytes(16);
assert.strictEqual(bytes.length, 16, 'randomBytes(16) should return 16 bytes');

const bytes32 = crypto.randomBytes(32);
assert.strictEqual(bytes32.length, 32, 'randomBytes(32) should return 32 bytes');

if (typeof crypto.randomUUID === 'function') {
  const uuid = crypto.randomUUID();
  assert.strictEqual(typeof uuid, 'string');
  assert.strictEqual(uuid.length, 36);
  assert.ok(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(uuid));
}

console.log('CONFORMANCE_PASS');

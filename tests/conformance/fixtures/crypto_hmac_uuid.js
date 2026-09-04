const assert = require('assert');
const crypto = require('crypto');

// 1. randomUUID
const uuid1 = crypto.randomUUID();
const uuid2 = crypto.randomUUID();
assert.strictEqual(typeof uuid1, 'string');
assert.strictEqual(uuid1.length, 36);
assert.ok(uuid1 !== uuid2);

// 2. createHmac
const hmac = crypto.createHmac('sha256', 'secret-key');
hmac.update('hello world');
const digest = hmac.digest('hex');
assert.strictEqual(typeof digest, 'string');
assert.strictEqual(digest.length, 64);

// 3. timingSafeEqual with Buffer
const b1 = Buffer.from('abcdef123456');
const b2 = Buffer.from('abcdef123456');
const b3 = Buffer.from('abcdef654321');
assert.strictEqual(crypto.timingSafeEqual(b1, b2), true);
assert.strictEqual(crypto.timingSafeEqual(b1, b3), false);

console.log('CONFORMANCE_PASS');

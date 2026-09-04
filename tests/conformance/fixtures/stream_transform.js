// stream.Transform conformance fixture
const stream = require('stream');
const assert = require('assert');

const Transform = stream.Transform;
assert.strictEqual(typeof Transform, 'function', 'stream.Transform must be a constructor');

const t = new Transform({
  transform(chunk, encoding, callback) {
    callback(null, chunk.toString().toUpperCase());
  }
});

assert.ok(t instanceof Transform, 't must be an instance of Transform');
assert.strictEqual(typeof t.pipe, 'function', 't.pipe must be a function');
assert.strictEqual(typeof t.write, 'function', 't.write must be a function');

console.log('CONFORMANCE_PASS');

const assert = require('node:assert');
const stream = require('node:stream');

const r = new stream.Readable({
  read() {
    this.push('chunk');
    this.push(null);
  }
});

let cbCalled = false;
stream.finished(r, (err) => {
  assert.ifError(err);
  cbCalled = true;
});

r.on('data', () => {});
r.on('end', () => {
  assert.strictEqual(cbCalled, true);
  console.log('CONFORMANCE_PASS');
});

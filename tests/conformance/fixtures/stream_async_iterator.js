const assert = require('node:assert');
const { Readable } = require('node:stream');

async function test() {
  const chunks = ['foo', 'bar', 'baz'];
  let idx = 0;
  const r = new Readable({
    read() {
      if (idx < chunks.length) {
        this.push(chunks[idx++]);
      } else {
        this.push(null);
      }
    }
  });

  const received = [];
  for await (const chunk of r) {
    received.push(String(chunk));
  }

  assert.deepStrictEqual(received, ['foo', 'bar', 'baz']);
  console.log('CONFORMANCE_PASS');
}

test().catch(err => {
  console.error(err);
  process.exit(1);
});

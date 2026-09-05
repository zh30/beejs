const assert = require('node:assert');
const { Readable } = require('node:stream');

async function test() {
  const source = ['alpha', 'beta', 'gamma'];
  const stream = Readable.from(source);

  const out = [];
  for await (const chunk of stream) {
    out.push(String(chunk));
  }

  assert.deepStrictEqual(out, ['alpha', 'beta', 'gamma']);
  console.log('CONFORMANCE_PASS');
}

test().catch(err => {
  console.error(err);
  process.exit(1);
});

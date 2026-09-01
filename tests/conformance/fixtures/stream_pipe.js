const stream = require('stream');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof stream.passThrough === 'function' || typeof stream.PassThrough === 'function', 'PassThrough exists');

const create = stream.PassThrough || stream.passThrough;
let output = '';
const src = typeof create === 'function' ? create() : new create();
const dest = typeof create === 'function' ? create() : new create();

dest.on('data', (chunk) => {
  output += chunk;
});

const piped = src.pipe(dest);
assert(piped === dest, 'pipe returns destination');

src.write('hello');
src.end();

assert(output === 'hello', 'piped data, got: ' + JSON.stringify(output));
console.log('CONFORMANCE_PASS');

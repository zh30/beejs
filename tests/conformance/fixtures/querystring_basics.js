const qs = require('querystring');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

const parsed = qs.parse('a=1&b=2');
assert(parsed.a === '1', 'parse a');
assert(parsed.b === '2', 'parse b');
const s = qs.stringify({ x: 1, y: 2 });
assert(s.includes('x=1') && s.includes('y=2'), 'stringify');

console.log('CONFORMANCE_PASS');

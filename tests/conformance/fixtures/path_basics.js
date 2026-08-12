// Node path conformance smoke fixture
const path = require('path');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(path.join('a', 'b', 'c') === 'a/b/c' || path.join('a', 'b', 'c').includes('b'), 'join');
assert(path.basename('/tmp/foo.txt') === 'foo.txt', 'basename');
assert(path.extname('foo.txt') === '.txt', 'extname');
assert(typeof path.resolve('.') === 'string', 'resolve');
assert(path.isAbsolute('/tmp') === true, 'isAbsolute unix');
assert(path.isAbsolute('tmp') === false, 'isAbsolute relative');

console.log('CONFORMANCE_PASS');

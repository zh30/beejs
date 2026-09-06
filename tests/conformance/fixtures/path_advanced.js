const assert = require('assert');
const path = require('path');

// relative
const rel = path.relative('/data/orandea/test/aaa', '/data/orandea/impl/bbb');
assert.strictEqual(rel, '../../impl/bbb');

// parse & format
const parsed = path.parse('/home/user/dir/file.txt');
assert.strictEqual(parsed.root, '/');
assert.strictEqual(parsed.dir, '/home/user/dir');
assert.strictEqual(parsed.base, 'file.txt');
assert.strictEqual(parsed.ext, '.txt');
assert.strictEqual(parsed.name, 'file');

const formatted = path.format(parsed);
assert.strictEqual(formatted, '/home/user/dir/file.txt');

// isAbsolute
assert.strictEqual(path.isAbsolute('/foo/bar'), true);
assert.strictEqual(path.isAbsolute('qux/'), false);

console.log('CONFORMANCE_PASS');

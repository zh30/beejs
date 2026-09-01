const os = require('os');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof os.platform === 'function', 'os.platform');
assert(typeof os.arch === 'function', 'os.arch');
assert(typeof os.tmpdir === 'function', 'os.tmpdir');
assert(typeof os.homedir === 'function', 'os.homedir');

const platform = os.platform();
assert(typeof platform === 'string' && platform.length > 0, 'platform string');
assert(typeof os.arch() === 'string', 'arch string');
assert(typeof os.tmpdir() === 'string', 'tmpdir string');

console.log('CONFORMANCE_PASS');

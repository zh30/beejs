const assert = require('assert');
const cp = require('child_process');

let threwExec = false;
try {
  cp.execSync('echo should_fail');
} catch (e) {
  threwExec = true;
  assert(e.message.includes('Permission denied') || e.message.includes('permission') || e.message.includes('sandbox'));
}
assert.strictEqual(threwExec, true, 'execSync should throw under --sandbox');

let threwSpawn = false;
try {
  cp.spawnSync('echo', ['should_fail']);
} catch (e) {
  threwSpawn = true;
  assert(e.message.includes('Permission denied') || e.message.includes('permission') || e.message.includes('sandbox'));
}
assert.strictEqual(threwSpawn, true, 'spawnSync should throw under --sandbox');

console.log('CONFORMANCE_PASS');

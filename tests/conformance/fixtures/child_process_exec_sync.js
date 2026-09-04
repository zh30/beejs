const assert = require('assert');
const cp = require('child_process');

// 1. execSync returns buffer with toString()
const out = cp.execSync('echo hello_sync');
assert(out.toString().includes('hello_sync'), 'execSync output should include hello_sync');

// 2. execSync with encoding: 'utf8' returns string
const outStr = cp.execSync('echo hello_utf8', { encoding: 'utf8' });
assert.strictEqual(typeof outStr, 'string', 'execSync with utf8 encoding should return string');
assert(outStr.includes('hello_utf8'), 'execSync output string should include hello_utf8');

// 3. spawnSync returns result object with status and stdout/stderr
const spawnRes = cp.spawnSync('echo', ['spawn_works']);
assert.strictEqual(spawnRes.status, 0, 'spawnSync exit status should be 0');
assert(spawnRes.stdout.toString().includes('spawn_works'), 'spawnSync stdout should contain spawn_works');
assert.strictEqual(Array.isArray(spawnRes.output), true, 'spawnSync output should be an array');

console.log('CONFORMANCE_PASS');

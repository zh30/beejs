const path = require('path');
const moduleApi = require('module');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof moduleApi.createRequire === 'function', 'module.createRequire exists');
const created = moduleApi.createRequire(__filename);
const loadedPath = created('path');
assert(typeof loadedPath.join === 'function', 'createRequire can load path');
assert(path.join('a', 'b') === loadedPath.join('a', 'b'), 'createRequire path.join matches');
console.log('CONFORMANCE_PASS');

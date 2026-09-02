const childProcess = require('child_process');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

let denied = false;
try {
  childProcess.exec('ls -la');
} catch (error) {
  const text = String(error && error.message ? error.message : error).toLowerCase();
  denied = text.includes('permission') || text.includes('denied');
}

assert(denied, 'child_process.exec must match argv0 and stay denied under --sandbox');
console.log('CONFORMANCE_PASS');

const fs = require('fs');
const path = require('path');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

const allowed = path.join(__dirname, 'jail', 'allowed.txt');
const blocked = path.join(__dirname, 'secret.txt');

const text = fs.readFileSync(allowed, 'utf8').trim();
assert(text === 'jail-ok', 'prefix allow must read files under the jail directory');

let denied = false;
try {
  fs.readFileSync(blocked, 'utf8');
} catch (error) {
  const message = String(error && error.message ? error.message : error).toLowerCase();
  denied = message.includes('permission') || message.includes('denied');
}

assert(denied, 'paths outside the jail prefix must stay denied');
console.log('CONFORMANCE_PASS');

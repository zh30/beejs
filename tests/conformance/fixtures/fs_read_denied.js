const fs = require('fs');
const path = require('path');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

const blocked = path.join(__dirname, 'secret.txt');
let denied = false;
try {
  fs.readFileSync(blocked, 'utf8');
} catch (error) {
  const text = String(error && error.message ? error.message : error).toLowerCase();
  denied = text.includes('permission') || text.includes('denied');
}

assert(denied, 'fs.readFileSync must be denied under --sandbox');
console.log('CONFORMANCE_PASS');

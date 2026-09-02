function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

function isDenied(error) {
  const text = String(error && error.message ? error.message : error).toLowerCase();
  return text.includes('permission') || text.includes('denied');
}

let blockedDenied = false;
try {
  fetch('http://example.com/');
} catch (error) {
  blockedDenied = isDenied(error);
}

assert(blockedDenied, 'fetch outside the allow-list must be denied');

let allowedDenied = false;
try {
  fetch('http://127.0.0.1:9/agent-allowlist');
} catch (error) {
  allowedDenied = isDenied(error);
}

assert(!allowedDenied, 'fetch to an allow-listed host must pass the broker');
console.log('CONFORMANCE_PASS');

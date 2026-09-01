const dns = require('dns');

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof dns.lookup === 'function', 'dns.lookup exists');

dns.lookup('localhost', (err, address) => {
  if (err) throw new Error('dns.lookup failed: ' + err);
  const first = Array.isArray(address) ? address[0] : address;
  const value = typeof first === 'string' ? first : first && first.address;
  assert(typeof value === 'string' && value.length > 0, 'lookup returns an address');
  console.log('CONFORMANCE_PASS');
});

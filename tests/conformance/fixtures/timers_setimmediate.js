function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof setImmediate === 'function', 'setImmediate exists');
assert(typeof setTimeout === 'function', 'setTimeout exists');

let remaining = 2;
function done(label) {
  assert(label === 'immediate' || label === 'timeout', 'unexpected timer label');
  remaining -= 1;
  if (remaining === 0) {
    console.log('CONFORMANCE_PASS');
  }
}

setImmediate(() => done('immediate'));
setTimeout(() => done('timeout'), 0);

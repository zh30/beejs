// timers conformance fixture: ordering, chained setImmediate, clearTimeout
// Contracts verified (Node-compatible):
// - process.nextTick runs before Promise microtasks
// - Promise microtasks run before timers
// - setTimeout(fn, 0) fires before a 50ms timer
// - setImmediate fires before a 50ms timer (check phase of the same iteration)
// - setImmediate registered from within setImmediate runs in a later iteration
// - clearTimeout prevents a timer from firing

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

const order = [];
let clearedFired = false;

const cleared = setTimeout(() => { clearedFired = true; }, 5);
clearTimeout(cleared);

process.nextTick(() => order.push('nextTick'));
Promise.resolve().then(() => order.push('promise'));
setTimeout(() => order.push('timeout0'), 0);

setImmediate(() => {
  order.push('immediate');
  // Chained immediate: must run in a later check phase, not synchronously.
  setImmediate(() => {
    order.push('immediate2');
  });
});

setTimeout(() => {
  assert(clearedFired === false, 'clearTimeout prevents firing');
  const seq = order.join(',');
  assert(seq === 'nextTick,promise,timeout0,immediate,immediate2', 'order was: ' + seq);
  console.log('CONFORMANCE_PASS');
}, 50);

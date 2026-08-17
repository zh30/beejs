// process.nextTick conformance fixture
// Contracts verified (Node-compatible):
// - nextTick callbacks run after the current synchronous execution
// - nextTick runs before Promise microtasks
// - FIFO order within the nextTick queue
// - nextTick registered inside a nextTick runs after the current queue drains
// - extra arguments are forwarded to the callback

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

const order = [];

process.nextTick((a, b) => {
  assert(a === 1 && b === 2, 'nextTick forwards arguments');
  order.push('a');
  process.nextTick(() => order.push('c'));
}, 1, 2);
process.nextTick(() => order.push('b'));
Promise.resolve().then(() => order.push('promise'));
order.push('sync');

setTimeout(() => {
  const seq = order.join(',');
  // nextTick queue drains fully (FIFO, including nested) before Promises.
  assert(seq === 'sync,a,b,c,promise', 'order was: ' + seq);
  console.log('CONFORMANCE_PASS');
}, 20);

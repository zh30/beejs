const assert = require('assert');
const EventEmitter = require('events');

const ee = new EventEmitter();
assert.strictEqual(typeof ee.on, 'function');
assert.strictEqual(typeof ee.prependListener, 'function');
assert.strictEqual(typeof ee.eventNames, 'function');
assert.strictEqual(typeof ee.listeners, 'function');

// listenerCount static and instance
assert.strictEqual(EventEmitter.listenerCount(ee, 'foo'), 0);

const order = [];
ee.on('foo', () => order.push('second'));
ee.prependListener('foo', () => order.push('first'));

assert.strictEqual(EventEmitter.listenerCount(ee, 'foo'), 2);

ee.emit('foo');
assert.deepStrictEqual(order, ['first', 'second']);

const names = ee.eventNames();
assert(names.includes('foo'));

console.log('CONFORMANCE_PASS');

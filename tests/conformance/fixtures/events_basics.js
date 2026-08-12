// events EventEmitter conformance smoke fixture
const events = require('events');
const EventEmitter = events.EventEmitter || events;

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

assert(typeof EventEmitter === 'function', 'EventEmitter constructor');
const ee = new EventEmitter();
let hit = 0;
ee.on('ping', (v) => {
  hit += v;
});
ee.emit('ping', 2);
ee.emit('ping', 3);
assert(hit === 5, 'listener received events');

console.log('CONFORMANCE_PASS');

const assert = require('assert');

assert.strictEqual(typeof globalThis.Worker, 'function');

const workerCode = "self.onmessage = (e) => { postMessage({ reply: e.data.msg.toUpperCase() }); };";
const worker = new Worker(workerCode, { eval: true });

worker.onmessage = (e) => {
  assert.strictEqual(e.data.reply, 'HELLO WORKER');
  worker.terminate();
  console.log('CONFORMANCE_PASS');
};

worker.postMessage({ msg: 'hello worker' });

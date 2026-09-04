const assert = require('assert');
const { Worker, isMainThread, parentPort, workerData } = require('worker_threads');

assert.strictEqual(isMainThread, true);
assert.strictEqual(parentPort, null);
assert.strictEqual(workerData, undefined);

const workerCode = `
const { parentPort, workerData, isMainThread } = require('worker_threads');
if (!isMainThread && workerData && workerData.x) {
  parentPort.postMessage({ answer: workerData.x * 2 });
}
`;

const worker = new Worker(workerCode, { eval: true, workerData: { x: 21 } });
worker.on('message', (msg) => {
  assert.strictEqual(msg.answer, 42);
  console.log('CONFORMANCE_PASS');
});

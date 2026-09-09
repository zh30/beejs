const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');

const fastifyPath = path.resolve(__dirname, '../../../benchmarks/idle_memory/node_modules/fastify');
if (!fs.existsSync(fastifyPath)) {
    console.log('CONFORMANCE_SKIP');
    process.exit(0);
}
const Fastify = require(fastifyPath);

const app = Fastify({ logger: false });
assert.ok(app);
assert.strictEqual(typeof app.get, 'function');
assert.strictEqual(typeof app.post, 'function');
assert.strictEqual(typeof app.listen, 'function');

app.get('/health', async () => ({ status: 'ok' }));

console.log('CONFORMANCE_PASS');

const Fastify = require('fastify');

const fastify = Fastify({ logger: false });
const port = Number(process.env.PORT || 3000);
const host = '127.0.0.1';

fastify.get('/', async (request, reply) => {
  return 'Hello from Fastify!';
});

fastify.listen({ port, host }, (err) => {
  if (err) {
    console.error(err);
    process.exit(1);
  }
  console.log(`[Fastify Server] listening on http://${host}:${port}/ (PID: ${process.pid})`);
});

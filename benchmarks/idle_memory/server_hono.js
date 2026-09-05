const { Hono } = require('hono');

const app = new Hono();
app.get('/', (c) => c.text('Hello from Hono!'));

const port = Number(process.env.PORT || 3000);
const host = '127.0.0.1';

if (typeof Bun !== 'undefined') {
  Bun.serve({
    port,
    hostname: host,
    fetch: app.fetch,
  });
  console.log(`[Hono/Bun Server] listening on http://${host}:${port}/ (PID: ${process.pid})`);
} else {
  try {
    const { serve } = require('@hono/node-server');
    serve({
      fetch: app.fetch,
      port,
      hostname: host,
    }, (info) => {
      console.log(`[Hono/Node Server] listening on http://${host}:${port}/ (PID: ${process.pid})`);
    });
  } catch (e) {
    const http = require('http');
    http.createServer((req, res) => {
      res.writeHead(200, { 'Content-Type': 'text/plain' });
      res.end('Hello from Hono!');
    }).listen(port, host, () => {
      console.log(`[Hono/HTTP Server] listening on http://${host}:${port}/ (PID: ${process.pid})`);
    });
  }
}

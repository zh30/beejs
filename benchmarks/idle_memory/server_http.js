const http = require('http');

const port = Number(process.env.PORT || 3000);
const host = '127.0.0.1';

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('Hello from HTTP baseline!');
});

server.listen(port, host, () => {
  console.log(`[HTTP Server] listening on http://${host}:${port}/ (PID: ${process.pid})`);
});

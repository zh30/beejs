const http = require('http');

const port = Number(process.env.PORT || 3000);
const host = process.env.HOST || '127.0.0.1';

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });
  res.end('hello\n');
});

server.listen(port, host, () => {
  console.log(`hello_server listening on http://${host}:${port}/`);
});

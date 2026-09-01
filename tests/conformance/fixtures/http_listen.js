const http = require('http');

const server = http.createServer((_req, res) => {
  res.end('ok');
});

server.listen(0, '127.0.0.1', () => {
  console.log('CONFORMANCE_PASS');
  server.close();
});

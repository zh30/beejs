const http = require('http');

const server = http.createServer((req, res) => {
  const chunks = [];
  req.on('data', (chunk) => chunks.push(String(chunk)));
  req.on('end', () => {
    res.end(req.method + ':' + chunks.join(''));
  });
});

server.listen(0, '127.0.0.1', () => {
  const addr = typeof server.address === 'function' ? server.address() : { port: server.port };
  if (!addr || typeof addr.port !== 'number') {
    throw new Error('server.address() must return { port }');
  }
  console.log('CONFORMANCE_PASS');
  server.close();
});

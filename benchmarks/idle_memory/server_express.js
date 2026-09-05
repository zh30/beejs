const express = require('express');

const app = express();
const port = Number(process.env.PORT || 3000);
const host = '127.0.0.1';

app.get('/', (req, res) => {
  res.type('text/plain').send('Hello from Express!');
});

app.listen(port, host, () => {
  console.log(`[Express Server] listening on http://${host}:${port}/ (PID: ${process.pid})`);
});

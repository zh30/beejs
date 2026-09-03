function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}

let denied = false;
try {
  const value = process.env.HOME;
  if (value !== undefined) {
    throw new Error('HOME should not be readable under --sandbox');
  }
} catch (error) {
  const text = String(error && error.message ? error.message : error).toLowerCase();
  denied = text.includes('permission') || text.includes('denied');
}

assert(denied, 'process.env reads must throw under --sandbox');
console.log('CONFORMANCE_PASS');

const assert = require('assert');
const util = require('util');

assert.strictEqual(typeof util.promisify, 'function');
assert.strictEqual(typeof util.types, 'object');

function callbackFn(x, cb) {
    if (x < 0) {
        cb(new Error('negative'));
    } else {
        cb(null, x * 2);
    }
}

const asyncFn = util.promisify(callbackFn);
assert.strictEqual(typeof asyncFn, 'function');

asyncFn(21)
    .then(res => {
        assert.strictEqual(res, 42);
        return asyncFn(-1);
    })
    .then(() => {
        assert.fail('should have failed');
    })
    .catch(err => {
        assert.strictEqual(err.message, 'negative');
        console.log('CONFORMANCE_PASS');
    });

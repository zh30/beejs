const assert = require('assert');
const { StringDecoder } = require('string_decoder');

const decoder = new StringDecoder('utf8');

const buf1 = Buffer.from([0xE4, 0xBD]); // Half of 你 (0xE4 0xBD 0xA0)
const buf2 = Buffer.from([0xA0, 0xE5, 0xA5, 0xBD]); // Rest of 你 + 好 (0xE5 0xA5 0xBD)

const part1 = decoder.write(buf1);
const part2 = decoder.write(buf2);
const endPart = decoder.end();

const combined = part1 + part2 + endPart;
assert.strictEqual(combined, '你好');

console.log('CONFORMANCE_PASS');

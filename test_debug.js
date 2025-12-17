const fs = require('fs');
const path = require('path');

console.log('Starting test...');

try {
    const math = require('./debug_module.js');
    console.log('Math module:', math);
    console.log('Add function:', math.add);
    console.log('Result:', math.add(5, 3));
} catch (e) {
    console.log('Error:', e.message);
    console.log('Stack:', e.stack);
}

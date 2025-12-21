// Unique test to bypass any caching
const UNIQUE_ID_12345 = Math.random();
console.log('UNIQUE_ID_12345:', UNIQUE_ID_12345);
const blob = new Blob(['test' + UNIQUE_ID_12345]);
console.log('Blob created successfully:', blob.size);

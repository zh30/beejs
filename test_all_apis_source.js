// Check all Web APIs
console.log('fetch:', typeof fetch, fetch ? fetch.toString().substring(0, 50) : 'undefined');
console.log('WebSocket:', typeof WebSocket, WebSocket ? WebSocket.toString().substring(0, 50) : 'undefined');
console.log('URL:', typeof URL, URL ? URL.toString().substring(0, 50) : 'undefined');
console.log('TextEncoder:', typeof TextEncoder, TextEncoder ? TextEncoder.toString().substring(0, 50) : 'undefined');
console.log('Blob:', typeof Blob, Blob ? Blob.toString().substring(0, 50) : 'undefined');
console.log('File:', typeof File, File ? File.toString().substring(0, 50) : 'undefined');

// 测试 EventTarget 是否正常工作
console.log('测试 EventTarget:');
const et = new EventTarget();
console.log('typeof et.addEventListener:', typeof et.addEventListener);
console.log('typeof et.removeEventListener:', typeof et.removeEventListener);
console.log('typeof et.dispatchEvent:', typeof et.dispatchEvent);

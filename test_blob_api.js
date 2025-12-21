// Blob API 测试脚本
// 测试 Blob 和 File 构造函数及基本方法

console.log('=== Blob API 测试开始 ===\n');

// 测试 1: 检查 Blob 构造函数是否存在
console.log('测试 1: 检查 Blob 构造函数');
if (typeof Blob !== 'undefined') {
    console.log('✅ Blob 构造函数存在');
    console.log('   类型:', typeof Blob);
} else {
    console.log('❌ Blob 构造函数不存在');
}

// 测试 2: 创建基本 Blob
console.log('\n测试 2: 创建基本 Blob');
try {
    const blob = new Blob();
    console.log('✅ 空 Blob 创建成功');
    console.log('   size:', blob.size);
    console.log('   type:', blob.type);
} catch (e) {
    console.log('❌ 空 Blob 创建失败:', e.message);
}

// 测试 3: 创建带数据的 Blob
console.log('\n测试 3: 创建带数据的 Blob');
try {
    const blob = new Blob(['Hello, World!']);
    console.log('✅ 带字符串的 Blob 创建成功');
    console.log('   size:', blob.size);
    console.log('   type:', blob.type);
} catch (e) {
    console.log('❌ 带字符串的 Blob 创建失败:', e.message);
}

// 测试 4: 创建带 MIME 类型的 Blob
console.log('\n测试 4: 创建带 MIME 类型的 Blob');
try {
    const blob = new Blob(['{"test": "data"}'], { type: 'application/json' });
    console.log('✅ 带 MIME 类型的 Blob 创建成功');
    console.log('   size:', blob.size);
    console.log('   type:', blob.type);
} catch (e) {
    console.log('❌ 带 MIME 类型的 Blob 创建失败:', e.message);
}

// 测试 5: 检查 Blob 方法
console.log('\n测试 5: 检查 Blob 方法');
try {
    const blob = new Blob(['test']);
    const methods = ['arrayBuffer', 'text', 'slice', 'stream'];
    let allMethodsExist = true;

    for (const method of methods) {
        if (typeof blob[method] === 'function') {
            console.log(`✅ ${method} 方法存在`);
        } else {
            console.log(`❌ ${method} 方法不存在`);
            allMethodsExist = false;
        }
    }

    if (allMethodsExist) {
        console.log('✅ 所有 Blob 方法都存在');
    }
} catch (e) {
    console.log('❌ 检查 Blob 方法时出错:', e.message);
}

// 测试 6: 检查 File 构造函数
console.log('\n测试 6: 检查 File 构造函数');
if (typeof File !== 'undefined') {
    console.log('✅ File 构造函数存在');
    console.log('   类型:', typeof File);
} else {
    console.log('❌ File 构造函数不存在');
}

// 测试 7: 创建 File 对象
console.log('\n测试 7: 创建 File 对象');
try {
    const file = new File(['content'], 'test.txt');
    console.log('✅ File 创建成功');
    console.log('   name:', file.name);
    console.log('   size:', file.size);
    console.log('   type:', file.type);
    console.log('   lastModified:', file.lastModified);
} catch (e) {
    console.log('❌ File 创建失败:', e.message);
}

// 测试 8: 创建带 MIME 类型的 File
console.log('\n测试 8: 创建带 MIME 类型的 File');
try {
    const file = new File(['<html></html>'], 'index.html', { type: 'text/html' });
    console.log('✅ 带 MIME 类型的 File 创建成功');
    console.log('   name:', file.name);
    console.log('   type:', file.type);
} catch (e) {
    console.log('❌ 带 MIME 类型的 File 创建失败:', e.message);
}

// 测试 9: File 继承 Blob 方法
console.log('\n测试 9: File 继承 Blob 方法');
try {
    const file = new File(['test'], 'test.txt');
    const methods = ['arrayBuffer', 'text', 'slice', 'stream'];
    let allMethodsInherit = true;

    for (const method of methods) {
        if (typeof file[method] === 'function') {
            console.log(`✅ File.${method} 方法存在`);
        } else {
            console.log(`❌ File.${method} 方法不存在`);
            allMethodsInherit = false;
        }
    }

    if (allMethodsInherit) {
        console.log('✅ File 正确继承了 Blob 的所有方法');
    }
} catch (e) {
    console.log('❌ 检查 File 继承时出错:', e.message);
}

// 测试 10: 复杂的 Blob 创建（多部分数据）
console.log('\n测试 10: 创建多部分数据的 Blob');
try {
    const parts = ['Hello', ' ', 'World', '!'];
    const blob = new Blob(parts);
    console.log('✅ 多部分数据 Blob 创建成功');
    console.log('   size:', blob.size);
    console.log('   type:', blob.type);
} catch (e) {
    console.log('❌ 多部分数据 Blob 创建失败:', e.message);
}

console.log('\n=== Blob API 测试完成 ===');
console.log('如果看到 ✅ 标记，说明该功能正常工作');

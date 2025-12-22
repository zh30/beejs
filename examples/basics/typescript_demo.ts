/**
 * Beejs TypeScript 支持示例
 *
 * 注意: 当前版本 TypeScript 支持正在开发中
 * 请使用 .js 扩展名或查看文档了解支持状态
 */

// 类型定义 (用于文档说明)
/*
interface User {
    id: number;
    name: string;
    email: string;
    isActive: boolean;
}

interface Product {
    id: number;
    name: string;
    price: number;
    tags?: string[];
}
*/

// 泛型函数示例
function createUser<T extends User>(userData: T): T {
    console.log(`Creating user: ${userData.name}`);
    return userData;
}

// 异步函数示例
async function fetchProducts(): Promise<Product[]> {
    // 模拟 API 调用
    const products: Product[] = [
        { id: 1, name: "Laptop", price: 1200 },
        { id: 2, name: "Mouse", price: 25, tags: ["wireless", "gaming"] },
        { id: 3, name: "Keyboard", price: 80 }
    ];

    // 模拟异步延迟
    await new Promise(resolve => setTimeout(resolve, 100));

    return products;
}

// 箭头函数和函数式编程
const activeUsers = (users: User[]): User[] =>
    users.filter(user => user.isActive);

const userNames = (users: User[]): string[] =>
    users.map(user => user.name);

// 联合类型和类型守卫
function formatId(id: number | string): string {
    if (typeof id === "number") {
        return `User-${id.toString().padStart(4, '0')}`;
    }
    return `User-${id}`;
}

// 主函数
async function main() {
    console.log("=== Beejs TypeScript 支持示例 ===\n");

    // 创建用户
    const user: User = {
        id: 1,
        name: "Alice",
        email: "alice@example.com",
        isActive: true
    };

    const newUser = createUser(user);
    console.log("Created user:", newUser);

    // 获取产品列表
    console.log("\nFetching products...");
    const products = await fetchProducts();
    console.log("Products:", products);

    // 计算总价值
    const totalValue = products.reduce((sum, product) => sum + product.price, 0);
    console.log(`Total inventory value: $${totalValue}`);

    // 使用类型守卫
    console.log("\nFormatted IDs:");
    console.log(formatId(123));
    console.log(formatId("ABC123"));

    // 函数式编程示例
    const users: User[] = [
        { id: 1, name: "Alice", email: "alice@example.com", isActive: true },
        { id: 2, name: "Bob", email: "bob@example.com", isActive: false },
        { id: 3, name: "Charlie", email: "charlie@example.com", isActive: true }
    ];

    console.log("\nActive users:");
    const active = activeUsers(users);
    console.log(active);

    console.log("\nUser names:");
    console.log(userNames(users));

    console.log("\n✅ TypeScript 示例执行完成!");
}

// 执行主函数
main().catch(error => {
    console.error("Error:", error);
});

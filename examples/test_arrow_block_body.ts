// Test async arrow function with block body
const processUser = async (userId: string) => {
    const user = await fetchUser(userId);
    const validated = validateUser(user);
    return validated;
};

// Test regular arrow function with block body
const calculateTotal = (a: number, b: number) => {
    const sum = a + b;
    const message = "Total: " + sum;
    return sum;
};

console.log("Arrow functions with block body work correctly!");

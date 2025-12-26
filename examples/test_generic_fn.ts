// Test generic function
function identity<T>(value: T): T {
    return value;
}
console.log(`Generic: ${identity("test")}`);

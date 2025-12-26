// Generic function test
function identity<T>(value: T): T {
    return value;
}

console.log(identity("test"));

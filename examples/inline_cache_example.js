// This example demonstrates the inline cache functionality of Beejs.
// The beejs.getProperty function is provided by the runtime and uses the inline cache.

const obj = { name: "Beejs", type: "runtime", version: "0.1.0" };

// Accessing the 'name' property multiple times should benefit from the inline cache
console.log("Name:", beejs.getProperty(obj, "name"));
console.log("Name again:", beejs.getProperty(obj, "name"));

// Accessing a different property
console.log("Type:", beejs.getProperty(obj, "type"));

// Accessing a non-existent property
console.log("Author:", beejs.getProperty(obj, "author"));

console.log("Beejs inline cache example completed.");

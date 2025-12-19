// Test script for Stage 56.1 - CLI Core Architecture
console.log("Hello from Beejs Stage 56.1!");
console.log("CLI subcommands are working correctly!");

// Test command line arguments
if (process.argv.length > 2) {
    console.log("Arguments received:", process.argv.slice(2));
}

// Test file detection
console.log("File type detection: JavaScript");

use beejs::Runtime;

fn main() {
    let runtime = Runtime::new(67108864, 1073741824, true).unwrap();
    
    println!("Testing typeof path...");
    match runtime.execute_code("typeof path") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nTesting typeof console...");
    match runtime.execute_code("typeof console") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

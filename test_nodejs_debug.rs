use beejs::Runtime;

fn main() {
    let runtime = Runtime::new(67108864, 1073741824, true).unwrap();
    
    println!("Testing path.join...");
    match runtime.execute_code(r#"path.join("foo", "bar", "baz")"#) {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nTesting process.argv...");
    match runtime.execute_code("process.argv") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

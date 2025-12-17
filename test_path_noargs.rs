use beejs::Runtime;

fn main() {
    let runtime = Runtime::new(67108864, 1073741824, true).unwrap();
    
    println!("Testing path.cwd...");
    match runtime.execute_code("path.cwd()") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

use beejs::Runtime;

fn main() {
    let runtime = Runtime::new(67108864, 1073741824, true).unwrap();
    
    println!("Testing path.join existence...");
    match runtime.execute_code("typeof path.join") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nTesting path.join call...");
    match runtime.execute_code("path.join('foo', 'bar')") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

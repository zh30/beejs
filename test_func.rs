use beejs::Runtime;

fn main() {
    let runtime = Runtime::new(67108864, 1073741824, true).unwrap();
    
    println!("Testing simple function call...");
    match runtime.execute_code("console.log('test')") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

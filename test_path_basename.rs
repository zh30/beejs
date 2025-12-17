use beejs::Runtime;

fn main() {
    let runtime = Runtime::new(67108864, 1073741824, true).unwrap();
    
    println!("Testing path.basename...");
    match runtime.execute_code(r#"path.basename("/foo/bar/baz.txt")"#) {
        Ok(result) => println!("Success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

// Test temporary file
const runtime = beejs::Runtime::new(67108864, 1073741824, false).unwrap();

// Create a simple module
let mut module_file = NamedTempFile::new().unwrap();
writeln!(module_file, r#"
    module.exports = {{
        add: (a, b) => a + b,
        multiply: (a, b) => a * b
    }};
"#).unwrap();

// Create main file that uses the module
let mut main_file = NamedTempFile::new().unwrap();
writeln!(main_file, r#"
    const math = require('./{}');
    console.log('Math:', math);
    console.log('Add:', math.add);
    console.log('Result:', math.add(5, 3));
    math.add(5, 3);
"#, module_file.path().file_name().unwrap().to_str().unwrap()).unwrap();

let result = runtime.execute_file(&main_file.path().to_path_buf());
println!("Result: {:?}", result);
if let Ok(output) = result {
    println!("Output: {}", output);
}

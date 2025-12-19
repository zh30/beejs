//! Test V8 API compatibility for Buffer module
//! This test verifies that the Buffer API can be properly set up

use rusty_v8 as v8;

fn main() {
    let platform = v8::Platform::new_single_threaded().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::init();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let mut scope = v8::HandleScope::new(&mut isolate);

    let context = v8::Context::new(&mut scope);
    let mut scope = v8::ContextScope::new(&mut scope, context);

    // Try to set up Buffer API
    match beejs::nodejs_core::buffer::setup_buffer_api(&mut scope, &context) {
        Ok(_) => println!("✅ Buffer API setup successful"),
        Err(e) => {
            println!("❌ Buffer API setup failed: {:?}", e);
            std::process::exit(1);
        }
    }

    println!("✅ All tests passed!");
}

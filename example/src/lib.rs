// Simplified example with logging capability

// Generate bindings from the WIT file
wit_bindgen::generate!({
    path: "wit",
    world: "hello",
    exports: {
        "example:hello/example": HelloComponent,
    },
});

struct HelloComponent;

// Implement the example interface
impl exports::example::hello::example::Guest for HelloComponent {
    // Log a message to demonstrate logging capability
    fn log(level: exports::example::hello::example::Level, message: String) {
        // In a real component, this would be implemented by the host
        println!("[{:?}] {}", level, message);
    }

    // Our main hello function
    fn hello() -> i32 {
        // Log a message using our own logging function
        Self::log(
            exports::example::hello::example::Level::Info,
            "Hello from WebAssembly via WIT logging!".to_string(),
        );

        // Return value
        42
    }
}

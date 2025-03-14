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

    // Our main hello function with a loop that runs 100 iterations
    fn hello() -> i32 {
        // Log a message using our own logging function
        Self::log(
            exports::example::hello::example::Level::Info,
            "Starting loop for 100 iterations".to_string(),
        );

        let mut count = 0;

        // Loop for 100 iterations, logging each step
        for i in 0..100 {
            count += 1;

            // Log the current iteration number
            Self::log(
                exports::example::hello::example::Level::Debug,
                format!("Loop iteration: {}", i + 1),
            );

            // Add some operations to consume more fuel
            let mut _sum = 0;
            for j in 0..i {
                _sum += j;
            }
        }

        // Log completion message
        Self::log(
            exports::example::hello::example::Level::Info,
            format!("Completed {} iterations", count),
        );

        // Return total iterations
        count
    }
}

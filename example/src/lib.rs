// Example using WASI logging imports

// Generate bindings from the WIT file
wit_bindgen::generate!({
    // Specify the path to our WIT directory
    path: "wit",

    // Define the world we're implementing
    world: "hello",

    // Specify which interfaces we're exporting
    exports: {
        "example:hello/example": HelloComponent,
    },
});

// Import the generated logging functions
use example::hello::logging;

struct HelloComponent;

// Implement the example interface
impl exports::example::hello::example::Guest for HelloComponent {
    // Our main hello function that runs a loop for several iterations
    fn hello() -> i32 {
        // Log a message using the imported WASI logging function
        logging::log(
            logging::Level::Info,
            "example",
            "TEST_MESSAGE: This is a test message from the component",
        );

        //let count = 0;
        let mut count = 0;

        // Loop for 5 iterations, logging each step
        for i in 0..5 {
            count += 1;

            // Add some operations to consume more fuel
            let mut _sum = 0;
            for j in 0..i {
                _sum += j;
            }
        }

        // Log completion message
        let final_message = format!("Completed {} iterations", count);
        logging::log(logging::Level::Info, "example", &final_message);
        logging::log(
            logging::Level::Info,
            "end",
            "TEST_MESSAGE_END: This is a test message from the component",
        );

        // Return total iterations
        count
    }
}

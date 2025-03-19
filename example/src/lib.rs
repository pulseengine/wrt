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
    // Test function specifically for unsigned comparison operations
    fn hello() -> i32 {
        // Start test
        logging::log(
            logging::Level::Info,
            "test",
            "UNSIGNED_TEST: Testing unsigned comparisons",
        );

        // Test cases for unsigned comparisons
        // These values are chosen to test the difference between signed and unsigned comparisons
        let a: i32 = -10; // In unsigned view: 4294967286 (very large positive)
        let b: i32 = 10; // In unsigned view: 10 (small positive)

        // For LtU: -10 < 10 (signed) but 4294967286 > 10 (unsigned)
        let lt_signed = a < b;
        let lt_unsigned = (a as u32) < (b as u32);

        if lt_signed {
            logging::log(logging::Level::Info, "test", "Signed: -10 < 10 (correct)");
        }

        if !lt_unsigned {
            logging::log(
                logging::Level::Info,
                "test",
                "Unsigned: 4294967286 > 10 (correct)",
            );
        }

        // For GtU: -10 > 10 (unsigned comparison: 4294967286 > 10)
        if (a as u32) > (b as u32) {
            logging::log(
                logging::Level::Info,
                "test",
                "GtU test passed: 4294967286 > 10",
            );
        }

        // For GeU: -10 >= 10 (unsigned comparison: 4294967286 >= 10)
        if (a as u32) >= (b as u32) {
            logging::log(
                logging::Level::Info,
                "test",
                "GeU test passed: 4294967286 >= 10",
            );
        }

        // For LeU: 10 <= -10 (unsigned comparison: 10 <= 4294967286)
        if (b as u32) <= (a as u32) {
            logging::log(
                logging::Level::Info,
                "test",
                "LeU test passed: 10 <= 4294967286",
            );
        }

        logging::log(
            logging::Level::Info,
            "test",
            "All unsigned comparison tests passed!",
        );

        // Return success
        0
    }
}

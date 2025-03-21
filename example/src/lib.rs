// Example using WASI logging imports

// Generate bindings from the WIT file
wit_bindgen::generate!({
    path: "wit",
    world: "hello",
    exports: {
        "example:hello/example": HelloComponent,
    },
});

use example::hello::logging;
use exports::example::hello::example::Guest;

struct HelloComponent;

impl Guest for HelloComponent {
    fn hello() -> i32 {
        logging::log(
            logging::Level::Info,
            "example",
            "SIMPLE_TEST: Minimal example with I32 operations",
        );

        let a: i32 = 10;
        let b: i32 = 20;

        // Test addition
        let sum = a + b;
        logging::log(
            logging::Level::Info,
            "example",
            &format!("I32 Add: 10 + 20 = {}", sum),
        );

        // Test subtraction
        let diff = b - a;
        logging::log(
            logging::Level::Info,
            "example",
            &format!("I32 Sub: 20 - 10 = {}", diff),
        );

        // Test multiplication
        let product = a * b;
        logging::log(
            logging::Level::Info,
            "example",
            &format!("I32 Mul: 10 * 20 = {}", product),
        );

        // Test comparison
        if a < b {
            logging::log(
                logging::Level::Info,
                "example",
                "I32 comparison: 10 < 20 (correct)",
            );
        } else {
            logging::log(
                logging::Level::Info,
                "example",
                "I32 comparison: 10 >= 20 (wrong)",
            );
            return 1;
        }

        sum
    }
}

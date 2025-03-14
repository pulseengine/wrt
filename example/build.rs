fn main() {
    // Tell Cargo to rerun this build script if the WIT files change
    println!("cargo:rerun-if-changed=wit/example.wit");

    // This will prevent false positives from cargo-udeps by using the dependency
    let _ = wit_component::ComponentEncoder::default();
}

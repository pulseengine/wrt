load("@crates//:defs.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")
# Fix the path for wasm_bindgen
# load("@rules_rust//wasm_bindgen:defs.bzl", "rust_wasm_bindgen")

package(default_visibility = ["//visibility:public"])

# Define filegroup for WIT files
filegroup(
    name = "wit_files",
    srcs = glob(["wit/**/*.wit"]),
)

# Create a simpler example that doesn't use wit-bindgen
rust_library(
    name = "example_lib",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    crate_name = "example",
    # Use selective dependencies to avoid problematic ones
    deps = [
        # Let's use direct dependencies instead of all_crate_deps()
        # that would pull in the problematic packages
    ],
    data = [":wit_files"],
    rustc_env = {
        "CARGO_PKG_VERSION": "0.1.0",
    },
)

# Simple build tool that doesn't depend on wit-bindgen
rust_binary(
    name = "example_build",
    srcs = ["build.rs"],
    edition = "2021",
    crate_name = "build_script_build",
    deps = [],
    data = [":wit_files"],
)

# Comment out the wasm_bindgen target until we fix the import path
# rust_wasm_bindgen(
#     name = "example_wasm",
#     wasm_file = ":example_lib",
# ) 
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test", "rust_test_suite", "rustfmt_test")
load("@crates//:defs.bzl", "all_crate_deps")

# Keep these load statements for now, we might need to adjust them later
load("//bazel:test.bzl", "test")
load("//bazel:fmt.bzl", "fmt")
load("//bazel:check.bzl", "check")
load("//bazel:docs.bzl", "docs")
load("//bazel:all.bzl", "all")

# Keep workspace filegroups
filegroup(
    name = "cargo_lock",
    srcs = ["Cargo.lock"],
    visibility = ["//visibility:public"],
)

filegroup(
    name = "workspace_files",
    srcs = [
        "Cargo.toml",
        "Cargo.lock",
    ],
    visibility = ["//visibility:public"],
)

# Aliases for all crate targets
alias(
    name = "wrt-common",
    actual = "//wrt-common:wrt-common",
    visibility = ["//visibility:public"],
)

alias(
    name = "wrt-error",
    actual = "//wrt-error:wrt-error",
    visibility = ["//visibility:public"],
)

alias(
    name = "wrt-types",
    actual = "//wrt-types:wrt-types",
    visibility = ["//visibility:public"],
)

alias(
    name = "wrt-runtime",
    actual = "//wrt-runtime:wrt-runtime",
    visibility = ["//visibility:public"],
)

alias(
    name = "wrt-component",
    actual = "//wrt-component:wrt-component",
    visibility = ["//visibility:public"],
)

alias(
    name = "wrt-sync",
    actual = "//wrt-sync:wrt-sync",
    visibility = ["//visibility:public"],
)

alias(
    name = "wrt",
    actual = "//wrt:wrt",
    visibility = ["//visibility:public"],
)

# --- Standard Rust Targets ---
# wrt library is now defined in wrt/BUILD
# wrtd binary is now defined in wrtd/BUILD

# Use the new example targets defined in example/BUILD
alias(
    name = "example",
    actual = "//example:example_lib",
    visibility = ["//visibility:public"],
)

# Commented out until wasm_bindgen is properly set up
# alias(
#     name = "example_wasm",
#     actual = "//example:example_wasm",
#     visibility = ["//visibility:public"],
# )

# Renamed from logging_adapter based on Cargo.toml members
rust_library(
    name = "wrt_logging",
    srcs = glob(["wrt-logging/src/**/*.rs"]),
    crate_root = "wrt-logging/src/lib.rs",
    edition = "2021",
    rustc_flags = ["--target=wasm32-wasip2"],
    visibility = ["//visibility:public"],
    deps = all_crate_deps(
        normal = True,
        proc_macro = True,
    ),
    # TODO: Investigate cargo component build mapping
)

# --- Remaining Custom Targets ---
# We may need to update these later to depend on the new rust_* targets

test(
    name = "test",
)

fmt(
    name = "fmt",
)

check(
    name = "check",
)

docs(
    name = "docs",
)

all(
    name = "all",
)

# Migrated from justfile
load("//bazel:build-example.bzl", "build-example")
load("//bazel:build-adapter.bzl", "build-adapter")

build-example(
    name = "build-example",
)

build-adapter(
    name = "build-adapter",
)

# Standard tasks across all crates
rust_test_suite(
    name = "all_tests",
    srcs = [],
    deps = [
        "//wrt:wrt_test",
        "//wrt-common:wrt-common_test",
        "//wrt-error:wrt-error_test",
        "//wrt-types:wrt-types_test",
        "//wrt-component:wrt-component_test",
        "//wrt-sync:wrt-sync_test",
        "//wrt-runtime:wrt-runtime_test",
    ],
)

# Format all code
filegroup(
    name = "all_files",
    srcs = glob(["**/*.rs"]),
)

# Use rustfmt_test rule for formatting
rustfmt_test(
    name = "fmt",
    targets = [
        "//wrt:wrt",
        "//wrt-common:wrt-common",
        "//wrt-error:wrt-error", 
        "//wrt-types:wrt-types",
        "//wrt-component:wrt-component",
        "//wrt-sync:wrt-sync",
        "//wrt-runtime:wrt-runtime",
    ],
) 
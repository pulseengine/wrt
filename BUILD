load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test", "rust_test_suite", "rustfmt_test", "rust_clippy")
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
    # Adding a data dependency to ensure clippy targets exist
    # (they will be called by the check script)
    data = [
        ":clippy_all",
    ],
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
        "//wrt:wrt_no_std_test",
        "//wrt-common:wrt-common_test",
        "//wrt-error:wrt-error_test",
        "//wrt-error:wrt-error_no_std_test",
        "//wrt-types:wrt-types_test",
        "//wrt-types:wrt-types_no_std_test",
        "//wrt-component:wrt-component_test",
        "//wrt-component:wrt-component_no_std_test",
        "//wrt-sync:wrt-sync_test",
        "//wrt-sync:wrt-sync_no_std_test",
        "//wrt-runtime:wrt-runtime_test",
        "//wrt-runtime:wrt-runtime_no_std_test",
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

# Add feature-specific build targets
load("//bazel:feature-build.bzl", "build_with_features")

build_with_features(
    name = "build_std",
    feature_set = "std",
)

build_with_features(
    name = "build_no_std",
    feature_set = "no_std",
)

# Add clippy targets
load("//bazel:clippy.bzl", "clippy_all", "clippy_with_features")

# Run clippy on specific targets with default configuration
rust_clippy(
    name = "clippy",
    deps = [
        "//wrt:wrt",
        "//wrt-runtime:wrt-runtime",
        "//wrt-types:wrt-types",
        "//wrt-component:wrt-component",
        "//wrt-sync:wrt-sync",
        "//wrt-error:wrt-error",
        "//wrtd",
    ],
)

# Run clippy with std features
clippy_with_features(
    name = "clippy_std",
    feature_set = "std",
)

# Run clippy with no_std features
clippy_with_features(
    name = "clippy_no_std",
    feature_set = "no_std",
)

# Run clippy with both feature sets
clippy_all(
    name = "clippy_all",
) 
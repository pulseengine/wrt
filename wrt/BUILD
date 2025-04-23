load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test", "rust_binary")
load("@crates//:defs.bzl", "aliases", "all_crate_deps")
load("//bazel:features.bzl", "rust_library_with_features", "rust_test_with_features")

# Create a standard library target with std features
rust_library_with_features(
    name = "wrt",
    srcs = glob(["src/**/*.rs"]),
    feature_set = "std",
    deps = [
        "//:wrt-common",
        "//:wrt-types",
        "//:wrt-error",
        "//:wrt-runtime",
        "//:wrt-component",
        "//:wrt-sync",
    ],
)

# Create a no_std library target for when both are needed
rust_library_with_features(
    name = "wrt_no_std",
    srcs = glob(["src/**/*.rs"]),
    feature_set = "no_std",
    deps = [
        "//:wrt-common",
        "//:wrt-types",
        "//:wrt-error",
        "//:wrt-runtime",
        "//:wrt-component",
        "//:wrt-sync",
    ],
)

# Test targets for both std and no_std
rust_test_with_features(
    name = "wrt",
    srcs = glob(["src/**/*.rs"]),
    feature_set = "both",  # Create both std and no_std test targets
    deps = [
        "//:wrt-common",
        "//:wrt-types",
        "//:wrt-error",
        "//:wrt-runtime",
        "//:wrt-component",
        "//:wrt-sync",
    ],
)

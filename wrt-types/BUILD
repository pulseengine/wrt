load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crates//:defs.bzl", "aliases", "all_crate_deps")
load("//bazel:features.bzl", "rust_library_with_features", "rust_test_with_features")

# Create std library target
rust_library_with_features(
    name = "wrt-types",
    srcs = glob(["src/**/*.rs"]),
    feature_set = "std",
    deps = [
        "//:wrt-common",
        "//:wrt-error",
    ],
)

# Create a no_std library target
rust_library_with_features(
    name = "wrt-types_no_std",
    srcs = glob(["src/**/*.rs"]),
    feature_set = "no_std",
    deps = [
        "//:wrt-common",
        "//:wrt-error",
    ],
)

# Test targets for both std and no_std
rust_test_with_features(
    name = "wrt-types",
    srcs = glob(["src/**/*.rs"]),
    feature_set = "both",
    deps = [
        "//:wrt-common",
        "//:wrt-error",
    ],
)

load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@crates//:defs.bzl", "all_crate_deps")

package(default_visibility = ["//visibility:public"])

rust_binary(
    name = "xtask",
    srcs = glob(["src/**/*.rs"]),
    crate_root = "src/main.rs",
    edition = "2021",
    deps = all_crate_deps(
        normal = True,
        proc_macro = True,
        exclude = ["xtask"],
    ),
) 
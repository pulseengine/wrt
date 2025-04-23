load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test", "rust_binary")
load("@crates//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "wrt-error",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    rustc_flags = [
        "--cfg=feature=\"alloc\"",
        "--cfg=feature=\"std\"",
    ],
    deps = all_crate_deps(
        normal = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
)

rust_test(
    name = "wrt-error_test",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    rustc_flags = [
        "--cfg=feature=\"alloc\"",
        "--cfg=feature=\"std\"",
    ],
    deps = all_crate_deps(
        normal = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
)

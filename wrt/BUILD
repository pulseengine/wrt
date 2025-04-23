load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test", "rust_binary")
load("@crates//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "wrt",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    deps = all_crate_deps(
        normal = True,
    ) + [
        "//:wrt-common",
        "//:wrt-types",
        "//:wrt-error",
        "//:wrt-runtime",
        "//:wrt-component",
        "//:wrt-sync",
    ],
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
)

rust_test(
    name = "wrt_test",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    deps = all_crate_deps(
        normal = True,
    ) + [
        "//:wrt-common",
        "//:wrt-types",
        "//:wrt-error",
        "//:wrt-runtime",
        "//:wrt-component",
        "//:wrt-sync",
    ],
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
)

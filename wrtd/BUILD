load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")
load("@crates//:defs.bzl", "all_crate_deps")

package(default_visibility = ["//visibility:public"])

rust_binary(
    name = "wrtd",
    srcs = glob(["src/**/*.rs"]),
    crate_root = "src/main.rs",
    edition = "2021",
    deps = ["//wrt:wrt"] + all_crate_deps(
        normal = True,
        proc_macro = True,
        exclude = ["wrtd"],
    ),
)

rust_test(
    name = "wrtd_test",
    crate = ":wrtd",
    edition = "2021",
    deps = ["//wrt:wrt"] + all_crate_deps(
        normal = True,
        normal_dev = True,
        proc_macro = True,
        proc_macro_dev = True,
        exclude = ["wrtd"],
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
        proc_macro_dev = True,
        exclude = ["wrtd"],
    ),
)

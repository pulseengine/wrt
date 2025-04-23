"""Helper functions for managing Rust feature flags across targets."""

load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crates//:defs.bzl", "all_crate_deps")

# Standard feature sets
STD_FEATURES = [
    "--cfg=feature=\"std\"",
    "--cfg=feature=\"alloc\"",
]

NO_STD_FEATURES = [
    "--cfg=feature=\"no_std\"",
]

OPTIMIZE_FEATURES = [
    "--cfg=feature=\"optimize\"",
]

SAFETY_FEATURES = [
    "--cfg=feature=\"safety\"",
]

def rust_library_with_features(
        name,
        srcs,
        edition = "2021",
        feature_set = "std",  # Can be "std", "no_std", "both"
        deps = [],
        proc_macro_deps = [],
        additional_rustc_flags = [],
        visibility = ["//visibility:public"]):
    """Creates Rust library targets with the specified feature sets.
    
    Args:
        name: Base name for the targets
        srcs: Source files
        edition: Rust edition
        feature_set: Which feature set to use ("std", "no_std", or "both")
        deps: Additional dependencies
        proc_macro_deps: Procedural macro dependencies
        additional_rustc_flags: Additional rustc flags
        visibility: Visibility of the targets
    """
    
    all_deps = deps + all_crate_deps(normal = True)
    all_proc_deps = proc_macro_deps + all_crate_deps(proc_macro = True)
    
    if feature_set == "std" or feature_set == "both":
        rust_library(
            name = name,
            srcs = srcs,
            edition = edition,
            rustc_flags = STD_FEATURES + additional_rustc_flags,
            deps = all_deps,
            proc_macro_deps = all_proc_deps,
            visibility = visibility,
        )
    
    if feature_set == "no_std" or feature_set == "both":
        target_name = name + "_no_std" if feature_set == "both" else name
        rust_library(
            name = target_name,
            srcs = srcs,
            edition = edition,
            rustc_flags = NO_STD_FEATURES + additional_rustc_flags,
            deps = all_deps,
            proc_macro_deps = all_proc_deps,
            visibility = visibility,
        )

def rust_test_with_features(
        name,
        srcs,
        edition = "2021",
        feature_set = "std",  # Can be "std", "no_std", "both"
        deps = [],
        proc_macro_deps = [],
        additional_rustc_flags = []):
    """Creates Rust test targets with the specified feature sets.
    
    Args:
        name: Base name for the targets
        srcs: Source files
        edition: Rust edition
        feature_set: Which feature set to use ("std", "no_std", or "both")
        deps: Additional dependencies
        proc_macro_deps: Procedural macro dependencies
        additional_rustc_flags: Additional rustc flags
    """
    
    all_deps = deps + all_crate_deps(normal = True)
    all_proc_deps = proc_macro_deps + all_crate_deps(proc_macro = True)
    
    if feature_set == "std" or feature_set == "both":
        rust_test(
            name = name + "_test",
            srcs = srcs,
            edition = edition,
            rustc_flags = STD_FEATURES + additional_rustc_flags,
            deps = all_deps,
            proc_macro_deps = all_proc_deps,
        )
    
    if feature_set == "no_std" or feature_set == "both":
        test_name = name + "_no_std_test" if feature_set == "both" else name + "_test"
        rust_test(
            name = test_name,
            srcs = srcs,
            edition = edition,
            rustc_flags = NO_STD_FEATURES + additional_rustc_flags,
            deps = all_deps,
            proc_macro_deps = all_proc_deps,
        ) 
"""Rules for building with specific feature sets."""

load("@bazel_skylib//lib:shell.bzl", "shell")

def _build_with_features_impl(ctx):
    feature_set = ctx.attr.feature_set
    if feature_set != "std" and feature_set != "no_std":
        fail("Feature set must be either 'std' or 'no_std'")
    
    script_content = """#!/bin/bash
    set -euo pipefail
    echo "Building with %s features..."
    cargo build --features %s
    """ % (feature_set, feature_set)

    script_file = ctx.actions.declare_file(ctx.label.name + ".sh")
    ctx.actions.write(script_file, script_content, is_executable = True)

    wrapper = ctx.actions.declare_file(ctx.label.name + "_wrapper.sh")
    ctx.actions.write(
        wrapper,
        """#!/bin/bash
        set -euo pipefail
        exec "$1" "$@"
        """,
        is_executable = True,
    )

    runfiles = ctx.runfiles(files = [script_file])
    return DefaultInfo(
        executable = wrapper,
        runfiles = runfiles,
    )

build_with_features = rule(
    implementation = _build_with_features_impl,
    attrs = {
        "feature_set": attr.string(
            default = "std",
            doc = "Feature set to build with (std or no_std)",
        ),
    },
    executable = True,
) 
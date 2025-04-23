"""Rules converted from justfile."""

load("@bazel_skylib//lib:shell.bzl", "shell")

def _check_impl(ctx):
    script_content = """
    # Run cargo fmt check
    cargo fmt -- --check
    
    # Run clippy with both std and no_std features via Bazel
    bazel run //:clippy_all
    """

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

check = rule(
    implementation = _check_impl,
    executable = True,
)

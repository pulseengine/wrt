"""Rules converted from justfile."""

load("@bazel_skylib//lib:shell.bzl", "shell")

def _build-example_impl(ctx):
    script_content = """
    # Use standard cargo build
    cargo build -p example --target wasm32-wasip2
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

build-example = rule(
    implementation = _build-example_impl,
    executable = True,
)

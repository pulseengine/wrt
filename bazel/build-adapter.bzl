"""Rules converted from justfile."""

load("@bazel_skylib//lib:shell.bzl", "shell")

def _build-adapter_impl(ctx):
    script_content = """
    # Ensure cargo-component is installed
    cargo install cargo-component --locked || true
    # Build using cargo component
    cargo component build -p logging-adapter --target wasm32-wasip2
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

build-adapter = rule(
    implementation = _build-adapter_impl,
    executable = True,
)

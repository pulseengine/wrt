"""Rules converted from justfile."""

load("@bazel_skylib//lib:shell.bzl", "shell")

def _check_impl(ctx):
    script_content = """
    cargo fmt -- --check
    cargo clippy --package wrtd -- -W clippy::missing_panics_doc -W clippy::missing_docs_in_private_items -A clippy::missing_errors_doc -A dead_code -A clippy::borrowed_box -A clippy::vec_init_then_push -A clippy::new_without_default
    # TBD: temporary disable checking no_std
    cargo clippy --package wrt --features std -- -W clippy::missing_panics_doc -W clippy::missing_docs_in_private_items -A clippy::missing_errors_doc -A dead_code -A clippy::borrowed_box -A clippy::vec_init_then_push -A clippy::new_without_default
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

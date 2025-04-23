"""Rules for running all key commands."""

def _all_impl(ctx):
    script_content = """
    echo "Running all key build steps in sequence..."

    # Run build first
    bazel build //...
    if [ $? -ne 0 ]; then
        echo "❌ Build failed."
        exit 1
    fi
    echo "✅ Build completed successfully."

    # Run tests
    bazel test //...
    if [ $? -ne 0 ]; then
        echo "❌ Tests failed."
        exit 1
    fi
    echo "✅ Tests completed successfully."

    # Run checks
    bazel run //:check
    if [ $? -ne 0 ]; then
        echo "❌ Checks failed."
        exit 1
    fi
    echo "✅ Checks completed successfully."

    # Build documentation
    bazel run //:docs
    if [ $? -ne 0 ]; then
        echo "❌ Documentation build failed."
        exit 1
    fi
    echo "✅ Documentation build completed successfully."

    echo "✨ All commands completed successfully!"
    """

    script_file = ctx.actions.declare_file(ctx.label.name + ".sh")
    ctx.actions.write(script_file, script_content, is_executable = True)

    # Create wrapper script for execution
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
    return [DefaultInfo(
        executable = wrapper,
        runfiles = runfiles,
    )]

all = rule(
    implementation = _all_impl,
    executable = True,
) 
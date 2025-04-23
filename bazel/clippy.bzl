"""Rules for running clippy checks with specific feature sets."""

load("@rules_rust//rust:defs.bzl", "rust_clippy")
load("@bazel_skylib//lib:shell.bzl", "shell")

def clippy_all(name):
    """Run clippy checks across all crates with both std and no_std features.
    
    Args:
        name: Target name
    """
    native.sh_binary(
        name = name,
        srcs = [name + ".sh"],
        data = [
            ":clippy_std",
            ":clippy_no_std",
        ],
    )
    
    native.genrule(
        name = name + "_script",
        outs = [name + ".sh"],
        cmd = """
cat > $@ << 'EOF'
#!/bin/bash
set -euo pipefail
echo "Running clippy with std features..."
bazel run :clippy_std
echo ""
echo "Running clippy with no_std features..."
bazel run :clippy_no_std
EOF
chmod +x $@
        """,
    )

def _clippy_with_features_impl(ctx):
    """Implementation for clippy_with_features rule."""
    feature_set = ctx.attr.feature_set
    if feature_set != "std" and feature_set != "no_std":
        fail("Feature set must be either 'std' or 'no_std'")
    
    script_content = """#!/bin/bash
set -euo pipefail
echo "Running clippy with %s features..."
bazel build --aspects=@rules_rust//rust:defs.bzl%%rust_clippy_aspect --output_groups=+clippy_checks %s
    """ % (feature_set, ctx.attr.targets)

    script_file = ctx.actions.declare_file(ctx.label.name + ".sh")
    ctx.actions.write(script_file, script_content, is_executable = True)

    wrapper = ctx.actions.declare_file(ctx.label.name + "_wrapper.sh")
    ctx.actions.write(
        wrapper,
        """#!/bin/bash
set -euo pipefail
exec "$1"
        """,
        is_executable = True,
    )

    runfiles = ctx.runfiles(files = [script_file])
    return DefaultInfo(
        executable = wrapper,
        runfiles = runfiles,
    )

clippy_with_features = rule(
    implementation = _clippy_with_features_impl,
    attrs = {
        "feature_set": attr.string(
            default = "std",
            doc = "Feature set to use for clippy ('std' or 'no_std')",
        ),
        "targets": attr.string(
            default = "//...",
            doc = "Targets to run clippy on (e.g., '//wrt:wrt')",
        ),
    },
    executable = True,
) 
# Git Hooks for WRT Project

This directory contains Git hooks used to enforce quality standards in the WRT project.

## Available Hooks

1. **pre-commit** - Runs before each commit:
   - Runs code formatting checks (`just check`)
   - Runs the core test suite (`just test-wrt`)

2. **pre-push** - Runs before each push:
   - Runs the full check suite (`just check-all`)
   - This includes all tests, linting, documentation checks, and more

## Installation

The hooks are automatically installed when you run:

```bash
just setup
```

Or you can install just the hooks with:

```bash
just setup-hooks
```

## Manual Installation

If you need to install the hooks manually:

```bash
cp .githooks/pre-commit .git/hooks/pre-commit
cp .githooks/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-commit .git/hooks/pre-push
```

## Bypassing Hooks

In emergency situations, you can bypass the hooks with the `--no-verify` flag:

```bash
git commit --no-verify -m "Emergency commit message"
git push --no-verify
```

However, this should be used only in exceptional cases.

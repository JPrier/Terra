#!/bin/bash
# Setup script to install git hooks for the Terra project

set -e

echo "Setting up Terra development environment..."

# Install pre-commit hook for cargo fmt
echo "Installing pre-commit hook for cargo fmt..."
cp scripts/pre-commit-hook.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "✅ Pre-commit hook installed successfully"
echo "The hook will automatically run 'cargo fmt --all' before each commit"
echo ""
echo "To install manually run:"
echo "  cp scripts/pre-commit-hook.sh .git/hooks/pre-commit"
echo "  chmod +x .git/hooks/pre-commit"
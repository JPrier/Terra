#!/bin/sh
# Pre-commit hook to run cargo fmt on all Rust code

set -e

echo "Running cargo fmt..."

# Check if any Rust files are being committed
if git diff --cached --name-only | grep -q '\.rs$'; then
    # Run cargo fmt on all Rust code
    cargo fmt --all
    
    # Add any reformatted files back to the staging area
    git add $(git diff --cached --name-only | grep '\.rs$')
    
    echo "✅ Cargo fmt completed successfully"
else
    echo "No Rust files to format"
fi
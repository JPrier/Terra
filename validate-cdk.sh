#!/bin/bash

# Local validation script that mirrors the GitHub Actions workflow
# Run this to test CDK deployment workflow locally

set -e

echo "🔍 Validating CDK Infrastructure Locally..."
echo "==========================================="

# Change to infra directory
cd "$(dirname "$0")/infra"

# Install dependencies
echo "📦 Installing CDK dependencies..."
npm ci

# Build CDK code
echo "🏗️  Building CDK TypeScript..."
npm run build

# Run tests
echo "🧪 Running CDK tests..."
npm test

# Validate synthesis
echo "🔨 Validating CDK synthesis..."
npm run synth > /dev/null

# Check Lambda compilation
echo "🦀 Validating Lambda compilation..."
cd ../backend
cargo check --manifest-path lambdas/api_rfqs/Cargo.toml
cargo check --manifest-path lambdas/api_uploads/Cargo.toml
cargo check --manifest-path lambdas/api_manufacturers/Cargo.toml
cargo check --manifest-path lambdas/image_ingest/Cargo.toml
cargo check --manifest-path lambdas/publisher/Cargo.toml

echo ""
echo "✅ All validations passed!"
echo "🚀 CDK infrastructure is ready for deployment."
echo ""
echo "To deploy manually (requires AWS credentials):"
echo "  cd infra && npm run deploy"
echo ""
echo "For automated deployment, push to main branch."
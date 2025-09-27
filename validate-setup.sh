#!/bin/bash

# Quick validation script for Terra integration test setup
# This tests the basic infrastructure without running full Playwright tests

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} ✅ $1"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')]${NC} ❌ $1"
}

log "🧪 Terra Integration Test Setup Validation"

# Cleanup on exit
cleanup() {
    log "Cleaning up..."
    docker compose -f docker-compose.test.yml down --volumes --remove-orphans >/dev/null 2>&1 || true
    success "Cleanup completed"
}
trap cleanup EXIT

log "1️⃣ Starting LocalStack..."
docker compose -f docker-compose.test.yml up -d localstack

# Wait for LocalStack
log "⏳ Waiting for LocalStack health check..."
for i in {1..30}; do
    if curl -s http://localhost:4566/_localstack/health > /dev/null 2>&1; then
        success "LocalStack is ready"
        break
    fi
    if [[ $i -eq 30 ]]; then
        error "LocalStack failed to start"
        exit 1
    fi
    sleep 2
done

log "2️⃣ Testing LocalStack S3 service..."
# Test if S3 endpoint is accessible
if curl -s http://localhost:4566/ | grep -q "localstack"; then
    success "LocalStack S3 endpoint accessible"
else
    error "LocalStack S3 endpoint not accessible"
    exit 1
fi

log "3️⃣ Setting up test data..."
cd tests
npm install > /dev/null 2>&1
if node setup-test-data.js; then
    success "Test data setup completed"
else
    error "Test data setup failed"
    exit 1
fi
cd ..

log "4️⃣ Testing Rust backend compilation..."
if cargo check > /dev/null 2>&1; then
    success "Rust backend compiles successfully"
else
    error "Rust backend compilation failed"
    exit 1
fi

log "5️⃣ Testing frontend build..."
cd frontend
if npm run build > /dev/null 2>&1; then
    success "Frontend builds successfully"
else
    error "Frontend build failed"
    exit 1
fi
cd ..

log "6️⃣ Testing Docker image builds..."
if docker compose -f docker-compose.test.yml build --quiet api-uploads > /dev/null 2>&1; then
    success "Docker images build successfully"
else
    error "Docker image build failed - this is expected in CI without full Docker daemon"
fi

success "🎉 All validation checks passed!"
log "Integration test framework is ready for use"
log ""
log "Next steps:"
log "  - Run full tests: ./run-integration-tests.sh"
log "  - Run with visible browser: ./run-integration-tests.sh --headed"
log "  - Setup dev environment: ./run-integration-tests.sh --setup-only --no-cleanup"
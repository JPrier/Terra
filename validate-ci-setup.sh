#!/bin/bash

# Simplified validation script for CI environments
# Focuses on basic compilation and setup without full Docker stack

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
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

log "🧪 Terra CI Validation"

log "1️⃣ Testing Rust backend compilation..."
if cargo check > /dev/null 2>&1; then
    success "Rust backend compiles successfully"
else
    error "Rust backend compilation failed"
    exit 1
fi

log "2️⃣ Testing frontend dependencies..."
cd frontend
if npm ci > /dev/null 2>&1; then
    success "Frontend dependencies installed"
else
    error "Frontend dependency installation failed"
    exit 1
fi

log "3️⃣ Testing frontend build..."
if ASTRO_TELEMETRY_DISABLED=1 npm run build > /dev/null 2>&1; then
    success "Frontend builds successfully"
else
    error "Frontend build failed"
    exit 1
fi
cd ..

log "4️⃣ Testing test dependencies..."
cd tests
if npm ci > /dev/null 2>&1; then
    success "Test dependencies installed"
else
    error "Test dependency installation failed"
    exit 1
fi

log "5️⃣ Setting up CI test data..."
if node setup-ci-test-data.js; then
    success "CI test data setup completed"
else
    error "CI test data setup failed"
    exit 1
fi
cd ..

success "🎉 All CI validation checks passed!"
log "CI environment is ready for testing"
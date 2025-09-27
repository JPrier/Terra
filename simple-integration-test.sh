#!/bin/bash

# Simple integration test for CI environments that may not have full Playwright support
# This provides basic validation of frontend functionality

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

cleanup() {
    if [ ! -z "$SERVER_PID" ]; then
        log "Killing server process $SERVER_PID"
        kill $SERVER_PID 2>/dev/null || true
        sleep 1
    fi
    pkill -f "astro" 2>/dev/null || true
}

trap cleanup EXIT

log "🧪 Simple Integration Test"

# Start the dev server
cd frontend
log "Starting Astro dev server..."
ASTRO_TELEMETRY_DISABLED=1 npm run dev > ../test-server.log 2>&1 &
SERVER_PID=$!

# Wait for server
log "Waiting for server to start..."
for i in {1..15}; do
    if curl -s http://localhost:4321 > /dev/null 2>&1; then
        success "Server started successfully"
        break
    fi
    if [ $i -eq 15 ]; then
        error "Server failed to start"
        echo "Server logs:"
        cat ../test-server.log
        exit 1
    fi
    sleep 2
done

cd ..

# Test basic functionality
log "Testing homepage..."
if curl -s http://localhost:4321 | grep -q "Connect with US Manufacturers"; then
    success "Homepage loads correctly"
else
    error "Homepage test failed"
    exit 1
fi

log "Testing catalog page..."
if curl -s http://localhost:4321/catalog/machining/ | grep -q "CNC Machining Manufacturers"; then
    success "Catalog page loads correctly"
else
    error "Catalog page test failed"
    exit 1
fi

log "Testing RFQ page..."
if curl -s http://localhost:4321/rfq/submit | grep -q "Submit Request for Quote"; then
    success "RFQ page loads correctly"
else
    error "RFQ page test failed"
    exit 1
fi

success "🎉 All basic integration tests passed!"
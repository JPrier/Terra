#!/bin/bash

# Terra Platform Integration Test Runner
# This script sets up and runs comprehensive E2E tests using LocalStack for AWS mocking

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} ✅ $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')]${NC} ⚠️  $1"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')]${NC} ❌ $1"
}

# Default values
HEADLESS=true
BROWSER="chromium"
SETUP_ONLY=false
SKIP_BUILD=false
CLEANUP=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --headed)
            HEADLESS=false
            shift
            ;;
        --browser)
            BROWSER="$2"
            shift 2
            ;;
        --setup-only)
            SETUP_ONLY=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --no-cleanup)
            CLEANUP=false
            shift
            ;;
        --help)
            echo "Terra Integration Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --headed        Run tests in headed mode (show browser)"
            echo "  --browser NAME  Specify browser (chromium, firefox, webkit)"
            echo "  --setup-only    Only setup test environment, don't run tests"
            echo "  --skip-build    Skip building Docker images"
            echo "  --no-cleanup    Don't cleanup containers after tests"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Cleanup function
cleanup() {
    if [[ "$CLEANUP" == "true" ]]; then
        log "Cleaning up Docker containers..."
        docker-compose -f docker-compose.test.yml down --volumes --remove-orphans >/dev/null 2>&1 || true
        success "Cleanup completed"
    else
        warn "Skipping cleanup (use --no-cleanup flag to keep containers running)"
    fi
}

# Set trap for cleanup on exit
trap cleanup EXIT

# Main execution
main() {
    log "Starting Terra Platform Integration Tests"
    log "Configuration: HEADLESS=$HEADLESS, BROWSER=$BROWSER"
    
    # Check prerequisites
    log "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        error "Docker is required but not installed"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is required but not installed"
        exit 1
    fi
    
    success "Prerequisites check passed"
    
    # Build Docker images (unless skipped)
    if [[ "$SKIP_BUILD" != "true" ]]; then
        log "Building Docker images..."
        docker-compose -f docker-compose.test.yml build --parallel
        success "Docker images built successfully"
    else
        warn "Skipping Docker image build"
    fi
    
    # Start services
    log "Starting test infrastructure..."
    docker-compose -f docker-compose.test.yml up -d localstack
    
    # Wait for LocalStack to be ready
    log "Waiting for LocalStack to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:4566/_localstack/health > /dev/null 2>&1; then
            success "LocalStack is ready"
            break
        fi
        if [[ $i -eq 30 ]]; then
            error "LocalStack failed to start within timeout"
            exit 1
        fi
        sleep 2
    done
    
    # Setup test data
    log "Setting up test data..."
    cd tests && npm install > /dev/null 2>&1
    node setup-test-data.js
    cd ..
    success "Test data setup completed"
    
    # Start backend services  
    log "Starting backend services..."
    docker-compose -f docker-compose.test.yml up -d api-uploads api-manufacturers api-rfqs
    
    # Wait for backend services
    log "Waiting for backend services..."
    for service in api-uploads:3000 api-manufacturers:3002 api-rfqs:3001; do
        service_name=$(echo $service | cut -d: -f1)
        port=$(echo $service | cut -d: -f2)
        
        for i in {1..30}; do
            if curl -s http://localhost:$port/health > /dev/null 2>&1; then
                success "$service_name is ready"
                break
            fi
            if [[ $i -eq 30 ]]; then
                warn "$service_name failed to start (may not have health endpoint)"
                break
            fi
            sleep 2
        done
    done
    
    # Start frontend
    log "Starting frontend..."
    docker-compose -f docker-compose.test.yml up -d frontend
    
    # Wait for frontend
    log "Waiting for frontend..."
    for i in {1..30}; do
        if curl -s http://localhost:4321 > /dev/null 2>&1; then
            success "Frontend is ready"
            break
        fi
        if [[ $i -eq 30 ]]; then
            error "Frontend failed to start within timeout"
            exit 1
        fi
        sleep 2
    done
    
    if [[ "$SETUP_ONLY" == "true" ]]; then
        success "Test environment setup completed"
        log "Services are running at:"
        log "  Frontend: http://localhost:4321"
        log "  RFQ API: http://localhost:3001"
        log "  Uploads API: http://localhost:3000"
        log "  Manufacturers API: http://localhost:3002"
        log "  LocalStack: http://localhost:4566"
        log ""
        log "Run tests manually with: cd tests && npm test"
        log "Or use --no-cleanup to keep services running"
        return 0
    fi
    
    # Run tests
    log "Running E2E tests..."
    cd tests
    
    # Install test dependencies if not already done
    if [[ ! -d "node_modules" ]]; then
        log "Installing test dependencies..."
        npm install > /dev/null 2>&1
    fi
    
    # Install Playwright browsers if needed
    if [[ ! -d "node_modules/@playwright/test" ]] || [[ ! -d "$HOME/.cache/ms-playwright" ]]; then
        log "Installing Playwright browsers..."
        npx playwright install --with-deps > /dev/null 2>&1
    fi
    
    # Set test environment variables
    export BASE_URL="http://localhost:4321"
    export API_BASE_URL="http://localhost:3001"
    
    # Run tests based on configuration
    if [[ "$HEADLESS" == "true" ]]; then
        log "Running tests in headless mode..."
        npx playwright test --project="$BROWSER"
    else
        log "Running tests in headed mode..."
        npx playwright test --project="$BROWSER" --headed
    fi
    
    test_exit_code=$?
    cd ..
    
    if [[ $test_exit_code -eq 0 ]]; then
        success "All tests passed! 🎉"
        log "Test results available in tests/test-results/"
        log "HTML report available in tests/playwright-report/"
    else
        error "Some tests failed"
        log "Check test output above and test-results/ directory for details"
        exit $test_exit_code
    fi
}

# Run main function
main "$@"
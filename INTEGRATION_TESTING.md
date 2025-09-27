# Terra Platform - Integration Testing 

This implementation provides comprehensive end-to-end integration tests covering every user-facing button and functionality on the Terra manufacturing platform.

## 🚀 Quick Start

### Run All Integration Tests
```bash
./run-integration-tests.sh
```

### Validate Setup Only
```bash
./validate-setup.sh
```

### Development Mode (Keep Services Running)
```bash
./run-integration-tests.sh --setup-only --no-cleanup
```

## 📋 Test Coverage

### ✅ Complete Button Coverage
- **Homepage**: "Browse Now", "Get Started", all category buttons
- **Catalog Pages**: "View Details", "Submit RFQ", filter controls, CTA buttons  
- **RFQ Form**: "Submit RFQ" with full validation and success flow
- **Manufacturer Cards**: All interactive elements and navigation
- **Mobile Responsiveness**: All tests run on mobile viewports

### 🔄 End-to-End Workflows
- Homepage → Catalog → Manufacturer Details
- Homepage → RFQ Form → Submission Success
- Catalog → Manufacturer → RFQ Submission
- Complete form validation and error handling
- API integration with mocked AWS services

### 🛠️ Infrastructure Testing
- **LocalStack Integration**: S3 and SES mocking
- **Backend API Testing**: All three service endpoints
- **File Upload Testing**: Presigned URL generation
- **Email Integration**: SES notification testing
- **Data Persistence**: S3 storage validation

## 🏗️ Architecture

```
Frontend (Astro)  ←→  Backend APIs (Rust)  ←→  LocalStack (AWS Mock)
     ↓                      ↓                        ↓
  Port 4321            Ports 3000-3002           Port 4566
     ↓                      ↓                        ↓
   Playwright Tests  ←→  HTTP Requests      ←→  S3/SES Operations
```

### Services
- **LocalStack**: AWS S3 and SES mocking
- **API RFQs**: RFQ management service (port 3001)
- **API Uploads**: File upload service (port 3000)
- **API Manufacturers**: Manufacturer management (port 3002)
- **Frontend**: Astro development server (port 4321)

## 📁 File Structure

```
├── tests/                          # Integration test suite
│   ├── tests/
│   │   ├── homepage.spec.js         # Homepage button tests
│   │   ├── catalog.spec.js          # Catalog functionality tests
│   │   ├── rfq-form.spec.js         # RFQ form submission tests
│   │   ├── manufacturer-details.spec.js  # Detail page tests
│   │   └── api-integration.spec.js  # Backend API tests
│   ├── setup-test-data.js           # LocalStack data seeding
│   ├── package.json                 # Test dependencies
│   └── playwright.config.js         # Playwright configuration
├── docker-compose.test.yml          # Test environment orchestration
├── run-integration-tests.sh         # Main test runner
├── validate-setup.sh               # Quick validation script
└── .github/workflows/integration-tests.yml  # CI/CD pipeline
```

## 🎯 Test Categories

### 1. Navigation Tests (`homepage.spec.js`)
- Tests every clickable element on homepage
- Validates navigation to catalog and RFQ pages
- Verifies feature sections and content display

### 2. Catalog Tests (`catalog.spec.js`)  
- Tests manufacturer card interactions
- Validates "View Details" and "Submit RFQ" buttons
- Tests CTA sections and mobile responsiveness

### 3. RFQ Form Tests (`rfq-form.spec.js`)
- Complete form validation (required fields, email format)
- Tests submission workflow and success states
- Validates loading states and error handling
- Tests manufacturer parameter handling

### 4. API Integration Tests (`api-integration.spec.js`)
- Tests backend service connectivity
- Validates file upload functionality
- Tests email notification system
- Verifies S3 data storage operations

### 5. Mobile Tests (All specs)
- Every test includes mobile viewport testing
- Validates responsive design and touch interactions
- Tests mobile-specific UI adaptations

## 🔧 Configuration

### Environment Variables
```bash
# LocalStack Configuration
AWS_ENDPOINT_URL=http://localhost:4566
AWS_ACCESS_KEY_ID=test
AWS_SECRET_ACCESS_KEY=test
AWS_DEFAULT_REGION=us-east-1

# Service Configuration  
PUBLIC_BUCKET=app-public-test
PRIVATE_BUCKET=app-private-test
SES_FROM_EMAIL=test@terra.local

# Test Configuration
BASE_URL=http://localhost:4321
API_BASE_URL=http://localhost:3001
```

### Test Data
The `setup-test-data.js` script creates:
- Sample manufacturers with realistic data
- S3 bucket structure with catalog data
- SES email configuration
- Manufacturer profiles and capabilities

## 🚀 CI/CD Integration

### GitHub Actions Workflow
The `.github/workflows/integration-tests.yml` provides:
- Automated test execution on PR and push
- Multi-browser testing (Chrome, Firefox, Safari)
- Test result artifacts and reports
- Performance testing with Lighthouse

### Local Development
```bash
# Setup development environment
./run-integration-tests.sh --setup-only --no-cleanup

# Run tests with browser visible
./run-integration-tests.sh --headed

# Debug specific test
cd tests && npx playwright test --debug homepage.spec.js
```

## 📊 Test Results

Tests generate comprehensive reports:
- **HTML Report**: Visual test results with screenshots
- **Video Recording**: Failed test playback
- **Screenshots**: Error state captures  
- **JSON Results**: Machine-readable test data

## 🐛 Troubleshooting

### Common Issues
1. **LocalStack not starting**: Check port 4566 availability
2. **Frontend build fails**: Verify Node.js 18+ and dependencies
3. **Backend compilation fails**: Check Rust toolchain and AWS SDK versions
4. **Tests timeout**: Increase timeout values in playwright.config.js

### Debug Commands
```bash
# Check service health
curl http://localhost:4566/_localstack/health
curl http://localhost:4321
curl http://localhost:3001/health

# View logs
docker compose -f docker-compose.test.yml logs localstack
docker compose -f docker-compose.test.yml logs api-rfqs

# Manual test run
cd tests && npx playwright test --headed --debug
```

## 🎉 Success Metrics

This integration test suite ensures:
- ✅ **100% Button Coverage**: Every interactive element tested
- ✅ **End-to-End Workflows**: Complete user journeys validated
- ✅ **Cross-Browser Support**: Chrome, Firefox, Safari, mobile
- ✅ **AWS Integration**: Full LocalStack mocking and validation
- ✅ **Responsive Design**: Mobile and desktop viewport testing
- ✅ **Error Handling**: Graceful failure scenarios covered
- ✅ **Performance**: Automated lighthouse scoring
- ✅ **CI/CD Ready**: GitHub Actions integration included

The tests provide confidence for deployment and feature development by validating the complete user experience from frontend interactions through backend processing to data persistence.
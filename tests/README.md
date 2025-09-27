# Terra Platform Integration Tests

This directory contains comprehensive end-to-end integration tests for the Terra manufacturing platform. The tests use local AWS mocks (LocalStack) to provide a complete testing environment that covers every user-facing button and functionality.

## Overview

The integration tests are designed to:
- Test every button and interactive element on the website
- Validate end-to-end workflows from frontend to backend
- Mock AWS services (S3, SES) using LocalStack
- Provide confidence in deployments and feature changes
- Run in CI/CD pipelines and local development

## Test Coverage

### Homepage Tests (`tests/homepage.spec.js`)
- ✅ "Browse Now" button navigation
- ✅ "Get Started" button navigation  
- ✅ Category buttons (CNC Machining, Sheet Metal, Injection Molding)
- ✅ Feature section display
- ✅ Popular categories section

### Catalog Tests (`tests/catalog.spec.js`)
- ✅ Catalog page header and statistics
- ✅ Manufacturer card display and content
- ✅ "View Details" buttons on manufacturer cards
- ✅ "Submit RFQ" buttons on manufacturer cards
- ✅ CTA section "Submit RFQ" button
- ✅ Mobile responsiveness
- ✅ Multiple catalog pages (machining, molding, fabrication)

### RFQ Form Tests (`tests/rfq-form.spec.js`)
- ✅ Form field validation (required fields)
- ✅ Email format validation
- ✅ "Submit RFQ" button functionality
- ✅ Loading states during submission
- ✅ Success message display with RFQ ID
- ✅ Form handling with manufacturer parameters
- ✅ Mobile responsiveness
- ✅ Redirect behavior after submission

### Manufacturer Detail Tests (`tests/manufacturer-details.spec.js`)
- ✅ Navigation to manufacturer detail pages
- ✅ Graceful handling of non-existent pages
- ✅ Filter controls component testing (when implemented)

### API Integration Tests (`tests/api-integration.spec.js`)
- ✅ Upload API integration testing
- ✅ Manufacturer API data loading
- ✅ RFQ API submission workflow
- ✅ Email notification system testing
- ✅ LocalStack S3 integration verification

## Architecture

### Test Environment Stack
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Playwright    │────│    Frontend     │────│   Backend APIs  │
│  E2E Tests      │    │   (Astro)       │    │  (Rust/Axum)    │
│                 │    │  Port: 4321     │    │  Ports: 3000-3002│
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                                └───────────────────────┼─────────────────┐
                                                        │                 │
                               ┌─────────────────┐    ┌─────────────────┐
                               │   LocalStack    │    │   Test Data     │
                               │  (AWS Mocks)    │    │   Setup         │
                               │  Port: 4566     │    │                 │
                               └─────────────────┘    └─────────────────┘
```

### Services
- **LocalStack**: Provides S3 and SES mocking on port 4566
- **Frontend**: Astro development server on port 4321
- **API RFQs**: RFQ service on port 3001
- **API Uploads**: File upload service on port 3000  
- **API Manufacturers**: Manufacturer management on port 3002

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Node.js 18+ (for local test development)

### Run All Tests
```bash
./run-integration-tests.sh
```

### Run Tests with Browser Visible
```bash
./run-integration-tests.sh --headed
```

### Setup Environment Only (for development)
```bash
./run-integration-tests.sh --setup-only --no-cleanup
```

### Test Specific Browser
```bash
./run-integration-tests.sh --browser firefox
```

## Manual Test Execution

If you prefer to run tests manually:

1. **Start the test environment:**
   ```bash
   docker-compose -f docker-compose.test.yml up -d
   ```

2. **Setup test data:**
   ```bash
   cd tests
   npm install
   node setup-test-data.js
   ```

3. **Run tests:**
   ```bash
   npm test                    # Run all tests
   npm run test:headed        # Run with visible browser
   npm run test:debug         # Run with debugging
   ```

4. **Cleanup:**
   ```bash
   docker-compose -f docker-compose.test.yml down --volumes
   ```

## Test Development

### Adding New Tests

1. Create a new test file in `tests/tests/` following the naming pattern `*.spec.js`
2. Use the existing test files as templates
3. Follow these patterns for comprehensive button testing:

```javascript
test('should test [button name] button', async ({ page }) => {
  await page.goto('/page-url');
  
  // Verify button exists and is visible
  const button = page.locator('button-selector');
  await expect(button).toBeVisible();
  
  // Test button interaction
  await button.click();
  
  // Verify expected outcome
  await expect(page).toHaveURL(/expected-url/);
  // or
  await expect(page.locator('result-selector')).toBeVisible();
});
```

### Testing New Features

When adding new buttons or functionality:

1. Add a test case in the appropriate spec file
2. Ensure the test covers:
   - Button visibility and accessibility  
   - Click interaction
   - Expected navigation or state change
   - Loading states if applicable
   - Error handling
   - Mobile responsiveness

### Mock Data

Test data is defined in `setup-test-data.js`. To add new test data:

1. Update the `sampleManufacturers` array
2. Add new catalog categories as needed
3. Run `node setup-test-data.js` to populate LocalStack

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Integration Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Integration Tests
        run: ./run-integration-tests.sh
      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: tests/test-results/
```

## Troubleshooting

### Common Issues

**Tests fail with "LocalStack not ready"**
- Increase timeout in `run-integration-tests.sh`
- Check Docker resources allocation
- Verify port 4566 is not in use

**Frontend not loading**
- Check that port 4321 is available
- Verify frontend build completed successfully
- Check Docker container logs: `docker logs terra_frontend`

**Backend APIs not responding**
- Check individual service health endpoints
- Verify environment variables are set correctly
- Check service logs: `docker logs terra_api-rfqs`

**Tests are flaky**
- Increase timeouts in test files
- Add explicit waits for async operations
- Check network request mocking

### Debug Mode

Run tests in debug mode to step through issues:
```bash
cd tests
npx playwright test --debug --headed
```

### Viewing Logs

Check service logs during test execution:
```bash
# All services
docker-compose -f docker-compose.test.yml logs

# Specific service
docker-compose -f docker-compose.test.yml logs api-rfqs
```

## Test Reports

Test results are available in multiple formats:
- **HTML Report**: `tests/playwright-report/index.html`
- **JSON Results**: `tests/test-results/`
- **Screenshots**: Captured on failure in `test-results/`
- **Videos**: Recorded on failure in `test-results/`

## Contributing

When contributing new tests:
1. Follow existing naming and structure conventions
2. Test both happy path and error scenarios  
3. Include mobile responsiveness tests
4. Add appropriate assertions for accessibility
5. Update this README if adding new test categories

## Performance Considerations

- Tests run in parallel by default (per browser)
- Use `test.describe.serial()` for tests that must run in order
- Consider resource limits when running full test suite
- LocalStack may need more memory for complex scenarios
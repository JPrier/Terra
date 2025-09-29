import { test, expect } from '@playwright/test';

test.describe('S3 HTML Prefetching', () => {

  test('should initialize prefetching script on page load', async ({ page }) => {
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.type() === 'debug' && msg.text().includes('S3 HTML prefetching')) {
        consoleMessages.push(msg.text());
      }
    });

    await page.goto('/');
    
    // Wait a moment for the script to initialize and run
    await page.waitForTimeout(1000);
    
    // Should have initialized
    expect(consoleMessages.some(msg => msg.includes('S3 HTML prefetching initialized'))).toBeTruthy();
    
    // Should have found internal links
    expect(consoleMessages.some(msg => msg.includes('Found') && msg.includes('internal links to prefetch'))).toBeTruthy();
  });

  test('should scan and attempt to prefetch internal links', async ({ page }) => {
    const networkRequests = [];
    page.on('request', request => {
      // Capture prefetch requests to S3
      if (request.url().includes('/test-data/catalog/')) {
        networkRequests.push({
          url: request.url(),
          method: request.method()
        });
      }
    });

    await page.goto('/');
    
    // Wait for prefetching to complete
    await page.waitForTimeout(2000);
    
    // Should have attempted to prefetch category pages
    expect(networkRequests.length).toBeGreaterThan(0);
    expect(networkRequests.some(req => req.url.includes('/catalog/machining/'))).toBeTruthy();
    expect(networkRequests.some(req => req.url.includes('/catalog/molding/'))).toBeTruthy();
    expect(networkRequests.some(req => req.url.includes('/catalog/fabrication/'))).toBeTruthy();
    
    // All should be GET requests
    networkRequests.forEach(req => {
      expect(req.method).toBe('GET');
    });
  });

  test('should not prefetch non-S3 routes', async ({ page }) => {
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.type() === 'debug' && msg.text().includes('Skipping prefetch')) {
        consoleMessages.push(msg.text());
      }
    });

    await page.goto('/');
    await page.waitForTimeout(1000);
    
    // Should skip routes that don't map to S3
    expect(consoleMessages.some(msg => msg.includes('/ - no S3 mapping'))).toBeTruthy();
    expect(consoleMessages.some(msg => msg.includes('/rfq/submit - no S3 mapping'))).toBeTruthy();
  });

  test('should expose prefetch utility functions globally', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
    
    // Check that global utility functions are available
    const hasTerraS3Prefetch = await page.evaluate(() => {
      return typeof window.TerraS3Prefetch === 'object' &&
             typeof window.TerraS3Prefetch.prefetchRoute === 'function' &&
             typeof window.TerraS3Prefetch.prefetchPageLinks === 'function' &&
             typeof window.TerraS3Prefetch.mapRouteToS3Path === 'function';
    });
    
    expect(hasTerraS3Prefetch).toBeTruthy();
  });

  test('should map routes to S3 paths correctly', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
    
    const mappings = await page.evaluate(() => {
      const { mapRouteToS3Path } = window.TerraS3Prefetch;
      return {
        home: mapRouteToS3Path('/'),
        category: mapRouteToS3Path('/catalog/machining/'),
        manufacturer: mapRouteToS3Path('/catalog/manufacturer/acme-precision/'),
        rfq: mapRouteToS3Path('/rfq/submit'),
        api: mapRouteToS3Path('/api/categories.json')
      };
    });
    
    expect(mappings.home).toBeNull(); // Home page doesn't need S3 mapping
    expect(mappings.category).toBe('/test-data/catalog/machining/index.html');
    expect(mappings.manufacturer).toBe('/test-data/catalog/manufacturer/acme-precision/index.html');
    expect(mappings.rfq).toBeNull(); // RFQ pages are static
    expect(mappings.api).toBeNull(); // API routes should be skipped
  });

  test('should handle dynamic content changes', async ({ page }) => {
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.type() === 'debug' && msg.text().includes('Found') && msg.text().includes('internal links')) {
        consoleMessages.push(msg.text());
      }
    });

    await page.goto('/');
    await page.waitForTimeout(1000);
    
    // Initial scan
    expect(consoleMessages.length).toBeGreaterThan(0);
    const initialMessage = consoleMessages[consoleMessages.length - 1];
    
    // Add a new link dynamically
    await page.evaluate(() => {
      const newLink = document.createElement('a');
      newLink.href = '/catalog/electronics/';
      newLink.textContent = 'Electronics';
      document.body.appendChild(newLink);
    });
    
    // Wait for mutation observer to trigger
    await page.waitForTimeout(1000);
    
    // Should have triggered another scan
    expect(consoleMessages.length).toBeGreaterThan(1);
    
    // The new scan should find more links (or at least re-scan)
    const finalMessage = consoleMessages[consoleMessages.length - 1];
    expect(finalMessage).toBeDefined();
  });

  test('should include S3 base URL meta tag', async ({ page }) => {
    await page.goto('/');
    
    const s3BaseUrl = await page.getAttribute('meta[name="s3-base-url"]', 'content');
    expect(s3BaseUrl).toBeDefined();
    expect(s3BaseUrl).toBeTruthy();
  });

});
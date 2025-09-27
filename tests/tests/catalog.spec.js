import { test, expect } from '@playwright/test';

test.describe('Catalog Browsing and Filtering', () => {
  
  test.describe('CNC Machining Catalog', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/catalog/machining/');
    });

    test('should display catalog page header and stats', async ({ page }) => {
      await expect(page.locator('h2')).toContainText('CNC Machining Manufacturers');
      await expect(page.locator('p').first()).toContainText('Find verified US-based CNC machining manufacturers');
      
      // Check catalog stats
      const statsSection = page.locator('.catalog-stats');
      await expect(statsSection).toBeVisible();
      await expect(statsSection).toContainText('verified manufacturers in CNC Machining');
    });

    test('should display manufacturer cards', async ({ page }) => {
      // Should have manufacturer grid
      const manufacturerGrid = page.locator('.manufacturer-grid');
      await expect(manufacturerGrid).toBeVisible();
      
      // Should have at least one manufacturer card
      const manufacturerCards = page.locator('.manufacturer-card');
      await expect(manufacturerCards.first()).toBeVisible();
      
      // Verify card content structure
      const firstCard = manufacturerCards.first();
      await expect(firstCard.locator('h3 a')).toBeVisible();
      await expect(firstCard.locator('.location')).toBeVisible();
      await expect(firstCard.locator('.categories')).toBeVisible();
      await expect(firstCard.locator('.actions')).toBeVisible();
    });

    test('should have working "View Details" buttons', async ({ page }) => {
      const firstViewDetailsBtn = page.locator('.manufacturer-card .actions').first().getByRole('link', { name: 'View Details' });
      await expect(firstViewDetailsBtn).toBeVisible();
      
      // Check that the link has proper href
      const href = await firstViewDetailsBtn.getAttribute('href');
      expect(href).toMatch(/\/catalog\/manufacturer\/.*\//);
      
      // Click and verify navigation would work (even if page doesn't exist yet)
      await firstViewDetailsBtn.click();
      await expect(page).toHaveURL(href);
    });

    test('should have working "Submit RFQ" buttons in cards', async ({ page }) => {
      const firstSubmitRfqBtn = page.locator('.manufacturer-card .actions').first().getByRole('link', { name: 'Submit RFQ' });
      await expect(firstSubmitRfqBtn).toBeVisible();
      
      // Check that the link goes to RFQ form with manufacturer parameter
      const href = await firstSubmitRfqBtn.getAttribute('href');
      expect(href).toMatch(/\/rfq\/submit\?mfg=.*/);
      
      await firstSubmitRfqBtn.click();
      await expect(page).toHaveURL(/.*rfq\/submit.*/);
      await expect(page.locator('h2')).toContainText('Submit Request for Quote');
    });

    test('should have CTA section with RFQ button', async ({ page }) => {
      const ctaSection = page.locator('.cta-section');
      await expect(ctaSection).toBeVisible();
      await expect(ctaSection.locator('h3')).toContainText("Don't see what you need?");
      
      const ctaButton = ctaSection.getByRole('link', { name: 'Submit RFQ' });
      await expect(ctaButton).toBeVisible();
      await ctaButton.click();
      
      await expect(page).toHaveURL(/.*rfq\/submit/);
    });

    test('should be mobile responsive', async ({ page }) => {
      // Test mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });
      
      const manufacturerGrid = page.locator('.manufacturer-grid');
      await expect(manufacturerGrid).toBeVisible();
      
      // Cards should stack vertically on mobile
      const firstCard = page.locator('.manufacturer-card').first();
      await expect(firstCard).toBeVisible();
    });
  });

  test.describe('Other Catalog Pages', () => {
    test('should display injection molding catalog', async ({ page }) => {
      await page.goto('/catalog/molding/');
      
      // May not have content yet, but should load without error
      await expect(page).not.toHaveTitle(/404|Error/);
    });

    test('should display fabrication catalog', async ({ page }) => {
      await page.goto('/catalog/fabrication/');
      
      // May not have content yet, but should load without error  
      await expect(page).not.toHaveTitle(/404|Error/);
    });
  });
});
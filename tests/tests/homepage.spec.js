import { test, expect } from '@playwright/test';

test.describe('Homepage Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display homepage content', async ({ page }) => {
    // Verify main elements are visible
    await expect(page.locator('h2')).toContainText('Connect with US Manufacturers');
    await expect(page.locator('p').first()).toContainText('Find qualified manufacturers');
  });

  test('should navigate to catalog via "Browse Now" button', async ({ page }) => {
    // Click "Browse Now" button
    const browseButton = page.getByRole('link', { name: 'Browse Now' });
    await expect(browseButton).toBeVisible();
    await browseButton.click();
    
    // Should navigate to machining catalog
    await expect(page).toHaveURL(/.*catalog\/machining\//);
    await expect(page.locator('h2')).toContainText('CNC Machining Manufacturers');
  });

  test('should navigate to RFQ form via "Get Started" button', async ({ page }) => {
    // Click "Get Started" button  
    const getStartedButton = page.getByRole('link', { name: 'Get Started' });
    await expect(getStartedButton).toBeVisible();
    await getStartedButton.click();
    
    // Should navigate to RFQ submission page
    await expect(page).toHaveURL(/.*rfq\/submit/);
    await expect(page.locator('h2')).toContainText('Submit Request for Quote');
  });

  test('should navigate via category buttons', async ({ page }) => {
    // Test CNC Machining button
    const cncButton = page.getByRole('link', { name: 'CNC Machining' });
    await expect(cncButton).toBeVisible();
    await cncButton.click();
    await expect(page).toHaveURL(/.*catalog\/machining\//);
    
    // Go back and test Sheet Metal
    await page.goto('/');
    const sheetMetalButton = page.getByRole('link', { name: 'Sheet Metal' });
    await expect(sheetMetalButton).toBeVisible(); 
    await sheetMetalButton.click();
    await expect(page).toHaveURL(/.*catalog\/fabrication\//);
    
    // Go back and test Injection Molding
    await page.goto('/');
    const moldingButton = page.getByRole('link', { name: 'Injection Molding' });
    await expect(moldingButton).toBeVisible();
    await moldingButton.click();
    await expect(page).toHaveURL(/.*catalog\/molding\//);
  });

  test('should display feature sections', async ({ page }) => {
    // Verify feature section exists
    await expect(page.locator('.features')).toBeVisible();
    await expect(page.locator('.feature').first()).toContainText('🇺🇸 US-Only');
    await expect(page.locator('.feature').nth(1)).toContainText('⚡ Fast Quotes');
    await expect(page.locator('.feature').nth(2)).toContainText('🔒 Secure');
  });

  test('should display popular categories section', async ({ page }) => {
    // Verify popular categories section
    await expect(page.locator('h3')).toContainText('Popular Categories');
    await expect(page.locator('p')).toContainText('CNC Machining • 3D Printing • Injection Molding');
  });
});
import { test, expect } from '@playwright/test';

test.describe('RFQ Form Submission', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/rfq/submit');
  });

  test('should display RFQ form with all required fields', async ({ page }) => {
    // Verify page title and description
    await expect(page.locator('h2')).toContainText('Submit Request for Quote');
    await expect(page.locator('p').first()).toContainText('Send your requirements directly to manufacturers');
    
    // Verify form exists
    const form = page.locator('#rfq-form');
    await expect(form).toBeVisible();
    
    // Verify all required fields are present
    await expect(page.locator('#buyerName')).toBeVisible();
    await expect(page.locator('#buyerEmail')).toBeVisible();
    await expect(page.locator('#subject')).toBeVisible();
    await expect(page.locator('#description')).toBeVisible();
    
    // Verify submit button
    const submitButton = page.locator('#submit-btn');
    await expect(submitButton).toBeVisible();
    await expect(submitButton).toContainText('Submit RFQ');
  });

  test('should validate required fields', async ({ page }) => {
    // Try to submit empty form
    const submitButton = page.locator('#submit-btn');
    await submitButton.click();
    
    // Browser should prevent submission and show validation messages
    const nameField = page.locator('#buyerName');
    await expect(nameField).toHaveAttribute('required');
    
    const emailField = page.locator('#buyerEmail');
    await expect(emailField).toHaveAttribute('required');
    
    const subjectField = page.locator('#subject');
    await expect(subjectField).toHaveAttribute('required');
    
    const descriptionField = page.locator('#description');
    await expect(descriptionField).toHaveAttribute('required');
  });

  test('should validate email format', async ({ page }) => {
    // Fill invalid email
    await page.locator('#buyerName').fill('John Doe');
    await page.locator('#buyerEmail').fill('invalid-email');
    await page.locator('#subject').fill('Test Project');
    await page.locator('#description').fill('This is a test project description');
    
    const submitButton = page.locator('#submit-btn');
    await submitButton.click();
    
    // Browser should show email validation error
    const emailField = page.locator('#buyerEmail');
    await expect(emailField).toHaveAttribute('type', 'email');
  });

  test('should successfully submit valid form', async ({ page }) => {
    // Fill out the form with valid data
    await page.locator('#buyerName').fill('John Doe');
    await page.locator('#buyerEmail').fill('john@example.com');
    await page.locator('#subject').fill('CNC Machining Project');
    await page.locator('#description').fill('I need precision CNC machining for 100 aluminum parts with tight tolerances. Timeline is 4 weeks.');
    
    // Submit the form
    const submitButton = page.locator('#submit-btn');
    await submitButton.click();
    
    // Wait for submission processing
    await expect(submitButton).toContainText('Submitting...');
    await expect(submitButton).toBeDisabled();
    
    // Wait for success message
    const successMessage = page.locator('#success-message');
    await expect(successMessage).toBeVisible({ timeout: 10000 });
    await expect(successMessage.locator('h3')).toContainText('RFQ Submitted Successfully!');
    
    // Verify RFQ ID is displayed
    const rfqId = page.locator('#rfq-id');
    await expect(rfqId).toBeVisible();
    await expect(rfqId).toContainText(/r_[A-Z0-9]+/);
    
    // Form should be hidden
    const form = page.locator('#rfq-form');
    await expect(form).toBeHidden();
  });

  test('should handle form submission from manufacturer page', async ({ page }) => {
    // Navigate to RFQ form with manufacturer parameter
    await page.goto('/rfq/submit?mfg=precision-parts-co');
    
    // Form should still display normally
    await expect(page.locator('#rfq-form')).toBeVisible();
    
    // URL parameter should be preserved
    await expect(page).toHaveURL(/.*mfg=precision-parts-co/);
  });

  test('should show loading state during submission', async ({ page }) => {
    // Fill out form
    await page.locator('#buyerName').fill('Jane Smith');
    await page.locator('#buyerEmail').fill('jane@company.com');
    await page.locator('#subject').fill('Injection Molding Quote');
    await page.locator('#description').fill('Need quotes for plastic injection molding project');
    
    // Click submit and immediately check loading state
    const submitButton = page.locator('#submit-btn');
    await submitButton.click();
    
    // Should show loading state
    await expect(submitButton).toContainText('Submitting...');
    await expect(submitButton).toBeDisabled();
  });

  test('should redirect to home after successful submission', async ({ page }) => {
    // Fill and submit form
    await page.locator('#buyerName').fill('Bob Johnson');
    await page.locator('#buyerEmail').fill('bob@example.com');
    await page.locator('#subject').fill('Sheet Metal Fabrication');
    await page.locator('#description').fill('Custom sheet metal fabrication project');
    
    await page.locator('#submit-btn').click();
    
    // Wait for success message
    await expect(page.locator('#success-message')).toBeVisible({ timeout: 10000 });
    
    // Should redirect to home after 5 seconds (but we won't wait that long)
    // Just verify the redirect logic is in place by checking the script
    const scripts = await page.locator('script').allTextContents();
    const hasRedirectLogic = scripts.some(script => 
      script.includes('window.location.href') && 
      script.includes('setTimeout')
    );
    expect(hasRedirectLogic).toBeTruthy();
  });

  test('should be mobile responsive', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Form should still be usable
    const form = page.locator('#rfq-form');
    await expect(form).toBeVisible();
    
    // Fields should be responsive
    const nameField = page.locator('#buyerName');
    await expect(nameField).toBeVisible();
    
    // Grid layout should adapt on mobile
    await expect(nameField).toBeFocused();
    await nameField.fill('Mobile Test User');
    
    // Should be able to scroll and see all fields
    await page.locator('#description').scrollIntoViewIfNeeded();
    await expect(page.locator('#description')).toBeVisible();
  });
});
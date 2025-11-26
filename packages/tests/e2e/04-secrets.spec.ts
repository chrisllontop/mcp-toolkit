import { test, expect, Locator } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Test suite for Secrets Management
 */
test.describe('Secrets Management', () => {
  let addSecretHeading: Locator;
  let addSecretButton: Locator;
  let secretsHeading: Locator;
  let apiKeyInput: Locator;
  let secretValueInput: Locator;
  let saveButton: Locator;
  /**
   * Setup test
   */
  test.beforeEach(async ({ page }) => {
    // Mock Tauri IPC
    const mockContent = fs.readFileSync(path.resolve(__dirname, 'mocks/tauri-mock.js'), 'utf-8');
    await page.addInitScript(mockContent);

    await page.goto('/secrets');
    await page.waitForLoadState('networkidle');

    secretsHeading = page.getByRole('heading', { name: 'Secrets' });
    addSecretHeading = page.getByRole('heading', { name: 'Add Secret' });
    addSecretButton = page.getByRole('button', { name: 'Add Secret' });
    apiKeyInput = page.getByRole('textbox', { name: 'API_KEY' });
    secretValueInput = page.getByRole('textbox', { name: 'secret value' });
    saveButton = page.getByRole('button', { name: 'Save' });
  });

  test('should display secrets page', async ({ page }) => {
    await expect(secretsHeading).toContainText('Secrets');
    await expect(addSecretButton).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Key' })).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Created At' })).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Status' })).toBeVisible();
  });

  test('should add a new secret', async ({ page }) => {
    await addSecretButton.click();
    await expect(addSecretHeading).toContainText('Add Secret');
    
    // Fill in secret details
    await apiKeyInput.fill('TEST_API_KEY');
    await secretValueInput.fill('super_secret_value_123');
    
    // Save
    await saveButton.click();
    
    // Verify secret appears in table
    await expect(page.getByRole('cell', { name: 'TEST_API_KEY' })).toHaveText('TEST_API_KEY');
  });

  test('should not display secret values in the table', async ({ page }) => {
    // Add a secret
    await addSecretButton.click();
    await apiKeyInput.fill('HIDDEN_SECRET');
    await secretValueInput.fill('this_should_not_be_visible');
    await saveButton.click();
    
    // Verify the secret value is NOT visible in the table
    const tableContent = await page.locator('table').textContent();
    expect(tableContent).not.toContain('this_should_not_be_visible');
    
    // But the key should be visible
    expect(tableContent).toContain('HIDDEN_SECRET');
  });

  test('should display created timestamp for secrets', async ({ page }) => {
    await addSecretButton.click();
    await apiKeyInput.fill('TIMESTAMP_TEST');
    await secretValueInput.fill('value');
    await saveButton.click();
    // await page.waitForTimeout(1000);
    
    // The created_at should be displayed
    const row = page.locator('table tr:has-text("TIMESTAMP_TEST")');
    await expect(row).toBeVisible();
    
    // Check if timestamp is present (basic check)
    const rowText = await row.textContent();

    // Should contain date/time pattern
    expect(rowText).toMatch(/\d{1,2}\/\d{1,2}\/\d{4}|\d{4}-\d{2}-\d{2}/);
  });

  test('should cancel secret creation', async ({ page }) => {
    await page.click('button:has-text("Add Secret")');
    
    await page.fill('input[placeholder="API_KEY"]', 'CANCELLED_SECRET');
    
    // Cancel
    await page.click('.modal button:has-text("Cancel")');
    
    // Modal should close
    await expect(page.locator('.modal')).not.toBeVisible();
    
    // Secret should not appear
    await expect(page.locator('table td:has-text("CANCELLED_SECRET")')).not.toBeVisible();
  });

  test('should handle multiple secrets', async ({ page }) => {
    const secrets = [
      { key: 'GITHUB_TOKEN', value: 'ghp_123456' },
      { key: 'OPENAI_KEY', value: 'sk-123456' },
      { key: 'DATABASE_URL', value: 'postgresql://...' }
    ];
    
    for (const secret of secrets) {
      await addSecretButton.click();
      await apiKeyInput.fill(secret.key);
      await secretValueInput.fill(secret.value);
      await saveButton.click();
      await page.waitForTimeout(500); // Small delay between creations
    }
    
    // Verify all secrets appear
    for (const secret of secrets) {
      await expect(page.locator(`table td:has-text("${secret.key}")`)).toBeVisible();
    }
  });

  test('should validate empty secret key', async ({ page }) => { // failing
    await page.click('button:has-text("Add Secret")');
    
    // Try to save without filling key
    await page.fill('input[type="password"]', 'value_without_key');
    await page.click('.modal button:has-text("Save")');
    
    // Modal should remain open (validation should prevent save)
    await expect(page.locator('.modal')).toBeVisible();
  });

  test('should validate empty secret value', async ({ page }) => { // failing
    await page.click('button:has-text("Add Secret")');
    
    // Try to save without filling value
    await page.fill('input[placeholder="API_KEY"]', 'KEY_WITHOUT_VALUE');
    await page.click('.modal button:has-text("Save")');
    
    // Modal should remain open
    await expect(page.locator('.modal')).toBeVisible();
  });

  test('should handle special characters in secret values', async ({ page }) => {
    await addSecretButton.click();
    
    await apiKeyInput.fill('SPECIAL_CHARS_SECRET');
    // Test with special characters
    await secretValueInput.fill('p@$$w0rd!#%^&*(){}[]|\\:;"<>?,./');
    
    // await page.click('.modal button:has-text("Save")');
    await saveButton.click();
    
    // Should save successfully
    await expect(page.locator('table td:has-text("SPECIAL_CHARS_SECRET")')).toBeVisible();
  });

  test('should display secrets table headers correctly', async ({ page }) => {
    const headers = await page.locator('table thead th').allTextContents();
    
    expect(headers).toContain('Key');
    expect(headers).toContain('Created At');
    expect(headers).toContain('Status');
  });

  test('should handle secret key updates (if supported)', async ({ page }) => {
    // Add initial secret
    await addSecretButton.click();
    await apiKeyInput.fill('UPDATE_TEST');
    await secretValueInput.fill('original_value');
    await saveButton.click();
    
    // Try to add again with same key (should update or error)
    await addSecretButton.click();
    await apiKeyInput.fill('UPDATE_TEST');
    await secretValueInput.fill('updated_value');
    await saveButton.click();
    
    // Check if there's only one entry with that key
    const keyCount = await page.locator('table td:has-text("UPDATE_TEST")').count();
    
    if (keyCount === 1) {
      console.log('✅ Secret update/overwrite: SUPPORTED');
    } else if (keyCount > 1) {
      console.log('⚠️  Secret update: Creates duplicate entries');
    }
  });
});

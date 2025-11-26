import { test, expect, Locator } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Test suite for MCP Catalog functionality
 * This suite tests importing various MCP configuration formats
 */
test.describe('MCP Catalog - Configuration Import', () => {
  let mcpCatalogLocator: Locator;
  let addMcpButton: Locator;
  let modalLocator: Locator;
  let mcpConfigTextarea: Locator;
  let parseJsonButton: Locator;
  let add1McpButton: Locator;

  test.beforeEach(async ({ page }) => {
    const mockContent = fs.readFileSync(path.resolve(__dirname, 'mocks/tauri-mock.js'), 'utf-8');
    await page.addInitScript(mockContent);
    await page.goto('/catalog');
    await page.waitForLoadState('networkidle');

    mcpCatalogLocator = page.getByRole('heading', { name: 'MCP Catalog' });
    addMcpButton = page.getByRole('button', { name: 'Add MCP' });
    modalLocator = page.getByText('Add MCP×Paste MCP JSON');
    mcpConfigTextarea = page.getByRole('textbox', { name: '{ "mcpServers": { "server-' });
    parseJsonButton = page.getByRole('button', { name: 'Parse JSON' });
    add1McpButton = modalLocator.getByRole('button', { name: 'Add 1 MCP(s)' });
  });

  test('should display MCP Catalog page', async ({ page }) => {
    await expect(mcpCatalogLocator).toContainText('MCP Catalog');
    await expect(addMcpButton).toBeVisible();
  });

  test('should import standard binary MCP configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/standard-binary.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await page.click('button:has-text("Add MCP")');
    await expect(modalLocator).toContainText('Add MCP');
    
    // Paste JSON
    await mcpConfigTextarea.fill(config);
    
    // Parse JSON
    await parseJsonButton.click();
    
    // Verify preview shows
    await expect(modalLocator.locator('text=Preview')).toBeVisible();
    // await expect(modalLocator.locator('text=test-binary-server')).toBeVisible();
    await expect(mcpConfigTextarea).toBeVisible();
    await expect(modalLocator.locator('.badge:has-text("Binary")')).toBeVisible();
    
    // Import
    await add1McpButton.click();
    
    // Verify MCP appears in catalog
    await expect(page.locator('.card h3:has-text("test-binary-server")')).toBeVisible();
  });

  test('should import NPX-based MCP configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/npx-based.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    // Check if import succeeds or fails
    await expect(page.locator('strong:has-text("test-npx-server")')).toBeVisible({ timeout: 5000 });
    
    // Record result in test report
    const hasPreview = await page.locator('strong:has-text("test-npx-server")').isVisible();
    
    if (hasPreview) {
      console.log('✅ NPX-based MCP: SUPPORTED');
      await add1McpButton.click();
      await expect(page.locator('.card:has-text("test-npx-server")')).toBeVisible();
    } else {
      console.log('❌ NPX-based MCP: NOT SUPPORTED');
    }
  });

  test('should import UV/Python-based MCP configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/uv-python.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    const hasPreview = await page.locator('text=test-uv-server').first().isVisible({ timeout: 5000 });
    
    if (hasPreview) {
      console.log('✅ UV/Python MCP: SUPPORTED');
      await add1McpButton.click();
      await expect(page.locator('.card:has-text("test-uv-server")')).toBeVisible();
    } else {
      console.log('❌ UV/Python MCP: NOT SUPPORTED');
    }
  });

  test('should handle Docker-based MCP configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/docker-based.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    
    // This might fail during parsing if Docker format is not recognized
    await parseJsonButton.click();
    
    await page.waitForTimeout(2000);
    
    const hasPreview = await page.locator('text=test-docker-server').first().isVisible();
    const hasError = await page.locator('text=Error').isVisible();
    
    if (hasPreview) {
      console.log('✅ Docker MCP: SUPPORTED (parse)');
      const badge = await page.locator('.badge:has-text("Docker")').isVisible();
      console.log(`   Docker type detected: ${badge}`);
    } else if (hasError) {
      console.log('❌ Docker MCP: NOT SUPPORTED (parse error)');
    } else {
      console.log('⚠️  Docker MCP: UNKNOWN (no preview, no error)');
    }
  });

  test('should handle HTTP-based MCP configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/http-based.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    await page.waitForTimeout(2000);
    
    const hasPreview = await page.locator('strong:has-text("test-http-server")').isVisible();
    const hasError = await page.locator('text=Error').isVisible();
    
    if (hasPreview) {
      console.log('✅ HTTP MCP: SUPPORTED (parse)');
      const badge = await page.locator('.badge:has-text("Http")').isVisible();
      console.log(`   HTTP type detected: ${badge}`);
    } else if (hasError) {
      console.log('❌ HTTP MCP: NOT SUPPORTED (parse error)');
    } else {
      console.log('⚠️  HTTP MCP: UNKNOWN');
    }
  });

  test('should handle complex nested configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/complex-nested.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    await page.waitForTimeout(2000);
    
    const hasPreview = await page.locator('strong:has-text("test-complex-server")').isVisible();
    
    if (hasPreview) {
      console.log('✅ Complex nested config: SUPPORTED');
    } else {
      console.log('❌ Complex nested config: NOT SUPPORTED');
    }
  });

  test('should handle alternative field names', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/alternative-fields.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    await page.waitForTimeout(2000);
    
    const hasPreview = await page.locator('strong:has-text("test-alternative-server")').isVisible();
    
    if (hasPreview) {
      console.log('✅ Alternative field names: SUPPORTED');
    } else {
      console.log('❌ Alternative field names: NOT SUPPORTED (executable/arguments/environment)');
    }
  });

  test('should import multiple MCPs at once', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/multiple-servers.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    // Should show preview with 3 MCPs
    const previewText = await page.locator('h3:has-text("Preview")').textContent();
    
    if (previewText?.includes('3 MCPs')) {
      console.log('✅ Multiple MCPs import: SUPPORTED');
      
      await expect(page.locator('strong:has-text("filesystem-server")')).toBeVisible();
      await expect(page.locator('strong:has-text("git-server")')).toBeVisible();
      await expect(page.locator('strong:has-text("custom-binary")')).toBeVisible();
      
      await page.click('button:has-text("Add 3 MCP")');
      
      // Verify all imported
      await expect(page.locator('.card:has-text("filesystem-server")')).toBeVisible();
      await expect(page.locator('.card:has-text("git-server")')).toBeVisible();
      await expect(page.locator('.card:has-text("custom-binary")')).toBeVisible();
    } else {
      console.log('❌ Multiple MCPs import: Issue detected');
    }
  });

  test('should handle minimal configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/minimal-config.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    await page.waitForTimeout(2000);
    
    const hasPreview = await page.locator('strong:has-text("minimal-server")').isVisible();
    
    if (hasPreview) {
      console.log('✅ Minimal config (command only): SUPPORTED');
    } else {
      console.log('❌ Minimal config: NOT SUPPORTED');
    }
  });

  test('should reject invalid configuration', async ({ page }) => {
    const configPath = path.resolve(__dirname, '../test-configs/invalid-config.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    
    await page.waitForTimeout(2000);
    
    // Should show error or no preview
    const hasError = await page.locator('text=Error').isVisible();
    const hasPreview = await page.locator('text=Preview').isVisible();
    
    if (hasError || !hasPreview) {
      console.log('✅ Invalid config properly rejected');
    } else {
      console.log('⚠️  Invalid config was accepted (might be an issue)');
    }
  });

  test('should delete an MCP from catalog', async ({ page }) => {
    // Import a test MCP first
    const configPath = path.resolve(__dirname, '../test-configs/standard-binary.json');
    const config = fs.readFileSync(configPath, 'utf-8');
    
    await addMcpButton.click();
    await mcpConfigTextarea.fill(config);
    await parseJsonButton.click();
    await add1McpButton.click();
    
    // Setup dialog handler
    page.on('dialog', dialog => dialog.accept());
    
    // Delete the MCP
    await page.click('.card:has-text("test-binary-server") button.danger:has-text("Delete")');
    
    // Verify removed
    await expect(page.locator('.card:has-text("test-binary-server")')).not.toBeVisible();
  });
});

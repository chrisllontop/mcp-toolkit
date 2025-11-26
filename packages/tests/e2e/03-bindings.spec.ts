import { test, expect, Locator } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// ES Module compatibility: Create __dirname equivalent
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Test suite for Project-MCP Bindings
 * Using serial mode to ensure tests run in order and share state
 */
test.describe.serial('Project-MCP Bindings', () => {
  let sharedPage: any; // Shared page across all tests
  let projectCreated = false;
  let mcpImported = false;

  test.beforeAll(async ({ browser }) => {
    // Create a page that will be shared across all tests
    sharedPage = await browser.newPage();
    
    let mockContent = fs.readFileSync(path.resolve(__dirname, 'mocks/tauri-mock.js'), 'utf-8');
    
    // 1. Make state persistent using localStorage
    mockContent = mockContent.replace(
      /const state = \{[\s\S]*?\};/,
      `const defaultState = { projects: [], mcps: [], bindings: [], secrets: [] };
       const saved = localStorage.getItem('tauri_mock_state');
       const state = saved ? JSON.parse(saved) : defaultState;
       const saveState = () => localStorage.setItem('tauri_mock_state', JSON.stringify(state));`
    );

    // 2. Wrap invoke to save state after every call
    mockContent = mockContent.replace(
      /window\.__TAURI_INTERNALS__ = \{ invoke: mockInvoke \};[\s\S]*?\}\)\(\);/,
      `const persistentInvoke = async (cmd, args) => {
        try {
          const result = await mockInvoke(cmd, args);
          saveState();
          return result;
        } catch (e) {
          throw e;
        }
      };
      window.__TAURI_INTERNALS__ = { invoke: persistentInvoke };
      window.__TAURI__ = { invoke: persistentInvoke };
    })();`
    );

    await sharedPage.addInitScript(mockContent);
    
    // Create a test project
    await sharedPage.goto('/');
    await sharedPage.waitForLoadState('networkidle');
    
    await sharedPage.click('button:has-text("Add Project")');
    await sharedPage.waitForSelector('.modal', { state: 'visible' });
    await sharedPage.fill('input[placeholder="My Project"]', 'Binding Test Project');
    await sharedPage.fill('input[placeholder="/path/to/project"]', '/tmp/binding-test');
    await sharedPage.click('.modal button:has-text("Create")');
    
    // Wait for modal to close and project to be created
    await sharedPage.waitForSelector('.modal', { state: 'hidden', timeout: 5000 }).catch(() => {});
    await sharedPage.waitForTimeout(1000);
    projectCreated = true;
    
    // Import an MCP
    await sharedPage.goto('/catalog');
    await sharedPage.waitForLoadState('networkidle');
    
    const configPath = path.resolve(__dirname, '../test-configs/standard-binary.json');
    const config = fs.readFileSync(configPath, 'utf-8');

    await sharedPage.click('button:has-text("Add MCP")');
    await sharedPage.fill('textarea', config);
    await sharedPage.click('button:has-text("Parse JSON")');
    await sharedPage.click('button:has-text("Add 1 MCP")');
    
    // Wait for modal to close
    await sharedPage.waitForSelector('.modal', { state: 'hidden', timeout: 5000 }).catch(() => {});
    await sharedPage.waitForTimeout(1000);
    mcpImported = true;
  });

  test.afterAll(async () => {
    if (sharedPage) {
      await sharedPage.close();
    }
  });

  test.beforeEach(async () => {
    // Navigate to project detail page using the shared page
    await sharedPage.goto('/');
    await sharedPage.waitForLoadState('networkidle');
    
    // Wait for the project card to appear and click it
    const projectCard = sharedPage.locator('.card', { hasText: 'Binding Test Project' });
    await projectCard.waitFor({ state: 'visible', timeout: 10000 });
    await projectCard.click();
    await sharedPage.waitForLoadState('networkidle');
  });

  test('should display project configuration page', async () => {
    await expect(sharedPage.getByRole('heading', { name: 'Project Configuration' })).toBeVisible();
    await expect(sharedPage.locator('button:has-text("Activate MCP")')).toBeVisible();
  });

  test('should activate an MCP for the project', async () => {
    await sharedPage.click('button:has-text("Activate MCP")');
    
    // Select MCP from dropdown
    await sharedPage.selectOption('select', { label: 'test-binary-server' });
    
    // Activate
    await sharedPage.click('.modal button:has-text("Activate")');
    
    // Verify binding appears
    await expect(sharedPage.locator('.card:has-text("test-binary-server")')).toBeVisible();
    await expect(sharedPage.locator('.badge:has-text("Enabled")')).toBeVisible();
  });

  test('should disable an MCP binding', async () => {
    // Assuming MCP is already activated from previous test
    const enabledBadge = sharedPage.locator('.badge:has-text("Enabled")');
    
    if (await enabledBadge.isVisible()) {
      // Click Enable/Disable button
      await sharedPage.click('.card:has-text("test-binary-server") button:has-text("Disable")');
      
      // Verify it's disabled
      await expect(sharedPage.locator('.badge:has-text("Disabled")')).toBeVisible();
      
      // Re-enable for other tests
      await sharedPage.click('.card:has-text("test-binary-server") button:has-text("Enable")');
      await expect(sharedPage.locator('.badge:has-text("Enabled")')).toBeVisible();
    }
  });


  test('should edit environment variable overrides', async () => {
    // Click Config button
    await sharedPage.click('.card:has-text("test-binary-server") button:has-text("Config")');
    
    // Wait for overrides modal
    await expect(sharedPage.locator('.modal h2:has-text("Edit Overrides")')).toBeVisible();
    
    // Add an override
    await sharedPage.click('button:has-text("Add Override")');
    
    // Fill in override details
    const overrideInputs = sharedPage.locator('input[placeholder="Key"]').last();
    await overrideInputs.fill('CUSTOM_VAR');
    
    const valueInputs = sharedPage.locator('input[placeholder="Value"]').last();
    await valueInputs.fill('custom_value');
    
    // Save
    await sharedPage.click('.modal button:has-text("Save")');
    
    // Verify modal closes
    await expect(sharedPage.locator('.modal h2:has-text("Edit Overrides")')).not.toBeVisible();
  });

  test('should add secret environment variable', async () => {
    // First, add a secret to reference
    await sharedPage.goto('/secrets');
    await sharedPage.click('button:has-text("Add Secret")');
    await sharedPage.fill('input[placeholder="API_KEY"]', 'TEST_SECRET');
    await sharedPage.fill('input[type="password"]', 'secret_value_123');
    await sharedPage.click('.modal button:has-text("Save")');
    
    // Go back to project
    await sharedPage.goto('/');
    await sharedPage.click('.card:has-text("Binding Test Project")');
    
    // Edit overrides
    await sharedPage.click('.card:has-text("test-binary-server") button:has-text("Config")');
    await sharedPage.click('button:has-text("Add Override")');
    
    const keyInput = sharedPage.locator('input[placeholder="Key"]').last();
    await keyInput.fill('SECRET_KEY');
    
    const valueInput = sharedPage.locator('input[placeholder="Value"]').last();
    await valueInput.fill('TEST_SECRET');
    
    // Mark as secret
    const secretCheckbox = sharedPage.locator('input[type="checkbox"]').last();
    await secretCheckbox.check();
    
    // Save
    await sharedPage.click('.modal button:has-text("Save")');
  });

  test('should remove an environment variable override', async () => {
    // Open overrides
    await sharedPage.click('.card:has-text("test-binary-server") button:has-text("Config")');
    
    // Add an override first
    await sharedPage.click('button:has-text("Add Override")');
    await sharedPage.locator('input[placeholder="Key"]').last().fill('TEMP_VAR');
    await sharedPage.locator('input[placeholder="Value"]').last().fill('temp_value');
    
    // Count current overrides
    const initialCount = await sharedPage.locator('button.danger:has-text("Remove")').count();
    
    // Remove it
    await sharedPage.locator('button.danger:has-text("Remove")').last().click();
    
    // Verify count decreased
    const finalCount = await sharedPage.locator('button.danger:has-text("Remove")').count();
    expect(finalCount).toBe(initialCount - 1);
    
    // Save
    await sharedPage.click('.modal button:has-text("Save")');
  });

  test('should cancel override editing', async () => {
    await sharedPage.click('.card:has-text("test-binary-server") button:has-text("Config")');
    
    // Make a change
    await sharedPage.click('button:has-text("Add Override")');
    
    // Cancel
    await sharedPage.click('.modal button:has-text("Cancel")');
    
    // Modal should close
    await expect(sharedPage.locator('.modal h2:has-text("Edit Overrides")')).not.toBeVisible();
  });

  test('should list all bindings for a project', async () => {
    // Import another MCP
    await sharedPage.goto('/catalog');
    const config = {
      mcpServers: {
        'second-test-server': {
          command: '/bin/ls',
          args: ['-la']
        }
      }
    };
    
    await sharedPage.click('button:has-text("Add MCP")');
    await sharedPage.fill('textarea', JSON.stringify(config, null, 2));
    await sharedPage.click('button:has-text("Parse JSON")');
    await sharedPage.click('button:has-text("Add 1 MCP")');
    
    // Go to project and activate second MCP
    await sharedPage.goto('/');
    await sharedPage.click('.card:has-text("Binding Test Project")');
    await sharedPage.click('button:has-text("Activate MCP")');
    await sharedPage.selectOption('select', { label: 'second-test-server' });
    await sharedPage.click('.modal button:has-text("Activate")');
    
    // Verify both bindings are shown
    await expect(sharedPage.locator('.card:has-text("test-binary-server")')).toBeVisible();
    await expect(sharedPage.locator('.card:has-text("second-test-server")')).toBeVisible();
  });
});

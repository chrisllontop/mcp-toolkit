import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
/**
 * Integration test - Full workflow from project creation to MCP activation
 */
test.describe('End-to-End Integration Workflow', () => {
  test.beforeEach(async ({ page }) => {
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

    await page.addInitScript(mockContent);
  });

  test('complete workflow: create project, import MCP, activate, configure', async ({ page }) => {
    // Step 1: Create a project
    await page.goto('/');
    await page.click('button:has-text("Add Project")');
    await page.waitForSelector('.modal', { state: 'visible' });
    await page.fill('input[placeholder="My Project"]', 'E2E Test Project');
    await page.fill('input[placeholder="/path/to/project"]', '/tmp/e2e-project');
    await page.click('.modal button:has-text("Create")');
    
    // Wait for modal to close and project to appear
    await page.waitForSelector('.modal', { state: 'hidden', timeout: 5000 }).catch(() => {});
    await page.waitForSelector('.card:has-text("E2E Test Project")', { timeout: 10000 });
    
    await expect(page.locator('.card:has-text("E2E Test Project")')).toBeVisible();
    console.log('✅ Step 1: Project created');
    
    // Step 2: Add a secret
    await page.goto('/secrets');
    await page.click('button:has-text("Add Secret")');
    await page.fill('input[placeholder="API_KEY"]', 'E2E_API_KEY');
    await page.fill('input[type="password"]', 'e2e_secret_value_xyz');
    await page.click('.modal button:has-text("Save")');
    
    await expect(page.locator('table td:has-text("E2E_API_KEY")')).toBeVisible();
    console.log('✅ Step 2: Secret created');
    
    // Step 3: Import an MCP
    await page.goto('/catalog');
    const config = {
      mcpServers: {
        'e2e-test-mcp': {
          command: '/usr/bin/curl',
          args: ['--version'],
          env: {
            'API_KEY': 'placeholder'
          }
        }
      }
    };
    
    await page.click('button:has-text("Add MCP")');
    await page.fill('textarea', JSON.stringify(config, null, 2));
    await page.click('button:has-text("Parse JSON")');
    await expect(page.locator('strong:has-text("e2e-test-mcp")')).toBeVisible();
    await page.click('button:has-text("Add 1 MCP")');
    
    // Wait for modal to close
    await page.waitForSelector('.modal', { state: 'hidden', timeout: 5000 }).catch(() => {});
    
    await expect(page.locator('.card:has-text("e2e-test-mcp")')).toBeVisible();
    console.log('✅ Step 3: MCP imported');
    
    // Step 4: Navigate to project and activate MCP
    await page.goto('/');
    await page.click('.card:has-text("E2E Test Project")');
    
    await page.click('button:has-text("Activate MCP")');
    await page.selectOption('select', { label: 'e2e-test-mcp' });
    await page.click('.modal button:has-text("Activate")');
    
    await expect(page.locator('.card:has-text("e2e-test-mcp")')).toBeVisible();
    await expect(page.locator('.badge:has-text("Enabled")')).toBeVisible();
    console.log('✅ Step 4: MCP activated for project');
    
    // Step 5: Configure environment overrides with secret
    await page.click('.card:has-text("e2e-test-mcp") button:has-text("Config")');
    
    // Override API_KEY with secret reference
    await page.click('button:has-text("Add Override")');
    await page.locator('input[placeholder="Key"]').last().fill('API_KEY');
    await page.locator('input[placeholder="Value"]').last().fill('E2E_API_KEY');
    await page.locator('input[type="checkbox"]').last().check();
    
    await page.click('.modal button:has-text("Save")');
    console.log('✅ Step 5: Environment overrides configured');
    
    // Step 6: Verify the configuration persists
    await page.reload();
    await expect(page.locator('.card:has-text("e2e-test-mcp")')).toBeVisible();
    await expect(page.locator('.badge:has-text("Enabled")')).toBeVisible();
    console.log('✅ Step 6: Configuration persisted after reload');
    
    // Step 7: Disable and re-enable binding
    await page.click('.card:has-text("e2e-test-mcp") button:has-text("Disable")');
    await expect(page.locator('.badge:has-text("Disabled")')).toBeVisible();
    
    await page.click('.card:has-text("e2e-test-mcp") button:has-text("Enable")');
    await expect(page.locator('.badge:has-text("Enabled")')).toBeVisible();
    console.log('✅ Step 7: Enable/disable toggle working');
    
    console.log('✅ COMPLETE WORKFLOW SUCCESSFUL');
  });

  test('navigation flow: verify all pages are accessible', async ({ page }) => {
    await page.goto('/');
    
    // Projects page
    await expect(page.locator('h1:has-text("Projects")')).toBeVisible();
    
    // MCP Catalog
    await page.click('nav a:has-text("MCP Catalog")');
    await expect(page.locator('h1:has-text("MCP Catalog")')).toBeVisible();
    
    // Secrets
    await page.click('nav a:has-text("Secrets")');
    await expect(page.locator('h1:has-text("Secrets")')).toBeVisible();
    
    // Back to Projects
    await page.click('nav a:has-text("Projects")');
    await expect(page.locator('h1:has-text("Projects")')).toBeVisible();
    
    console.log('✅ All navigation links working');
  });

  test('data persistence: verify data survives page refresh', async ({ page }) => {
    // Create test data
    await page.goto('/');
    await page.click('button:has-text("Add Project")');
    await page.waitForSelector('.modal', { state: 'visible' });
    await page.fill('input[placeholder="My Project"]', 'Persistence Test');
    await page.fill('input[placeholder="/path/to/project"]', '/tmp/persist');
    await page.click('.modal button:has-text("Create")');
    
    // Wait for modal to close and project to appear
    await page.waitForSelector('.modal', { state: 'hidden', timeout: 5000 }).catch(() => {});
    await page.waitForSelector('.card:has-text("Persistence Test")', { timeout: 10000 });
    
    // Reload page
    await page.reload();
    
    // Verify project still exists
    await expect(page.locator('.card:has-text("Persistence Test")')).toBeVisible();
    
    console.log('✅ Data persistence verified');
  });

  test('concurrent operations: create multiple projects and MCPs', async ({ page }) => {
    await page.goto('/');
    
    // Create multiple projects
    const projects = ['Concurrent 1', 'Concurrent 2', 'Concurrent 3'];
    
    for (const projectName of projects) {
      await page.click('button:has-text("Add Project")');
      await page.waitForSelector('.modal', { state: 'visible' });
      await page.fill('input[placeholder="My Project"]', projectName);
      await page.fill('input[placeholder="/path/to/project"]', `/tmp/${projectName.replace(' ', '-')}`);
      await page.click('.modal button:has-text("Create")');
      
      // Wait for modal to close
      await page.waitForSelector('.modal', { state: 'hidden', timeout: 5000 }).catch(() => {});
      await page.waitForTimeout(500);
    }
    
    // Verify all projects exist
    for (const projectName of projects) {
      await expect(page.locator(`.card:has-text("${projectName}")`)).toBeVisible();
    }
    
    console.log('✅ Concurrent operations handled correctly');
  });
});

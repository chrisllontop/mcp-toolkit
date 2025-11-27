import { test, expect, Locator } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
/**
 * Test suite for Project Management functionality
 */
test.describe('Project Management', () => {
  let projectsHeading: Locator;
  let addProjectButton: Locator;
  let copyConfigButton: Locator;
  let myProjectInput: Locator;
  let projectPathInput: Locator;
  let createButton: Locator;

  test.beforeEach(async ({ page }) => {
    const mockContent = fs.readFileSync(path.resolve(__dirname, 'mocks/tauri-mock.js'), 'utf-8');
    await page.addInitScript(mockContent);
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    projectsHeading = page.getByRole('heading', { name: 'Projects' });
    addProjectButton = page.getByRole('button', { name: 'Add Project' });
    copyConfigButton = page.getByRole('button', { name: 'Copy Config' });
    myProjectInput = page.getByRole('textbox', { name: 'My Project' });
    projectPathInput = page.getByRole('textbox', { name: '/path/to/project' });
    createButton = page.getByRole('button', { name: 'Create' });
  });

  test('should display projects page', async ({ page }) => {
    await expect(projectsHeading).toContainText('Projects');
    await expect(addProjectButton).toBeVisible();
    await expect(copyConfigButton).toBeVisible();
  });

  test('should create a new project', async ({ page }) => {
    // Click Add Project button
    await addProjectButton.click();
    
    // Wait for modal
    await expect(page.locator('.modal h2')).toContainText('Add Project');
    
    // Fill in project details
    await myProjectInput.fill('Test Project');
    await projectPathInput.fill('/tmp/test-project');
    
    // Submit
    await createButton.click();
    
    // Verify project appears in list
    await expect(page.locator('.card h3:has-text("Test Project")')).toBeVisible();
    await expect(page.locator('.card:has-text("Test Project") p')).toContainText('/tmp/test-project');
  });

  test('should cancel project creation', async ({ page }) => {
    await addProjectButton.click();
    await expect(page.locator('.modal')).toBeVisible();
    
    await page.click('.modal button:has-text("Cancel")');
    await expect(page.locator('.modal')).not.toBeVisible();
  });

  test('should navigate to project details', async ({ page }) => {
    // Create a project first
    await addProjectButton.click();
    await myProjectInput.fill('Nav Test Project');
    await projectPathInput.fill('/tmp/nav-test');
    await createButton.click();
    await expect(page.locator('.modal')).not.toBeVisible();
    
    // Click on the project card
    await page.click('.card:has-text("Nav Test Project")');
    
    // Verify we're on the project detail page
    await expect(page.getByRole('heading', { name: 'Project Configuration' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Activate MCP' })).toBeVisible();
    // await expect(page.getByRole('button', { name: 'Config' })).toBeVisible();
    // await expect(page.getByRole('button', { name: 'Disable' })).toBeVisible();
  });

  test('should delete a project', async ({ page }) => {
    // Create a project first
    await addProjectButton.click();
    await myProjectInput.fill('Delete Test Project');
    await projectPathInput.fill('/tmp/delete-test');
    await createButton.click();
    
    // Setup dialog handler
    page.on('dialog', dialog => dialog.accept());
    
    // Delete the project
    await page.click('.card:has-text("Delete Test Project") button.danger:has-text("Delete")');
    
    // Verify project is removed
    await expect(page.locator('.card:has-text("Delete Test Project")')).not.toBeVisible();
  });

  test('should copy MCP config to clipboard', async ({ page }) => {
    // Grant clipboard permissions
    await page.context().grantPermissions(['clipboard-read']);
    
    // Click Copy Config
    await copyConfigButton.click();
    
    // Verify toast notification appears
    // Note: This depends on react-hot-toast showing success message
    await expect(page.locator('.toast, [role="status"]')).toBeVisible({ timeout: 2000 });
  });

  test('should handle empty project list', async ({ page }) => {
    // This test assumes a fresh database
    // The page should still display correctly with no projects
    await expect(projectsHeading).toContainText('Projects');
    await expect(addProjectButton).toBeEnabled();
  });

  test('should validate project creation with empty fields', async ({ page }) => {
    await addProjectButton.click();
    
    // Try to create without filling fields
    await page.click('.modal button:has-text("Create")');
    
    // The form should still be visible (validation should prevent creation)
    // Note: This depends on backend validation
    await expect(page.locator('.modal')).toBeVisible();
  });
});

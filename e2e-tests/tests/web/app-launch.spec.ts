/**
 * Smoke Tests - Application Launch and Basic Functionality
 *
 * These tests verify that the application can start and respond to basic interactions.
 * They should be the first tests run and should always pass before proceeding to
 * more complex tests.
 *
 * Prerequisites:
 * - Web mode: Run with `moon run :client --web` (not yet implemented)
 * - Desktop mode: Not applicable for web tests
 */

import { test, expect } from '@playwright/test';
import { TestIds, waitForAppReady } from '../helpers/test-base';

test.describe('Smoke Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page loads successfully', async ({ page }) => {
    // Should not have any console errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await waitForAppReady(page);

    expect(errors).toHaveLength(0);
  });

  test('app root element exists', async ({ page }) => {
    await waitForAppReady(page);

    const appRoot = page.locator(`[data-testid="${TestIds.APP_ROOT}"]`);
    await expect(appRoot).toBeVisible();
  });

  test('no application errors displayed', async ({ page }) => {
    await waitForAppReady(page);

    const errorElement = page.locator(`[data-testid="${TestIds.APP_ERROR}"]`);
    await expect(errorElement).not.toBeVisible();
  });

  test('navigation bar is visible', async ({ page }) => {
    await waitForAppReady(page);

    const navBar = page.locator(`[data-testid="${TestIds.NAV_BAR}"]`);
    await expect(navBar).toBeVisible();

    // Check for navigation items
    await expect(page.locator(`[data-testid="${TestIds.NAV_HOME}"]`)).toBeVisible();
    await expect(page.locator(`[data-testid="${TestIds.NAV_BEADS}"]`)).toBeVisible();
    await expect(page.locator(`[data-testid="${TestIds.NAV_SETTINGS}"]`)).toBeVisible();
  });

  test('page has valid title', async ({ page }) => {
    await waitForAppReady(page);

    await expect(page).toHaveTitle(/Clarity/i);
  });
});

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
  });

  test('can navigate to beads page', async ({ page }) => {
    await page.click(`[data-testid="${TestIds.NAV_BEADS}"]`);

    await expect(page).toHaveURL(/\/beads/);
  });

  test('can navigate to settings page', async ({ page }) => {
    await page.click(`[data-testid="${TestIds.NAV_SETTINGS}"]`);

    await expect(page).toHaveURL(/\/settings/);
  });

  test('can navigate back to home', async ({ page }) => {
    // Go to beads
    await page.click(`[data-testid="${TestIds.NAV_BEADS}"]`);
    await expect(page).toHaveURL(/\/beads/);

    // Go back home
    await page.click(`[data-testid="${TestIds.NAV_HOME}"]`);
    await expect(page).toHaveURL(/\/^/);
  });
});

test.describe('Responsive Design', () => {
  test('mobile viewport loads correctly', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    await waitForAppReady(page);

    const appRoot = page.locator(`[data-testid="${TestIds.APP_ROOT}"]`);
    await expect(appRoot).toBeVisible();
  });

  test('tablet viewport loads correctly', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');

    await waitForAppReady(page);

    const appRoot = page.locator(`[data-testid="${TestIds.APP_ROOT}"]`);
    await expect(appRoot).toBeVisible();
  });

  test('desktop viewport loads correctly', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');

    await waitForAppReady(page);

    const appRoot = page.locator(`[data-testid="${TestIds.APP_ROOT}"]`);
    await expect(appRoot).toBeVisible();
  });
});

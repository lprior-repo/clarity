/**
 * Beads CRUD E2E Tests
 *
 * Tests the complete lifecycle of bead management:
 * - Create new beads
 * - Read/list beads
 * - Update existing beads
 * - Delete beads
 *
 * These tests validate the user-facing workflows for bead management.
 *
 * Prerequisites:
 * - Web mode: Run with `moon run :client --web`
 * - Database: Must be initialized (handled by app startup)
 */

import { test, expect } from '@playwright/test';
import {
  TestIds,
  waitForAppReady,
  navigateTo,
  createBead,
  getBeadCount,
  BeadStatus,
  BeadPriority,
  BeadType,
} from '../helpers/test-base';

test.describe('Beads CRUD', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await navigateTo(page, 'beads');
  });

  test('displays beads list page', async ({ page }) => {
    const beadsList = page.locator(`[data-testid="${TestIds.BEADS_LIST}"]`);
    await expect(beadsList).toBeVisible();
  });

  test('shows empty state when no beads exist', async ({ page }) => {
    const emptyState = page.locator(`[data-testid="${TestIds.BEADS_EMPTY}"]`);
    await expect(emptyState).toBeVisible();
    await expect(emptyState).toContainText('no beads', { ignoreCase: true });
  });

  test('can create a new bead', async ({ page }) => {
    const initialCount = await getBeadCount(page);

    await createBead(page, {
      title: 'Test Bead from E2E',
      description: 'This bead was created by automated tests',
      status: BeadStatus.OPEN,
      priority: BeadPriority.HIGH,
      type: BeadType.FEATURE,
    });

    // Verify bead was added
    const newCount = await getBeadCount(page);
    expect(newCount).toBe(initialCount + 1);

    // Verify bead appears in list
    await expect(page.locator('text=Test Bead from E2E')).toBeVisible();
  });

  test('can create bead with minimal data', async ({ page }) => {
    const initialCount = await getBeadCount(page);

    await createBead(page, {
      title: 'Minimal Bead',
    });

    const newCount = await getBeadCount(page);
    expect(newCount).toBe(initialCount + 1);

    await expect(page.locator('text=Minimal Bead')).toBeVisible();
  });

  test('can view bead details', async ({ page }) => {
    // Create a bead first
    await createBead(page, {
      title: 'Bead to View',
      description: 'This bead will be viewed',
    });

    // Click on the bead
    await page.click('text=Bead to View');

    // Should navigate to detail view
    const beadDetail = page.locator(`[data-testid="${TestIds.BEAD_DETAIL}"]`);
    await expect(beadDetail).toBeVisible();

    // Verify details
    await expect(page.locator(`[data-testid="${TestIds.BEAD_TITLE}"]`)).toContainText('Bead to View');
    await expect(page.locator(`[data-testid="${TestIds.BEAD_DESCRIPTION}"]`)).toContainText(
      'This bead will be viewed'
    );
  });

  test('can update an existing bead', async ({ page }) => {
    // Create a bead
    await createBead(page, {
      title: 'Original Title',
      description: 'Original description',
      status: BeadStatus.OPEN,
    });

    // Navigate to bead detail
    await page.click('text=Original Title');

    // Click edit button (assumes it exists)
    await page.click('[data-testid="bead-edit-button"]');

    // Wait for form
    await page.waitForSelector(`[data-testid="${TestIds.BEAD_FORM}"]`);

    // Update fields
    await page.fill(`[data-testid="${TestIds.BEAD_FORM_TITLE}"]`, 'Updated Title');
    await page.fill(`[data-testid="${TestIds.BEAD_FORM_DESCRIPTION}"]`, 'Updated description');
    await page.selectOption(
      `[data-testid="${TestIds.BEAD_FORM_STATUS}"]`,
      BeadStatus.IN_PROGRESS
    );

    // Submit
    await page.click(`[data-testid="${TestIds.BEAD_FORM_SUBMIT}"]`);

    // Wait for form to close
    await page.waitForSelector(`[data-testid="${TestIds.BEAD_FORM}"]`, { state: 'hidden' });

    // Verify updates
    await expect(page.locator(`[data-testid="${TestIds.BEAD_TITLE}"]`)).toContainText(
      'Updated Title'
    );
    await expect(page.locator(`[data-testid="${TestIds.BEAD_STATUS}"]`)).toContainText('in_progress');
  });

  test('can delete a bead', async ({ page }) => {
    // Create a bead
    await createBead(page, {
      title: 'Bead to Delete',
    });

    const initialCount = await getBeadCount(page);

    // Navigate to bead detail
    await page.click('text=Bead to Delete');

    // Click delete button (assumes it exists)
    await page.click('[data-testid="bead-delete-button"]');

    // Confirm deletion (modal should appear)
    await page.click(`[data-testid="${TestIds.MODAL_CONFIRM}"]`);

    // Should return to list
    await expect(page.locator(`[data-testid="${TestIds.BEADS_LIST}"]`)).toBeVisible();

    // Verify bead was deleted
    const newCount = await getBeadCount(page);
    expect(newCount).toBe(initialCount - 1);
    await expect(page.locator('text=Bead to Delete')).not.toBeVisible();
  });

  test('can cancel bead creation', async ({ page }) => {
    const initialCount = await getBeadCount(page);

    // Start creating a bead
    await page.click('[data-testid="bead-new-button"]');
    await page.waitForSelector(`[data-testid="${TestIds.BEAD_FORM}"]`);

    // Fill some fields
    await page.fill(`[data-testid="${TestIds.BEAD_FORM_TITLE}"]`, 'Unsaved Bead');

    // Cancel
    await page.click(`[data-testid="${TestIds.BEAD_FORM_CANCEL}"]`);

    // Wait for form to close
    await page.waitForSelector(`[data-testid="${TestIds.BEAD_FORM}"]`, { state: 'hidden' });

    // Verify bead was not created
    const newCount = await getBeadCount(page);
    expect(newCount).toBe(initialCount);
    await expect(page.locator('text=Unsaved Bead')).not.toBeVisible();
  });
});

test.describe('Bead Filtering and Search', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await navigateTo(page, 'beads');

    // Create test beads with different properties
    await createBead(page, { title: 'Open Feature', status: BeadStatus.OPEN, type: BeadType.FEATURE });
    await createBead(page, {
      title: 'Closed Bug',
      status: BeadStatus.CLOSED,
      type: BeadType.BUGFIX,
    });
    await createBead(page, {
      title: 'In Progress Refactor',
      status: BeadStatus.IN_PROGRESS,
      type: BeadType.REFACTOR,
    });
  });

  test('can filter by status', async ({ page }) => {
    // Use status filter (assumes filter UI exists)
    await page.selectOption('[data-testid="filter-status"]', BeadStatus.OPEN);

    // Should only show open beads
    await expect(page.locator('text=Open Feature')).toBeVisible();
    await expect(page.locator('text=Closed Bug')).not.toBeVisible();
    await expect(page.locator('text=In Progress Refactor')).not.toBeVisible();
  });

  test('can filter by type', async ({ page }) => {
    await page.selectOption('[data-testid="filter-type"]', BeadType.BUGFIX);

    await expect(page.locator('text=Open Feature')).not.toBeVisible();
    await expect(page.locator('text=Closed Bug')).toBeVisible();
    await expect(page.locator('text=In Progress Refactor')).not.toBeVisible();
  });

  test('can search beads', async ({ page }) => {
    // Use search box
    await page.fill('[data-testid="search-input"]', 'Feature');

    // Should show matching results
    await expect(page.locator('text=Open Feature')).toBeVisible();
    await expect(page.locator('text=Closed Bug')).not.toBeVisible();
    await expect(page.locator('text=In Progress Refactor')).not.toBeVisible();
  });

  test('can clear filters', async ({ page }) => {
    // Apply filter
    await page.selectOption('[data-testid="filter-status"]', BeadStatus.OPEN);
    await expect(page.locator('text=Open Feature')).toBeVisible();
    await expect(page.locator('text=Closed Bug')).not.toBeVisible();

    // Clear filter
    await page.click('[data-testid="clear-filters"]');

    // All beads should be visible
    await expect(page.locator('text=Open Feature')).toBeVisible();
    await expect(page.locator('text=Closed Bug')).toBeVisible();
    await expect(page.locator('text=In Progress Refactor')).toBeVisible();
  });
});

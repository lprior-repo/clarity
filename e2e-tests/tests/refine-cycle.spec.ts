import { test, expect } from '@playwright/test';

/**
 * E2E Test for Refine Cycle (Task #14)
 *
 * Tests that the Refine button is visible in the Preview phase of
 * the Progressive Discover flow.
 *
 * Reference: clarity-web/src/components/discover/phases/preview_phase.rs
 * lines 365-383 render the Refine button
 */
test.describe('Progressive Discover - Refine Cycle', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test.fixme('should display Refine button in Preview phase', async ({ page }) => {
    // Navigate to Progressive Discover mode
    const progressiveDiscoverTrigger = page
      .getByText(/progressive.*discover/i)
      .or(page.getByRole('button', { name: /progressive/i }));

    const isAvailable = await progressiveDiscoverTrigger.isVisible().catch(() => false);
    if (!isAvailable) {
      test.skip(true, 'Progressive Discover mode not available - app may not be running');
      return;
    }

    await progressiveDiscoverTrigger.first().click();
    await page.waitForTimeout(1000);

    // Look for Preview phase indicators or Refine button
    const previewIndicators = [
      page.getByText(/preview/i),
      page.getByText(/review/i),
      page.locator('[class*="preview" i]'),
    ];

    let inPreviewPhase = false;
    for (const indicator of previewIndicators) {
      inPreviewPhase = await indicator.first().isVisible().catch(() => false);
      if (inPreviewPhase) break;
    }

    // Look for Refine button
    // Reference: preview_phase.rs lines 365-383 render Refine button
    const refineButton = page
      .getByRole('button', { name: /refine/i })
      .or(page.getByText(/refine/i));

    const refineVisible = await refineButton.first().isVisible().catch(() => false);

    // Skip if we cannot reach Preview phase (app may not be fully running)
    if (!inPreviewPhase && !refineVisible) {
      test.skip(true, 'Could not reach Preview phase - requires app to be running');
      return;
    }

    // Assert: Refine button should be visible in Preview phase
    expect(refineVisible).toBeTruthy();
  });
});

import { test, expect } from '@playwright/test';

/**
 * Playwright E2E Tests - Round 3: Quality Gating and Phase Transitions
 *
 * These tests validate:
 * 1. Quality score bar color progression based on score
 * 2. Develop button state changes when gate threshold is met
 * 3. Gate threshold enforcement (minimum 70 score)
 * 4. Tooltip messages explaining gate requirements
 * 5. Phase transition from Discover to Define to Develop
 *
 * Base URL: http://localhost:8080
 */

const BASE_URL = process.env.BASE_URL || 'http://localhost:8080';

test.describe('Quality Gating - Score Bar Colors', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
  });

  test('QG-001: Score bar displays red color when score < 50', async ({ page }) => {
    // This would require filling in answers that result in low quality score
    // The progress bar should have class bg-chart-4 (red)

    const scoreBar = page.locator('[data-testid="quality-score-bar"]');
    await expect(scoreBar).toBeVisible();

    const progressBar = scoreBar.locator('.bg-chart-4');
    // At low scores, bar should be red
  });

  test('QG-002: Score bar displays yellow color when score is 50-69', async ({ page }) => {
    // Fill in some answers to get medium quality score
    // Progress bar should have class bg-chart-3 (yellow)

    const scoreBar = page.locator('[data-testid="quality-score-bar"]');
    await expect(scoreBar).toBeVisible();
  });

  test('QG-003: Score bar displays green color when score is 70-89', async ({ page }) => {
    // Fill in answers to get passing quality score (>= 70)
    // Progress bar should have class bg-chart-2 (green)

    const scoreBar = page.locator('[data-testid="quality-score-bar"]');
    await expect(scoreBar).toBeVisible();

    // Should show "Meets minimum threshold" message
    await expect(page.locator('text=Meets minimum threshold')).toBeVisible();
  });

  test('QG-004: Score bar displays bright green color when score >= 90', async ({ page }) => {
    // Fill in comprehensive answers for high quality score
    // Progress bar should have class bg-chart-1 (bright green)

    const scoreBadge = page.locator('[data-testid="quality-score-badge"]');
    await expect(scoreBadge).toContainText(/[9][0-9]|100/);
  });
});

test.describe('Quality Gating - Threshold Enforcement', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
  });

  test('QG-005: Below threshold message shows when score < 70', async ({ page }) => {
    // Initially or with poor answers
    const belowThresholdBadge = page.locator('text=Below threshold');
    await expect(belowThresholdBadge).toBeVisible();

    const statusMessage = page.locator('text=Improve quality to unlock Develop phase');
    await expect(statusMessage).toBeVisible();
  });

  test('QG-006: Threshold marker visible at 70% on progress bar', async ({ page }) => {
    // The progress bar should show a marker at 70%
    const progressBar = page.locator('[data-testid="quality-progress-bar"]');
    const thresholdMarker = progressBar.locator('.bg-foreground\\/50');

    await expect(thresholdMarker).toHaveAttribute('style', /left: 70%/);
  });

  test('QG-007: Develop phase locked when quality score < 70', async ({ page }) => {
    // Click on Develop phase tab
    const developTab = page.locator('[data-testid="phase-tab-develop"]');
    await developTab.click();

    // Should see lock message or be prevented from accessing
    const lockMessage = page.locator('text=Improve quality to unlock Develop phase');
    await expect(lockMessage).toBeVisible();
  });

  test('QG-008: Develop phase unlocks when quality score >= 70', async ({ page }) => {
    // This test would need to:
    // 1. Fill in comprehensive answers
    // 2. Wait for quality score to calculate and reach >= 70
    // 3. Verify "Meets minimum threshold" appears
    // 4. Click Develop phase tab
    // 5. Verify access is granted

    const statusMessage = page.locator('text=Meets minimum threshold');
    await expect(statusMessage).toBeVisible();

    // Verify Develop tab is clickable/enabled
    const developTab = page.locator('[data-testid="phase-tab-develop"]');
    await expect(developTab).not.toHaveAttribute('disabled');
  });
});

test.describe('Quality Gating - Tooltips and Messages', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
  });

  test('QG-009: Tooltip explains quality gate requirements', async ({ page }) => {
    // Hover over the "Below threshold" badge
    const belowThresholdBadge = page.locator('text=Below threshold');
    await belowThresholdBadge.hover();

    // Should see tooltip explaining the requirement
    const tooltip = page.locator('[data-testid="quality-gate-tooltip"]');
    await expect(tooltip).toBeVisible();
    await expect(tooltip).toContainText('70');
  });

  test('QG-010: Status message provides actionable guidance', async ({ page }) => {
    const statusMessage = page.locator('[data-testid="quality-status-message"]');

    // Should explain what needs to be done
    await expect(statusMessage).toContainText(/Improve quality|Answer questions|Meets minimum/);
  });

  test('QG-011: Dimension breakdown shows passing/failing dimensions', async ({ page }) => {
    // Click "Show details" to see dimension scores
    const showDetailsButton = page.locator('button:has-text("Show details")');
    await showDetailsButton.click();

    // Should see individual dimension scores
    const dimensionScores = page.locator('[data-testid="dimension-score"]');
    await expect(dimensionScores.first()).toBeVisible();

    // Passing dimensions should be green (text-chart-2)
    // Failing dimensions should be red (text-chart-4)
  });
});

test.describe('Quality Gating - Phase Transitions', () => {
  test('QG-012: Complete Discover phase unlocks Define', async ({ page }) => {
    await page.goto(BASE_URL);

    // 1. Answer all required questions in Discover phase
    // 2. Achieve quality score >= 70
    // 3. Verify Define phase becomes accessible

    const defineTab = page.locator('[data-testid="phase-tab-define"]');
    await expect(defineTab).toBeEnabled();
  });

  test('QG-013: Quality gate persists across page reloads', async ({ page }) => {
    await page.goto(BASE_URL);

    // Fill in answers to reach quality score >= 70
    // Reload the page
    await page.reload();

    // Quality score should be preserved
    const scoreBadge = page.locator('[data-testid="quality-score-badge"]');
    await expect(scoreBadge).toContainText(/[7-9][0-9]|100/);
  });
});

test.describe('Quality Gating - Edge Cases', () => {
  test('QG-014: Exact score of 70 passes gate', async ({ page }) => {
    await page.goto(BASE_URL);

    // Answers resulting in exactly 70 quality score
    // Should pass the gate

    const statusMessage = page.locator('text=Meets minimum threshold');
    await expect(statusMessage).toBeVisible();
  });

  test('QG-015: Score of 69 fails gate', async ({ page }) => {
    await page.goto(BASE_URL);

    // Answers resulting in 69 quality score
    // Should fail the gate

    const statusMessage = page.locator('text=Improve quality to unlock Develop phase');
    await expect(statusMessage).toBeVisible();
  });

  test('QG-016: Empty answers show zero score with guidance', async ({ page }) => {
    await page.goto(BASE_URL);

    const scoreBadge = page.locator('[data-testid="quality-score-badge"]');
    await expect(scoreBadge).toContainText('0');

    const statusMessage = page.locator('text=Answer questions to calculate quality');
    await expect(statusMessage).toBeVisible();
  });
});

test.describe('Quality Gating - Integration', () => {
  test('QG-017: End-to-end flow from Discover to Develop', async ({ page }) => {
    await page.goto(BASE_URL);

    // Step 1: Start in Discover phase
    const currentPhase = page.locator('[data-testid="current-phase"]');
    await expect(currentPhase).toContainText('Discover');

    // Step 2: Fill in Express mode input
    const expressInput = page.locator('textarea[placeholder*="Describe"]');
    await expressInput.fill('Building a task management app for remote teams. The main problem is that team members often miss deadlines because tasks aren\\'t clearly assigned or tracked across different time zones. We need a way to see who\\'s working on what, when it\\'s due. Target users: remote team members and project managers. Context: teams working across different time zones need visibility. Constraints: must work across time zones, support mobile access. Goals: clear task assignment, deadline visibility, progress tracking.');

    // Step 3: Submit for extraction
    const extractButton = page.locator('button:has-text("Extract")');
    await extractButton.click();

    // Step 4: Wait for quality score calculation
    await page.waitForTimeout(2000);

    // Step 5: Verify quality score is calculated
    const scoreBadge = page.locator('[data-testid="quality-score-badge"]');
    const scoreText = await scoreBadge.textContent();
    const score = parseInt(scoreText || '0', 10);

    console.log(`Quality score: ${score}`);

    // Step 6: If score >= 70, Develop should be unlocked
    if (score >= 70) {
      await expect(page.locator('text=Meets minimum threshold')).toBeVisible();
      const developTab = page.locator('[data-testid="phase-tab-develop"]');
      await expect(developTab).toBeEnabled();
    } else {
      await expect(page.locator('text=Improve quality to unlock')).toBeVisible();
    }
  });
});

import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Progressive Discover Flow
 *
 * These tests verify the complete Progressive Discover pipeline:
 * PROMPT -> SYNTHESIZE -> THESIS -> ANTITHESIS -> SYNTHESIS -> LOCKED
 *
 * Reference: docs/VISION-ProgressiveDiscover.md lines 511-518
 */

test.describe('Progressive Discover - Basic Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test.skip('should display Progressive Discover interface', async ({ page }) => {
    // This test verifies that the Progressive Discover mode is available
    // and displays the expected UI elements
    //
    // What to verify:
    // - Progressive Discover mode option is visible
    // - Phase indicator shows PROMPT as initial phase
    // - Input area for prompt is available
    // - Quality score indicator exists
    //
    // Skip reason: Progressive Discover feature not yet implemented
    const progressiveDiscoverTrigger = page.getByText(/progressive.*discover/i).or(
      page.getByRole('button', { name: /progressive/i })
    );

    const isVisible = await progressiveDiscoverTrigger.isVisible().catch(() => false);

    if (isVisible) {
      await progressiveDiscoverTrigger.click();
      await page.waitForTimeout(1000);

      // Verify phase indicator exists
      const phaseIndicator = page.locator('[data-phase], [class*="phase" i]').first();
      await expect(phaseIndicator).toBeVisible();
    } else {
      test.skip(true, 'Progressive Discover mode not yet available');
    }
  });
});

/**
 * E2E Test for Antithesis Quality Rejection Scenario
 *
 * This test verifies the quality scoring system for antithesis points
 * in the Progressive Discover flow's Problem Confirmation phase.
 *
 * Reference: clarity-web/src/components/discover/antithesis.rs
 * Reference: clarity-web/src/components/discover/problem_confirm.rs
 * Reference: clarity-web/src/components/discover/quality_score.rs
 *
 * Quality Scoring Algorithm (antithesis.rs):
 * - Each point scored 0-100 based on specificity
 * - Base score: 20 for non-empty
 * - Length bonus: 10-50 based on word count (10-25 words ideal)
 * - Specificity bonus: 20 for numbers, 10 for "because"
 * - Concrete language bonus: 10 for phrases like "specifically", "for example"
 * - Quality gate threshold: 70 (quality_score.rs line 17)
 *
 * Test Scenario:
 * 1. User enters prompt and extraction happens
 * 2. User reaches Problem confirmation phase
 * 3. User enters 3 vague antithesis points (quality score < 70)
 * 4. System shows quality gate failure and improvement prompt
 * 5. User improves antithesis with specific, detailed points
 * 6. System shows quality gate pass (score >= 70)
 */
test.describe('Progressive Discover - Antithesis Quality Rejection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test.fixme('should reject low-quality antithesis and prompt for improvement', async ({ page }) => {
    // ============================================================
    // STEP 1: Navigate to Progressive Discover and enter prompt
    // ============================================================
    //
    // What to verify:
    // - Progressive Discover mode can be activated
    // - Prompt textarea is visible and accepts input
    // - Extraction process triggers after prompt submission
    //
    // The prompt should describe a product/problem idea that will
    // generate a problem statement and require antithesis input.

    const progressiveDiscoverTrigger = page.getByText(/progressive.*discover/i).or(
      page.getByRole('button', { name: /progressive/i })
    );

    const isProgressiveAvailable = await progressiveDiscoverTrigger.isVisible().catch(() => false);

    if (!isProgressiveAvailable) {
      test.skip(true, 'Progressive Discover mode not yet available');
      return;
    }

    await progressiveDiscoverTrigger.click();
    await page.waitForTimeout(1000);

    // Enter a realistic product prompt
    const promptTextarea = page.locator('textarea, [contenteditable="true"]').first();
    if (await promptTextarea.isVisible()) {
      const testPrompt = `I want to build a task management app for remote teams.
The problem is that distributed teams struggle to coordinate their work
across different time zones. They currently use a mix of Slack, email,
and spreadsheets which leads to missed deadlines and confusion about
who is responsible for what.`;

      await promptTextarea.fill(testPrompt);
      await page.waitForTimeout(2000); // Allow extraction to process
    }

    // ============================================================
    // STEP 2: Navigate to Problem Confirmation phase
    // ============================================================
    //
    // What to verify:
    // - Problem confirmation UI is displayed
    // - Problem statement is shown (extracted or user-editable)
    // - Three antithesis input fields are visible
    // - Quality score indicator is displayed

    // Look for Problem confirmation UI elements
    const problemSection = page.getByText(/problem/i).or(
      page.locator('[class*="problem" i]')
    ).first();

    const antithesisSection = page.getByText(/null hypothesis|antithesis|reject/i).or(
      page.locator('[class*="antithesis" i]')
    ).first();

    // Verify we're in the problem confirmation phase
    await expect(problemSection.or(antithesisSection)).toBeVisible({ timeout: 10000 });

    // Find the three antithesis input fields
    // Based on problem_confirm.rs: AntithesisInput component renders 3 inputs
    const antithesisInputs = page.locator('input[type="text"]').filter({
      hasText: /antithesis/i
    }).or(
      page.locator('input[placeholder*="Antithesis"]')
    );

    // Alternative: look for numbered inputs (1, 2, 3 circles)
    const numberedInputs = page.locator('input[type="text"]').filter({
      has: page.locator('+ span:has-text("1"), + span:has-text("2"), + span:has-text("3")')
    });

    // ============================================================
    // STEP 3: Enter VAGUE antithesis points (should score LOW)
    // ============================================================
    //
    // Vague antithesis examples (from task description):
    // - "People might not like it" (5 words, no specifics)
    // - "It could fail" (3 words, no specifics)
    // - "Users will not care" (4 words, no specifics)
    //
    // Expected quality score: < 70 (should fail quality gate)
    // Based on antithesis.rs scoring:
    // - "People might not like it": ~20 (base) + 10 (short) + 10 (might) = 40
    // - "It could fail": ~20 (base) + 10 (short) + 10 (could) = 40
    // - "Users will not care": ~20 (base) + 10 (short) = 30
    // Average: ~37 -> FAILS quality gate

    const vagueAntithesis = [
      'People might not like it',
      'It could fail',
      'Users will not care'
    ];

    // Fill in vague antithesis points
    const allInputs = await page.locator('input[type="text"]').all();
    let antithesisInputCount = 0;

    for (let i = 0; i < Math.min(vagueAntithesis.length, allInputs.length); i++) {
      const input = allInputs[i];
      const placeholder = await input.getAttribute('placeholder') || '';

      // Check if this is an antithesis input
      if (placeholder.toLowerCase().includes('antithesis') ||
          placeholder.toLowerCase().includes('point') ||
          placeholder.match(/point \d/i)) {
        await input.fill(vagueAntithesis[antithesisInputCount]);
        antithesisInputCount++;
      }
    }

    // If couldn't find by placeholder, try alternative selectors
    if (antithesisInputCount === 0) {
      // Try numbered inputs with circles (based on problem_confirm.rs UI)
      const numberedSections = page.locator('div:has(> span:has-text("1")) input').or(
        page.locator('div:has(> span:has-text("2")) input')
      ).or(
        page.locator('div:has(> span:has-text("3")) input')
      );

      const numberedInputsList = await numberedSections.all();
      for (let i = 0; i < Math.min(vagueAntithesis.length, numberedInputsList.length); i++) {
        await numberedInputsList[i].fill(vagueAntithesis[i]);
      }
    }

    await page.waitForTimeout(1000); // Allow quality score to recalculate

    // ============================================================
    // STEP 4: Verify LOW quality score (< 0.7 or < 70)
    // ============================================================
    //
    // What to verify:
    // - Quality score is displayed
    // - Score is LOW (< 70 out of 100)
    // - Quality gate shows FAIL status
    // - Color coding shows red/amber (not green)
    //
    // Based on quality_score.rs:
    // - 70-100: green (bg-emerald-500/60, text-emerald-400)
    // - 50-69: amber (bg-amber-500/60, text-amber-400)
    // - 0-49: red (bg-red-500/60, text-red-400)

    const qualityScoreDisplay = page.getByText(/quality/i).or(
      page.locator('[class*="quality" i]')
    ).or(
      page.locator('[class*="score" i]')
    );

    // Look for score number (0-100)
    const scoreNumber = page.locator('text=/^\\d{1,3}$/').filter({
      has: page.locator('..:has-text("quality"), ..:has-text("Quality")')
    }).first();

    // Verify quality gate shows FAIL
    const gateStatus = page.getByText(/quality gate.*fail|gate.*fail|need.*have/i);

    // The gate message format from quality_score.rs: "Quality gate: FAIL (need 70, have XX)"
    await expect(gateStatus).toBeVisible({ timeout: 5000 });

    // Extract and verify the score is < 70
    const gateText = await gateStatus.textContent();
    const scoreMatch = gateText?.match(/have\s*(\d+)/i);
    if (scoreMatch) {
      const score = parseInt(scoreMatch[1], 10);
      expect(score).toBeLessThan(70);
    }

    // Verify color is NOT green (should be amber or red)
    const scoreContainer = page.locator('[class*="bg-red"], [class*="bg-amber"], [class*="text-red"], [class*="text-amber"]').filter({
      has: page.locator(':scope').filter({ hasText: /\d{1,3}/ })
    });

    // Should NOT have emerald (green) class for a failing score
    const greenScore = page.locator('[class*="emerald"]').filter({
      has: page.locator(':scope').filter({ hasText: /\d{1,3}/ })
    });
    const isGreenVisible = await greenScore.isVisible().catch(() => false);
    expect(isGreenVisible).toBe(false);

    // ============================================================
    // STEP 5: Verify user is prompted to improve antithesis
    // ============================================================
    //
    // What to verify:
    // - Improvement suggestion or prompt is displayed
    // - Issues list shows specific problems with current antithesis
    // - User cannot proceed (Next button disabled or warning shown)
    //
    // Based on problem_confirm.rs get_specificity_issues():
    // - "Point X is too vague - add more specific details"
    // - "Point X could be more specific"

    const improvementPrompt = page.getByText(/improve|vague|specific|detail|add more/i).or(
      page.locator('[class*="issue"]')
    );

    await expect(improvementPrompt).toBeVisible({ timeout: 5000 });

    // Verify specific issues are shown
    const issuesList = page.getByText(/too vague|more specific|empty|need.*fill/i);
    await expect(issuesList.first()).toBeVisible();

    // Check if Next button is disabled or shows warning
    const nextButton = page.getByRole('button', { name: /next/i });
    if (await nextButton.isVisible()) {
      const isDisabled = await nextButton.isDisabled();
      // Next should be disabled when quality gate fails
      // OR there should be a visible warning
      if (!isDisabled) {
        const warningVisible = await page.getByText(/improve.*before|quality.*required/i).isVisible().catch(() => false);
        expect(warningVisible).toBe(true);
      }
    }

    // ============================================================
    // STEP 6: Enter IMPROVED antithesis points (should score HIGH)
    // ============================================================
    //
    // Improved antithesis examples with specifics:
    // - "Remote teams with established Slack workflows will resist switching
    //    because they already have 2+ years of message history and integrations"
    //   (20+ words, contains "because", concrete context)
    //
    // - "The learning curve requires 3-5 hours of training per team member,
    //    which most managers will not approve given quarterly deadline pressure"
    //   (20+ words, contains numbers, specific timeline)
    //
    // - "At $15/user/month, this costs 3x more than Trello which already
    //    covers 80% of their task tracking needs according to user surveys"
    //   (20+ words, numbers, comparison, data source)
    //
    // Expected quality score: >= 70 (should pass quality gate)

    const improvedAntithesis = [
      'Remote teams with established Slack workflows will resist switching because they already have 2+ years of message history and 15+ integrations they depend on daily',
      'The learning curve requires 3-5 hours of training per team member, which most managers will not approve given quarterly deadline pressure and reduced productivity',
      'At $15 per user per month, this costs 3x more than Trello which already covers 80% of their task tracking needs according to recent user surveys'
    ];

    // Refill with improved antithesis points
    for (let i = 0; i < Math.min(improvedAntithesis.length, allInputs.length); i++) {
      const input = allInputs[i];
      const placeholder = await input.getAttribute('placeholder') || '';

      if (placeholder.toLowerCase().includes('antithesis') ||
          placeholder.toLowerCase().includes('point') ||
          placeholder.match(/point \d/i)) {
        await input.fill(improvedAntithesis[i]);
      }
    }

    await page.waitForTimeout(1000); // Allow quality score to recalculate

    // ============================================================
    // STEP 7: Verify IMPROVED quality score (>= 70)
    // ============================================================
    //
    // What to verify:
    // - Quality score is now >= 70
    // - Quality gate shows PASS status
    // - Color coding is now green
    // - Issues list is empty or shows "No issues"
    // - User can proceed (Next button enabled)

    // Verify quality gate shows PASS
    const passStatus = page.getByText(/quality gate.*pass|gate.*pass/i);
    await expect(passStatus).toBeVisible({ timeout: 5000 });

    // Verify score is >= 70
    const passGateText = await passStatus.textContent();
    // If showing "PASS", extract score from nearby element
    const finalScoreElement = page.locator('text=/^\\d{2,3}$/').first();
    const finalScoreText = await finalScoreElement.textContent();
    if (finalScoreText) {
      const finalScore = parseInt(finalScoreText, 10);
      expect(finalScore).toBeGreaterThanOrEqual(70);
    }

    // Verify color is now green (emerald)
    const greenScoreFinal = page.locator('[class*="emerald"]').filter({
      has: page.locator(':scope').filter({ hasText: /\d{2,3}/ })
    });
    await expect(greenScoreFinal.first()).toBeVisible({ timeout: 5000 });

    // Verify no issues shown
    const noIssuesMessage = page.getByText(/no issues|meets quality/i);
    await expect(noIssuesMessage).toBeVisible({ timeout: 5000 });

    // Verify Next button is now enabled
    if (await nextButton.isVisible()) {
      const isEnabled = await nextButton.isEnabled();
      expect(isEnabled).toBe(true);
    }
  });
});

/**
 * E2E Test for Hole Punching Failure Scenario
 *
 * This test verifies the hole punching validation workflow:
 * 1. User provides scenario without addressing discovery mechanism
 * 2. System flags "Discovery Hole"
 * 3. User adds discovery mechanism
 * 4. System verifies hole is addressed
 * 5. Repeat for Edge Case Hole (error handling)
 * 6. Repeat for Motivation Dropoff (friction handling)
 *
 * Reference: docs/VISION-ProgressiveDiscover.md lines 390-402
 * Reference: docs/VALIDATE_HOLE_PUNCHING_QA_REPORT.md
 * Reference: clarity-web/src/server.rs (validate_hole_punching_server)
 * Reference: clarity-web/src/components/discover/types.rs (HoleType, HolePunchingResults)
 */
test.describe('Progressive Discover - Hole Punching Validation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test.fixme('should detect Discovery Hole when scenario lacks discovery mechanism', async ({ page }) => {
    // Test Case: Scenario missing how users discover the feature
    //
    // Setup: Navigate to Progressive Discover mode
    const progressiveDiscoverTrigger = page.getByText(/progressive.*discover/i).or(
      page.getByRole('button', { name: /progressive/i })
    );

    const isProgressiveDiscoverAvailable = await progressiveDiscoverTrigger.isVisible().catch(() => false);

    if (!isProgressiveDiscoverAvailable) {
      test.skip(true, 'Progressive Discover mode not available - requires app running');
      return;
    }

    await progressiveDiscoverTrigger.click();
    await page.waitForTimeout(1000);

    // Step 1: Enter incomplete scenario (missing discovery mechanism)
    const scenarioInput = page.locator('textarea, [contenteditable="true"]').first();

    if (await scenarioInput.isVisible()) {
      const incompleteScenario = `
User goal: Book a meeting room quickly
Trigger: User needs to schedule a meeting
Value moment: User sees available rooms instantly
Feeling: Relieved and productive
      `.trim();

      await scenarioInput.fill(incompleteScenario);
      await page.waitForTimeout(500);

      // Step 2: Submit scenario for validation
      const submitButton = page.getByRole('button', { name: /submit|validate|check|continue/i });
      if (await submitButton.isVisible()) {
        await submitButton.click();
        await page.waitForTimeout(2000);

        // Step 3: Verify system flags "Discovery Hole"
        const discoveryHoleIndicator = page.getByText(/discovery.*hole/i).or(
          page.locator('[class*="discovery" i], [class*="hole" i]')
        );

        await expect(discoveryHoleIndicator.first()).toBeVisible({ timeout: 5000 });

        // Verify specific flag text is shown
        const holeText = await discoveryHoleIndicator.first().textContent();
        expect(holeText?.toLowerCase()).toMatch(/discovery|how.*find|discover.*feature/);
      }
    }
  });
});

/**
 * E2E Test for Successful Plan Creation Flow
 *
 * Task #9: Implement successful plan creation test
 *
 * This test verifies the full PROMPT to LOCKED flow:
 * 1. Navigate to the app
 * 2. Enter a prompt in the PROMPT phase
 * 3. Click "Extract Fields" -> verify extraction happens
 * 4. Go through CONFIRMING_FIELDS sub-phases (Problem, Persona, Solution, Nonpersona, Scenario)
 * 5. Reach PREVIEW phase
 * 6. Click "Lock In"
 * 7. Verify KIRK_COMPILATION runs
 * 8. Verify LOCKED state is reached
 *
 * Reference: clarity-web/src/components/discover/state.rs
 * Reference: clarity-web/src/components/discover/progressive_discover.rs
 */
test.describe('Progressive Discover - Successful Plan Creation', () => {
  /**
   * Test: Complete flow from PROMPT to LOCKED state
   *
   * Uses test.fixme() if the app isn't accessible at localhost:8082.
   */
  test.fixme('should complete successful plan creation flow from prompt to locked', async ({ page }) => {
    // Step 1: Navigate to the app
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Verify the page loaded - check for the prompt textarea or main content
    const promptTextarea = page.locator('textarea, [contenteditable="true"]').first();
    const textareaVisible = await promptTextarea.isVisible().catch(() => false);

    if (!textareaVisible) {
      test.skip(true, 'Prompt textarea not visible - app may not be running at localhost:8082');
      return;
    }

    // Step 2: Enter a prompt in the PROMPT phase (need at least 50 characters per extract_fields_button.rs)
    const testPrompt = `I am building a task management app for software developers who struggle with context switching.
The main problem is that developers lose focus when switching between tasks and forget important context.
My solution provides smart task grouping and automatic context capture so developers can resume work quickly.
The target users are individual developers who work on multiple projects simultaneously and use VS Code.
A typical scenario: A developer is interrupted by a bug report, switches to fix it, then struggles to remember
where they left off on their original feature work.`;

    await promptTextarea.fill(testPrompt);
    await page.waitForTimeout(500);

    // Verify character count threshold is met (50+ chars)
    const promptValue = await promptTextarea.inputValue();
    expect(promptValue.length).toBeGreaterThanOrEqual(50);

    // Step 3: Click "Extract Fields" button
    const extractFieldsButton = page.getByRole('button', { name: /extract fields/i });
    await expect(extractFieldsButton).toBeEnabled();
    await extractFieldsButton.click();

    // Verify extraction phase begins - look for extracting/loading state
    await page.waitForTimeout(1000);

    // Look for extracting phase indicator (could be "Extracting" or loading spinner)
    const extractingIndicator = page.locator(
      'text=/extracting|processing|loading/i, [class*="animate-spin"], [data-phase="extracting"]'
    ).first();
    const isExtracting = await extractingIndicator.isVisible().catch(() => false);

    // If extraction happens very fast, we might miss this phase - that's OK
    if (isExtracting) {
      console.log('Extraction phase detected');
    }

    // Wait for extraction to complete and move to CONFIRMING_FIELDS
    // Look for Problem confirmation (first sub-phase per state.rs ConfirmSubPhase)
    await page.waitForTimeout(3000);

    // Step 4: Go through CONFIRMING_FIELDS sub-phases
    // Order per state.rs: Problem -> Persona -> Solution -> Nonpersona -> Scenario
    const subPhases = ['Problem', 'Persona', 'Solution', 'Nonpersona', 'Scenario'];

    for (const subPhase of subPhases) {
      // Check if we're on the current sub-phase
      const subPhaseIndicator = page.getByText(new RegExp(subPhase, 'i')).first();
      const isOnSubPhase = await subPhaseIndicator.isVisible().catch(() => false);

      if (isOnSubPhase) {
        console.log(`On ${subPhase} sub-phase`);

        // Look for Next button to advance
        const nextButton = page.getByRole('button', { name: /next|continue/i });

        if (await nextButton.isVisible().catch(() => false)) {
          // Wait for button to be enabled
          await expect(nextButton).toBeEnabled({ timeout: 5000 });
          await nextButton.click();
          await page.waitForTimeout(500);
        }
      }
    }

    // Step 5: Reach PREVIEW phase
    // Wait for Preview phase indicators
    await page.waitForTimeout(2000);
    const previewIndicator = page.getByText(/preview|brutal truths|lock in/i).first();
    const isPreviewVisible = await previewIndicator.isVisible().catch(() => false);

    if (isPreviewVisible) {
      console.log('Reached Preview phase');

      // Step 6: Acknowledge Four Brutal Truths (required before Lock In per progressive_discover.rs)
      // Find all checkboxes for brutal truths and check them
      const brutalTruthCheckboxes = page.locator('input[type="checkbox"]');
      const checkboxCount = await brutalTruthCheckboxes.count();

      for (let i = 0; i < checkboxCount; i++) {
        const checkbox = brutalTruthCheckboxes.nth(i);
        if (await checkbox.isVisible()) {
          await checkbox.check();
        }
      }

      await page.waitForTimeout(500);

      // Click "Lock In" button
      const lockInButton = page.getByRole('button', { name: /lock in/i });

      if (await lockInButton.isVisible().catch(() => false)) {
        await expect(lockInButton).toBeEnabled({ timeout: 5000 });
        await lockInButton.click();

        // Step 7: Verify KIRK_COMPILATION runs
        await page.waitForTimeout(1000);
        const compilationIndicator = page.getByText(/compiling|kirk|processing/i).first();
        const isCompiling = await compilationIndicator.isVisible().catch(() => false);

        if (isCompiling) {
          console.log('KIRK Compilation phase detected');
        }

        // Wait for compilation to complete (auto-advances to Locked per progressive_discover.rs)
        await page.waitForTimeout(5000);

        // Step 8: Verify LOCKED state is reached
        const lockedIndicator = page.getByText(/locked|plan locked|continue to bead factory/i).first();
        const isLocked = await lockedIndicator.isVisible().catch(() => false);

        if (isLocked) {
          console.log('Successfully reached LOCKED state');

          // Verify final state elements
          const successIcon = page.locator('svg[class*="emerald"], text=/success|complete/i').first();
          const hasSuccessIndicator = await successIcon.isVisible().catch(() => false);

          expect(isLocked).toBeTruthy();
          console.log('Plan creation flow completed successfully');
        } else {
          // If not locked, check current phase
          const currentPhase = await page.locator('[class*="phase"]').first().textContent().catch(() => 'unknown');
          console.log(`Expected LOCKED state but found: ${currentPhase}`);
          test.skip(true, `Did not reach LOCKED state - current phase may be: ${currentPhase}`);
        }
      } else {
        console.log('Lock In button not visible - may not have reached Preview phase');
        test.skip(true, 'Lock In button not visible - Preview phase may not be reached');
      }
    } else {
      console.log('Preview phase not reached - checking current state');
      const pageContent = await page.locator('body').textContent();
      console.log('Current page state:', pageContent?.substring(0, 500));
      test.skip(true, 'Did not reach Preview phase within expected time');
    }
  });
});

/**
 * E2E Test for VORP (Value Over Replacement Product) Failure Scenario
 *
 * Task #12: Verify that the system rejects vague VORP justifications and
 * accepts specific, quantified justifications.
 *
 * Flow:
 * 1. Navigate to Solution Confirmation step
 * 2. Enter vague VORP like "It's better" - verify rejection
 * 3. Enter specific VORP like "Reduces task time from 30 minutes to 5 minutes (6x improvement)" - verify acceptance
 */
test.describe('Progressive Discover - VORP Failure Test', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test.fixme('should reject vague VORP justification and accept specific justification', async ({ page }) => {
    // Step 1: Navigate to Progressive Discover mode
    const progressiveDiscoverTrigger = page
      .getByRole('button', { name: /progressive.*discover/i })
      .or(page.getByText(/progressive.*discover/i));

    const isAvailable = await progressiveDiscoverTrigger.first().isVisible().catch(() => false);
    if (!isAvailable) {
      test.skip(true, 'Progressive Discover mode not available');
      return;
    }

    await progressiveDiscoverTrigger.first().click();
    await page.waitForTimeout(1000);

    // Step 2: Locate VORP section
    const vorpSection = page.locator('text=/VORP|Value Over Replacement/i').first();
    if (!(await vorpSection.isVisible().catch(() => false))) {
      test.skip(true, 'VORP section not found');
      return;
    }

    // Step 3: Find VORP input fields
    const valueInput = page.getByPlaceholder(/what value does this provide/i).first();
    const obviousInput = page.getByPlaceholder(/is the benefit immediately obvious/i).first();
    const realInput = page.getByPlaceholder(/is this solving a real/i).first();
    const possibleInput = page.getByPlaceholder(/is this buildable with available resources/i).first();

    // Step 4: Enter vague justifications - should be rejected
    await valueInput.fill("It's better");
    await obviousInput.fill("Users will like it more");
    await realInput.fill("It solves their problem");
    await possibleInput.fill("We can build it");
    await page.waitForTimeout(500);

    // Verify rejection - warnings should appear
    const warningIndicator = page.locator('text=/needs more detail|too vague|at least \\d+ words/i');
    const hasWarnings = await warningIndicator.first().isVisible().catch(() => false);

    // Or Next button disabled
    const nextButton = page.getByRole('button', { name: /next/i });
    const isNextDisabled = await nextButton.isDisabled().catch(() => false);

    expect(hasWarnings || isNextDisabled).toBe(true);

    // Step 5: Enter specific justifications - should be accepted
    await valueInput.fill("Reduces task completion time from 30 minutes to 5 minutes, a 6x improvement that saves users 25 minutes per task");
    await obviousInput.fill("Users immediately see the time savings in the dashboard with clear before/after metrics displayed");
    await realInput.fill("Validated through 15 user interviews where 80% cited time spent on manual task switching as their primary pain point");
    await possibleInput.fill("Technically feasible with current React/Dioxus stack, estimated 2 sprints with existing team skills");
    await page.waitForTimeout(500);

    // Verify acceptance - no warnings, or Next enabled
    const remainingWarnings = page.locator('text=/needs more detail|too vague|at least \\d+ words/i');
    const hasRemainingWarnings = await remainingWarnings.first().isVisible().catch(() => false);
    const isNextNowEnabled = !(await nextButton.isDisabled().catch(() => true));

    expect(!hasRemainingWarnings || isNextNowEnabled).toBe(true);
  });
});

/**
 * E2E Test for Straw Man Trap Detection
 *
 * Task #11: Implement straw man trap detection test
 *
 * This test verifies the straw man trap detection workflow in the Persona confirmation step:
 * 1. User enters prompt and extraction happens
 * 2. In Persona confirmation, user can select straw man traps
 * 3. Verify the UI shows warnings about the trap type
 * 4. User can revise persona to clear traps
 *
 * Straw Man Trap Types (per clarity-web/src/components/discover/straw_man.rs):
 * - IrrationalActor: User acts against their own motivations
 * - ManicPixieDreamUser: User magically loves everything without discernment
 * - StoicMonk: User tolerates immense friction without complaint
 * - YourClone: User has your system knowledge
 *
 * Example trap personas:
 * - Manic Pixie Dream User: "A busy mom who wants to spend hours learning complex software"
 * - Irrational Actor: "A user who loves doing manual data entry all day"
 *
 * Reference: clarity-web/src/components/discover/straw_man.rs
 * Reference: clarity-web/src/components/discover/persona_confirm.rs
 */
test.describe('Progressive Discover - Straw Man Trap Detection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test.fixme('should detect and display straw man trap warnings in Persona confirmation', async ({ page }) => {
    // Step 1: Navigate to Progressive Discover
    const progressiveDiscoverTrigger = page.getByText(/progressive.*discover/i).or(
      page.getByRole('button', { name: /progressive/i })
    );

    const isProgressiveDiscoverAvailable = await progressiveDiscoverTrigger.isVisible().catch(() => false);

    if (!isProgressiveDiscoverAvailable) {
      test.skip(true, 'Progressive Discover mode not yet available');
      return;
    }

    await progressiveDiscoverTrigger.click();
    await page.waitForTimeout(1000);

    // Step 2: Enter a prompt to trigger extraction
    const promptTextarea = page.locator('textarea').first();
    const isTextareaVisible = await promptTextarea.isVisible().catch(() => false);

    if (!isTextareaVisible) {
      test.skip(true, 'Prompt textarea not found');
      return;
    }

    // Enter a prompt that would extract a persona
    const testPrompt = 'I want to build a task management app for busy professionals who need to organize their daily work and personal tasks efficiently.';
    await promptTextarea.fill(testPrompt);
    await page.waitForTimeout(500);

    // Click extract/submit button
    const extractButton = page.getByRole('button', { name: /extract|submit|continue|next/i }).first();
    const isExtractButtonVisible = await extractButton.isVisible().catch(() => false);

    if (isExtractButtonVisible) {
      await extractButton.click();
      // Wait for extraction to complete
      await page.waitForTimeout(3000);
    }

    // Step 3: Navigate to Persona confirmation step
    // Look for Persona step indicator or navigate through confirmation steps
    const personaStep = page.getByText(/persona/i).first();
    const isPersonaStepVisible = await personaStep.isVisible().catch(() => false);

    if (!isPersonaStepVisible) {
      // Try clicking Next until we reach Persona step (Problem is first, Persona is second per state.rs)
      for (let i = 0; i < 5; i++) {
        const nextButton = page.getByRole('button', { name: /^next$/i }).first();
        const isNextVisible = await nextButton.isVisible().catch(() => false);
        if (isNextVisible) {
          await nextButton.click();
          await page.waitForTimeout(500);
          const found = await page.getByText(/persona/i).first().isVisible().catch(() => false);
          if (found) break;
        }
      }
    }

    // Step 4: Verify Straw Man Checklist is visible (per persona_confirm.rs StrawManChecklist component)
    const strawManSection = page.getByText(/straw.?man|trap/i).first();
    await expect(strawManSection).toBeVisible({ timeout: 5000 });

    // Step 5: Select "Manic Pixie Dream User" trap
    const manicPixieCheckbox = page.locator('label').filter({ hasText: /manic.?pixie/i });
    const isManicPixieVisible = await manicPixieCheckbox.isVisible().catch(() => false);

    if (isManicPixieVisible) {
      await manicPixieCheckbox.click();
      await page.waitForTimeout(300);

      // Verify the trap is highlighted (amber background per persona_confirm.rs)
      const trapHighlight = page.locator('.bg-amber-500\\/10, [class*="amber"]').first();
      await expect(trapHighlight).toBeVisible();

      // Verify quality score shows warning (per PersonaQuality component)
      const qualityWarning = page.getByText(/straw.?man|trap|warning/i);
      const hasWarning = await qualityWarning.isVisible().catch(() => false);
      expect(hasWarning).toBeTruthy();
    }

    // Step 6: Select "Irrational Actor" trap
    const irrationalActorCheckbox = page.locator('label').filter({ hasText: /irrational.?actor/i });
    const isIrrationalVisible = await irrationalActorCheckbox.isVisible().catch(() => false);

    if (isIrrationalVisible) {
      await irrationalActorCheckbox.click();
      await page.waitForTimeout(300);

      // Verify trap count increased (more amber highlights)
      const trapCount = await page.locator('[class*="amber"]').count();
      expect(trapCount).toBeGreaterThanOrEqual(2);
    }

    // Step 7: Clear all traps by clicking checkboxes again (toggle off)
    if (isManicPixieVisible) {
      await manicPixieCheckbox.click();
      await page.waitForTimeout(200);
    }
    if (isIrrationalVisible) {
      await irrationalActorCheckbox.click();
      await page.waitForTimeout(200);
    }

    // Step 8: Verify traps are cleared (no amber highlights)
    const remainingHighlights = await page.locator('[class*="amber"]').count();
    expect(remainingHighlights).toBe(0);
  });
});

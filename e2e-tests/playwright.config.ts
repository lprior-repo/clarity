import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E Test Configuration for Clarity Desktop Application
 *
 * This configuration supports two testing modes:
 *
 * 1. WEB MODE (Recommended for CI/CD):
 *    - Run clarity-client with --web flag
 *    - Tests run against http://localhost:8080
 *    - Fully compatible with Docker
 *
 * 2. DESKTOP MODE (Host-only):
 *    - Requires Xvfb (virtual framebuffer) for headless testing
 *    - Tests run via Electron/Chromium attach
 *    - Not compatible with Docker
 *
 * Usage:
 *   npm run test                    # Run all tests
 *   npm run test:headed             # Run with visible browser
 *   npm run test:ui                 # Run with Playwright UI
 *   npm run docker:test             # Run in Docker container
 */
export default defineConfig({
  testDir: './tests',
  testMatch: [
    '**/*.spec.ts',
    '**/*.test.ts'
  ],
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list']
  ],
  use: {
    // Base URL for web mode tests
    baseURL: process.env.BASE_URL || 'http://localhost:8080',

    // Collect trace when retrying the failed test
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'retain-on-failure',

    // Action timeout
    actionTimeout: 10 * 1000,
    navigationTimeout: 30 * 1000,
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    // Desktop mode (requires --web=false and Xvfb)
    {
      name: 'desktop-electron',
      testMatch: '**/desktop/**/*.spec.ts',
      use: {
        // Electron-specific launch options
        channel: 'electron',
        // Electron path will be set by tests
      },
    },

    // Web mode (requires --web=true)
    {
      name: 'web-mode',
      testMatch: '**/web/**/*.spec.ts',
      use: {
        // Standard Chromium for web mode
        channel: 'chromium',
      },
    },
  ],

  // Run local server before starting tests (for web mode)
  webServer: {
    command: 'npm run start:web',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});

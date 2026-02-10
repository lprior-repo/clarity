/**
 * Test helper utilities for Clarity E2E tests
 *
 * Provides common selectors, fixtures, and utilities for testing
 * the Clarity desktop application.
 */

/**
 * Test IDs for key Clarity UI components
 *
 * These IDs must be added to the Dioxus components via the `data-testid` attribute.
 * Example: rsx! { div { "data-testid": "app-root", ... } }
 */
export const TestIds = {
  // App structure
  APP_ROOT: 'app-root',
  APP_LOADING: 'app-loading',
  APP_ERROR: 'app-error',

  // Navigation
  NAV_BAR: 'nav-bar',
  NAV_HOME: 'nav-home',
  NAV_BEADS: 'nav-beads',
  NAV_SETTINGS: 'nav-settings',

  // Beads list
  BEADS_LIST: 'beads-list',
  BEADS_LIST_ITEM: (id: string) => `bead-item-${id}`,
  BEADS_LOADING: 'beads-loading',
  BEADS_EMPTY: 'beads-empty',

  // Bead details
  BEAD_DETAIL: 'bead-detail',
  BEAD_TITLE: 'bead-title',
  BEAD_DESCRIPTION: 'bead-description',
  BEAD_STATUS: 'bead-status',
  BEAD_PRIORITY: 'bead-priority',
  BEAD_TYPE: 'bead-type',

  // Forms
  BEAD_FORM: 'bead-form',
  BEAD_FORM_TITLE: 'bead-form-title',
  BEAD_FORM_DESCRIPTION: 'bead-form-description',
  BEAD_FORM_STATUS: 'bead-form-status',
  BEAD_FORM_PRIORITY: 'bead-form-priority',
  BEAD_FORM_TYPE: 'bead-form-type',
  BEAD_FORM_SUBMIT: 'bead-form-submit',
  BEAD_FORM_CANCEL: 'bead-form-cancel',

  // Auth
  AUTH_LOGIN: 'auth-login',
  AUTH_LOGIN_EMAIL: 'auth-login-email',
  AUTH_LOGIN_PASSWORD: 'auth-login-password',
  AUTH_LOGIN_SUBMIT: 'auth-login-submit',
  AUTH_LOGOUT: 'auth-logout',

  // Settings
  SETTINGS_FORM: 'settings-form',
  SETTINGS_THEME: 'settings-theme',
  SETTINGS_BACKUP: 'settings-backup',
  SETTINGS_RESTORE: 'settings-restore',

  // Toasts/Notifications
  TOAST_CONTAINER: 'toast-container',
  TOAST_MESSAGE: 'toast-message',
  TOAST_ERROR: 'toast-error',
  TOAST_SUCCESS: 'toast-success',

  // Modals
  MODAL_CONTAINER: 'modal-container',
  MODAL_CLOSE: 'modal-close',
  MODAL_CONFIRM: 'modal-confirm',
} as const;

/**
 * Bead status values for assertions
 */
export const BeadStatus = {
  OPEN: 'open',
  IN_PROGRESS: 'in_progress',
  CLOSED: 'closed',
} as const;

/**
 * Bead priority values for assertions
 */
export const BeadPriority = {
  LOW: '1',
  MEDIUM: '2',
  HIGH: '3',
} as const;

/**
 * Bead type values for assertions
 */
export const BeadType = {
  FEATURE: 'feature',
  BUGFIX: 'bugfix',
  REFACTOR: 'refactor',
  DOCS: 'docs',
  TEST: 'test',
} as const;

/**
 * Wait for app to be ready (not loading, no errors)
 */
export async function waitForAppReady(page: import('@playwright/test').Page) {
  // Wait for loading state to disappear
  await page.waitForSelector(`[data-testid="${TestIds.APP_LOADING}"]`, {
    state: 'hidden',
    timeout: 10000,
  }).catch(() => {
    // Loading element might not exist, that's ok
  });

  // Check for error state
  const errorElement = await page.$(`[data-testid="${TestIds.APP_ERROR}"]`);
  if (errorElement) {
    const errorText = await errorElement.textContent();
    throw new Error(`App in error state: ${errorText}`);
  }
}

/**
 * Navigate to a specific route
 */
export async function navigateTo(
  page: import('@playwright/test').Page,
  route: 'home' | 'beads' | 'settings'
) {
  const navButton = page.locator(`[data-testid="${TestIds[`NAV_${route.toUpperCase()}`]}"]`);
  await navButton.click();
  await page.waitForURL(`**/${route === 'home' ? '' : route}`);
}

/**
 * Create a new bead via the UI
 */
export async function createBead(
  page: import('@playwright/test').Page,
  bead: {
    title: string;
    description?: string;
    status?: string;
    priority?: string;
    type?: string;
  }
) {
  // Click "New Bead" button (assumes one exists)
  await page.click('[data-testid="bead-new-button"]');

  // Wait for form to appear
  await page.waitForSelector(`[data-testid="${TestIds.BEAD_FORM}"]`);

  // Fill form
  await page.fill(`[data-testid="${TestIds.BEAD_FORM_TITLE}"]`, bead.title);

  if (bead.description) {
    await page.fill(`[data-testid="${TestIds.BEAD_FORM_DESCRIPTION}"]`, bead.description);
  }

  if (bead.status) {
    await page.selectOption(`[data-testid="${TestIds.BEAD_FORM_STATUS}"]`, bead.status);
  }

  if (bead.priority) {
    await page.selectOption(`[data-testid="${TestIds.BEAD_FORM_PRIORITY}"]`, bead.priority);
  }

  if (bead.type) {
    await page.selectOption(`[data-testid="${TestIds.BEAD_FORM_TYPE}"]`, bead.type);
  }

  // Submit form
  await page.click(`[data-testid="${TestIds.BEAD_FORM_SUBMIT}"]`);

  // Wait for form to close
  await page.waitForSelector(`[data-testid="${TestIds.BEAD_FORM}"]`, {
    state: 'hidden',
  });
}

/**
 * Get bead count from list
 */
export async function getBeadCount(
  page: import('@playwright/test').Page
): Promise<number> {
  const list = page.locator(`[data-testid="${TestIds.BEADS_LIST}"]`);
  const items = list.locator('[data-testid^="bead-item-"]');
  return await items.count();
}

/**
 * Login with test credentials
 */
export async function login(
  page: import('@playwright/test').Page,
  email: string,
  password: string
) {
  await page.fill(`[data-testid="${TestIds.AUTH_LOGIN_EMAIL}"]`, email);
  await page.fill(`[data-testid="${TestIds.AUTH_LOGIN_PASSWORD}"]`, password);
  await page.click(`[data-testid="${TestIds.AUTH_LOGIN_SUBMIT}"]`);

  // Wait for navigation to home
  await page.waitForURL('**/');
}

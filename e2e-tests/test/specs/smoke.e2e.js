/**
 * E2E Smoke Test for Colimail
 *
 * Tests the critical path: Launch app → Verify UI loads → Check core components
 *
 * Note: This is a minimal smoke test to ensure the app can start and render
 * without errors. Full functional tests (account loading, email display) require
 * pre-configured test accounts and mock IMAP servers, which are out of scope for P2.
 */

describe('Colimail - Smoke Test', () => {
  it('should successfully launch the application', async () => {
    // Wait for the app window to be ready
    await browser.pause(2000);

    // Verify the window has dimensions (confirms window is open)
    // Note: getTitle() returns empty string for Tauri apps, so we check window size instead
    const windowSize = await browser.getWindowSize();
    expect(windowSize.width).toBeGreaterThan(0);
    expect(windowSize.height).toBeGreaterThan(0);
    console.log('✓ Application launched successfully');
  });

  it('should render the main application UI', async () => {
    // Check if body element exists
    const body = await $('body');
    const bodyExists = await body.isExisting();
    expect(bodyExists).toBe(true);
    console.log('✓ Main UI body element is present');
  });

  it('should display core UI components without white screen', async () => {
    // Wait for React/Svelte to render
    await browser.pause(1000);

    // Check if any interactive elements are present (buttons, inputs, etc.)
    const interactiveElements = await $$('button, input, a, [role="button"]');
    expect(interactiveElements.length).toBeGreaterThan(0);
    console.log(
      `✓ Found ${interactiveElements.length} interactive elements - UI has rendered`
    );
  });

  it('should not display fatal error messages', async () => {
    // Wait for potential errors to appear
    await browser.pause(500);

    // Check for common error indicators
    const bodyText = await $('body').getText();

    // These are common fatal error messages that shouldn't appear on startup
    const fatalErrors = [
      'Application Error',
      'Failed to load',
      'Cannot read properties of undefined',
      'Uncaught Error',
      'Fatal Error',
    ];

    for (const errorText of fatalErrors) {
      expect(bodyText).not.toContain(errorText);
    }

    console.log('✓ No fatal error messages detected');
  });

  it('should have responsive UI (basic smoke check)', async () => {
    // Get window size to verify app is not minimized/hidden
    const windowSize = await browser.getWindowSize();
    expect(windowSize.width).toBeGreaterThan(0);
    expect(windowSize.height).toBeGreaterThan(0);
    console.log(`✓ Window dimensions: ${windowSize.width}x${windowSize.height}`);
  });
});

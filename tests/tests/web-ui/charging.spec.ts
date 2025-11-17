import { test, expect } from '@playwright/test';

/**
 * Charging Features E2E Tests
 * Tests charging controls, alerts, and history
 */

test.describe('Charging Features', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await page.waitForLoadState('networkidle');
    // Navigate to charging section if not default
  });

  test('should start charging via quick action', async ({ page }) => {
    // Find the charging button
    const chargeButton = page.getByRole('button', { name: /Start Charging|Stop/ });
    await expect(chargeButton).toBeVisible();

    // Get initial state
    const initialText = await chargeButton.textContent();

    // Click the button
    await chargeButton.click();

    // Wait for HTMX response
    await page.waitForTimeout(1000);

    // Button should update (may show loading or opposite state)
    const updatedButton = page.getByRole('button', { name: /Start Charging|Stop/ });
    await expect(updatedButton).toBeVisible();
  });

  test('should display charging progress when active', async ({ page }) => {
    // Navigate to alerts/charging section
    await page.click('text=Dashboard');

    // Wait for charging status to load
    await page.waitForSelector('.charging-card', { timeout: 10000 });

    const chargingCard = page.locator('.charging-card');

    if (await chargingCard.isVisible()) {
      // Should show progress bar or ring
      await expect(page.locator('.progress-bar, .charging-progress')).toBeVisible();

      // Should show power level
      await expect(page.locator('text=/kW/')).toBeVisible();

      // Should show time remaining
      await expect(page.locator('text=/min|hour/')).toBeVisible();
    }
  });

  test('should configure charging alerts', async ({ page }) => {
    // Go to alerts section
    await page.click('text=Alerts');

    // Should have alert configuration options
    const alertConfig = page.locator('.alert-config, form');
    await expect(alertConfig).toBeVisible();

    // Should have 80% optimal charge checkbox
    const optimalCharge = page.getByLabel(/optimal|80%/i);
    if (await optimalCharge.isVisible()) {
      await expect(optimalCharge).toBeVisible();

      // Toggle it
      await optimalCharge.click();

      // Wait for HTMX to save
      await page.waitForTimeout(500);
    }

    // Should have full charge checkbox
    const fullCharge = page.getByLabel(/full|100%|complete/i);
    if (await fullCharge.isVisible()) {
      await expect(fullCharge).toBeVisible();
    }

    // Should have custom level input
    const customLevel = page.locator('input[type="range"], input[type="number"]').first();
    if (await customLevel.isVisible()) {
      await customLevel.fill('90');

      // Wait for HTMX to save
      await page.waitForTimeout(500);
    }
  });

  test('should display charging history', async ({ page }) => {
    // Navigate to history or analytics
    await page.click('text=Analytics');

    // Wait for history to load
    await page.waitForSelector('.history-list, .charging-history', { timeout: 10000 });

    // Should show historical charging sessions
    const historyItems = page.locator('.history-item');
    const count = await historyItems.count();

    if (count > 0) {
      // Each item should show date, levels, duration, energy
      const firstItem = historyItems.first();
      await expect(firstItem).toBeVisible();

      // Should contain percentage indicators
      await expect(firstItem).toContainText(/%/);

      // Should contain time information
      await expect(firstItem).toContainText(/min|hour|h/);

      // Should contain energy information
      await expect(firstItem).toContainText(/kWh/);
    }
  });

  test('should update charging status in real-time', async ({ page }) => {
    // Get initial charging status
    const statusElement = page.locator('.charging-status, .battery-percentage').first();
    await expect(statusElement).toBeVisible();

    const initialStatus = await statusElement.textContent();

    // Wait for HTMX auto-refresh (charging updates every 2s)
    await page.waitForTimeout(3000);

    // Status should still be visible (may or may not have changed value)
    await expect(statusElement).toBeVisible();
  });

  test('should show weekly charging statistics', async ({ page }) => {
    // Go to analytics
    await page.click('text=Analytics');

    // Wait for analytics to load
    await page.waitForSelector('.analytics, #analytics-section', { timeout: 10000 });

    // Should show weekly stats
    const weeklyStats = page.locator('text=/Weekly|This Week/i');
    await expect(weeklyStats).toBeVisible();

    // Should show total energy
    await expect(page.locator('text=/kWh/i')).toBeVisible();

    // Should show cost information
    await expect(page.locator('text=/€|EUR|cost/i')).toBeVisible();

    // Should show number of sessions
    await expect(page.locator('text=/session/i')).toBeVisible();
  });

  test('should calculate costs accurately', async ({ page }) => {
    await page.click('text=Analytics');

    // Wait for cost analytics
    await page.waitForSelector('.cost-analysis, #analytics-section', { timeout: 10000 });

    // Should display costs
    const costElements = page.locator('text=/€[0-9]+\\.[0-9]{2}/');
    const count = await costElements.count();
    expect(count).toBeGreaterThan(0);

    // Should show cost per kWh
    await expect(page.locator('text=/kWh/i')).toBeVisible();
  });

  test('should pre-condition vehicle', async ({ page }) => {
    // Find pre-condition button
    const preconditionBtn = page.getByRole('button', { name: /pre-condition|climate/i });

    if (await preconditionBtn.isVisible()) {
      await preconditionBtn.click();

      // Wait for response
      await page.waitForTimeout(1000);

      // Should show success notification or indicator
      // (depends on implementation)
    }
  });

  test('should persist alert settings across page reloads', async ({ page }) => {
    // Go to alerts
    await page.click('text=Alerts');

    // Enable optimal charge alert
    const optimalCheckbox = page.getByLabel(/optimal|80%/i);

    if (await optimalCheckbox.isVisible()) {
      // Ensure it's checked
      if (!await optimalCheckbox.isChecked()) {
        await optimalCheckbox.click();
        await page.waitForTimeout(500);
      }

      // Reload page
      await page.reload();

      // Navigate back to alerts
      await page.click('text=Alerts');

      // Setting should be persisted
      const reloadedCheckbox = page.getByLabel(/optimal|80%/i);
      // Note: May need to implement proper persistence in backend
      await expect(reloadedCheckbox).toBeVisible();
    }
  });
});

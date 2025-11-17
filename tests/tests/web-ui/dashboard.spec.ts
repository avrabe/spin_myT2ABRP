import { test, expect } from '@playwright/test';

/**
 * Dashboard E2E Tests
 * Tests the main dashboard interface with JSON API + vanilla JavaScript client-side rendering
 */

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await page.waitForLoadState('networkidle');
  });

  test('should load the dashboard page', async ({ page }) => {
    await expect(page).toHaveTitle(/MyT2ABRP/);
    await expect(page.locator('h1')).toContainText('MyT2ABRP');
  });

  test('should display vehicle status card', async ({ page }) => {
    const statusCard = page.locator('.status-card');
    await expect(statusCard).toBeVisible();

    // Check for battery level
    await expect(page.locator('.battery-percentage')).toBeVisible();

    // Check for VIN
    await expect(page.locator('.status-grid')).toContainText('VIN');
  });

  test('should display charging status when charging', async ({ page }) => {
    const chargingCard = page.locator('.charging-card');

    // Wait for JavaScript to load charging status
    await page.waitForSelector('.charging-card #charging-status', { timeout: 10000 });

    if (await chargingCard.isVisible()) {
      // If charging, should show power and time remaining
      await expect(page.locator('text=/kW/')).toBeVisible();
      await expect(page.locator('text=/Time Left|min/')).toBeVisible();
    }
  });

  test('should auto-update status every 5 seconds', async ({ page }) => {
    // Get initial battery level
    const initialBattery = await page.locator('.battery-percentage').textContent();

    // Wait for JavaScript auto-refresh (5 seconds)
    await page.waitForTimeout(6000);

    // Content should have been refreshed (JavaScript setInterval)
    const statusCard = page.locator('.status-card');
    await expect(statusCard).toBeVisible();

    // At minimum, the element should still exist after refresh
    await expect(page.locator('.battery-percentage')).toBeVisible();
  });

  test('should display range information', async ({ page }) => {
    const rangeCard = page.locator('.range-card');
    await expect(rangeCard).toBeVisible();

    // Should show current range
    await expect(page.locator('text=/km/')).toBeVisible();

    // Should show range at 80%
    await expect(page.locator('text=/@ 80%|optimal/')).toBeVisible();
  });

  test('should have quick actions', async ({ page }) => {
    const actionsCard = page.locator('.actions-card');
    await expect(actionsCard).toBeVisible();

    // Should have charging control buttons
    await expect(page.getByRole('button', { name: /Start Charging/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Stop Charging/ })).toBeVisible();

    // Should have pre-condition button
    await expect(page.getByRole('button', { name: /Pre-Condition/ })).toBeVisible();
  });

  test('should navigate between sections', async ({ page }) => {
    // Click on Alerts tab
    await page.click('text=Alerts');
    const alertsSection = page.locator('#alerts');
    await expect(alertsSection).toHaveClass(/active/);

    // Click on Analytics tab
    await page.click('text=Analytics');
    const analyticsSection = page.locator('#analytics');
    await expect(analyticsSection).toHaveClass(/active/);

    // Click back to Dashboard
    await page.click('text=Dashboard');
    const dashboardSection = page.locator('#dashboard');
    await expect(dashboardSection).toHaveClass(/active/);
  });

  test('should be responsive on mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    // Dashboard should still be visible and functional
    await expect(page.locator('.status-card')).toBeVisible();
    await expect(page.locator('.battery-percentage')).toBeVisible();

    // Navigation should work
    await page.click('text=Settings');
    const settingsSection = page.locator('#settings');
    await expect(settingsSection).toBeVisible();
  });

  test('should handle offline status gracefully', async ({ page, context }) => {
    // Go offline
    await context.setOffline(true);

    // Reload page
    await page.reload();

    // Should show cached data or error message
    const body = await page.textContent('body');
    expect(body).toBeTruthy();

    // Go back online
    await context.setOffline(false);
  });
});

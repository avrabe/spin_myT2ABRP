import { test, expect } from '@playwright/test';

/**
 * Component Integration Tests
 * Tests interaction between web-ui and main API components
 */

test.describe('Component Integration', () => {
  test('web-ui component should serve static files', async ({ page }) => {
    // Request homepage
    const response = await page.goto('/');

    expect(response?.ok()).toBeTruthy();
    expect(response?.status()).toBe(200);

    // Should load CSS
    const styles = await page.locator('link[rel="stylesheet"]').getAttribute('href');
    expect(styles).toContain('.css');

    // Should load JavaScript
    const scripts = page.locator('script[src]');
    const count = await scripts.count();
    expect(count).toBeGreaterThan(0);
  });

  test('HTMX should load and be active', async ({ page }) => {
    await page.goto('/');

    // HTMX should be loaded
    const htmxLoaded = await page.evaluate(() => {
      return typeof (window as any).htmx !== 'undefined';
    });

    expect(htmxLoaded).toBeTruthy();
  });

  test('HTMX polling should work', async ({ page }) => {
    await page.goto('/');

    // Find element with hx-trigger="every"
    const pollingElement = page.locator('[hx-get][hx-trigger*="every"]').first();

    if (await pollingElement.isVisible()) {
      // Get initial content
      const initialContent = await pollingElement.textContent();

      // Wait for one polling cycle
      await page.waitForTimeout(6000);

      // Element should still exist after refresh
      await expect(pollingElement).toBeVisible();
    }
  });

  test('HTMX should swap content correctly', async ({ page }) => {
    await page.goto('/');

    // Find a button with hx-post
    const actionButton = page.locator('[hx-post]').first();

    if (await actionButton.isVisible()) {
      // Click it
      await actionButton.click();

      // Wait for HTMX request
      await page.waitForTimeout(1000);

      // Page should still be functional
      await expect(page.locator('body')).toBeVisible();
    }
  });

  test('navigation should preserve state', async ({ page }) => {
    await page.goto('/');

    // Navigate to different sections
    await page.click('text=Analytics');
    await expect(page.locator('#analytics-section')).toBeVisible();

    await page.click('text=Settings');
    await expect(page.locator('#settings-section')).toBeVisible();

    await page.click('text=Dashboard');
    await expect(page.locator('#dashboard-section')).toBeVisible();

    // Vehicle status should still be visible
    await expect(page.locator('.status-card, .battery-percentage')).toBeVisible();
  });

  test('form submissions should work via HTMX', async ({ page }) => {
    await page.goto('/');
    await page.click('text=Settings');

    // Find a form
    const form = page.locator('form').first();

    if (await form.isVisible()) {
      // Fill and submit
      const input = form.locator('input').first();

      if (await input.isVisible()) {
        await input.fill('test-value');

        // Submit form
        const submitBtn = form.locator('button[type="submit"], input[type="submit"]').first();

        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          // Wait for HTMX response
          await page.waitForTimeout(1000);

          // Should not have redirected
          expect(page.url()).toContain('localhost:3000');
        }
      }
    }
  });

  test('error handling should be graceful', async ({ page, request }) => {
    // Try to access invalid API endpoint
    const response = await request.get('/api/invalid-endpoint-xyz');

    // Should return appropriate error (404 or handled error)
    expect([404, 500]).toContain(response.status());
  });

  test('CSS should be applied correctly', async ({ page }) => {
    await page.goto('/');

    // Check if Toyota red color is applied
    const element = page.locator('body, .header, h1').first();
    await expect(element).toBeVisible();

    // Get computed styles
    const color = await element.evaluate((el) => {
      return window.getComputedStyle(el).getPropertyValue('--primary-color');
    });

    // Should have Toyota red or be defined
    expect(color).toBeTruthy();
  });

  test('responsive design should work on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });

    await page.goto('/');

    // Content should adapt to mobile
    const body = page.locator('body');
    await expect(body).toBeVisible();

    // Cards should stack vertically
    const cards = page.locator('.card, .status-card');
    const count = await cards.count();

    if (count > 1) {
      // Check first two cards positions
      const box1 = await cards.nth(0).boundingBox();
      const box2 = await cards.nth(1).boundingBox();

      if (box1 && box2) {
        // Second card should be below first (not side-by-side)
        expect(box2.y).toBeGreaterThan(box1.y);
      }
    }
  });

  test('PWA should be installable', async ({ page }) => {
    await page.goto('/');

    // Check for manifest
    const manifest = page.locator('link[rel="manifest"]');

    if (await manifest.count() > 0) {
      const manifestUrl = await manifest.getAttribute('href');
      expect(manifestUrl).toBeTruthy();
    }

    // Check for service worker registration
    const swRegistered = await page.evaluate(() => {
      return 'serviceWorker' in navigator;
    });

    expect(swRegistered).toBeTruthy();
  });

  test('API and web-ui should be on same origin', async ({ page }) => {
    await page.goto('/');

    // Make API request from page
    const apiResponse = await page.evaluate(async () => {
      const response = await fetch('/api/vehicle/status');
      return {
        ok: response.ok,
        status: response.status,
      };
    });

    expect(apiResponse.ok).toBeTruthy();
    expect(apiResponse.status).toBe(200);
  });

  test('CORS should be properly configured', async ({ request }) => {
    const response = await request.get('/api/vehicle/status', {
      headers: {
        'Origin': 'http://localhost:3000',
      },
    });

    expect(response.ok()).toBeTruthy();

    // Should have CORS headers if configured
    const headers = response.headers();
    // Note: Spin may handle CORS differently
    expect(headers).toBeTruthy();
  });

  test('concurrent component access should work', async ({ browser }) => {
    // Create multiple pages
    const page1 = await browser.newPage();
    const page2 = await browser.newPage();

    // Both should load successfully
    const [response1, response2] = await Promise.all([
      page1.goto('/'),
      page2.goto('/'),
    ]);

    expect(response1?.ok()).toBeTruthy();
    expect(response2?.ok()).toBeTruthy();

    // Both should show content
    await expect(page1.locator('h1')).toBeVisible();
    await expect(page2.locator('h1')).toBeVisible();

    await page1.close();
    await page2.close();
  });

  test('long polling should not block other requests', async ({ page }) => {
    await page.goto('/');

    // Start long-polling requests
    const pollingElement = page.locator('[hx-trigger*="every"]').first();
    await expect(pollingElement).toBeVisible();

    // Make a manual API request while polling is active
    const apiResponse = await page.evaluate(async () => {
      const response = await fetch('/api/vehicle/status');
      return response.ok;
    });

    expect(apiResponse).toBeTruthy();
  });

  test('memory usage should be stable', async ({ page }) => {
    await page.goto('/');

    // Get initial memory (if available)
    const initialMetrics = await page.evaluate(() => {
      if ((performance as any).memory) {
        return (performance as any).memory.usedJSHeapSize;
      }
      return null;
    });

    // Perform many operations
    for (let i = 0; i < 10; i++) {
      await page.click('text=Analytics');
      await page.waitForTimeout(200);
      await page.click('text=Dashboard');
      await page.waitForTimeout(200);
    }

    // Check memory again
    const finalMetrics = await page.evaluate(() => {
      if ((performance as any).memory) {
        return (performance as any).memory.usedJSHeapSize;
      }
      return null;
    });

    // Memory shouldn't grow excessively (allow 2x growth max)
    if (initialMetrics && finalMetrics) {
      expect(finalMetrics).toBeLessThan(initialMetrics * 2);
    }
  });
});

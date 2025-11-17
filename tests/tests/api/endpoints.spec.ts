import { test, expect } from '@playwright/test';

/**
 * API Endpoint Tests
 * Tests all API endpoints for correctness and performance
 */

test.describe('API Endpoints', () => {
  test('GET /api/vehicle/status should return HTML fragment', async ({ request }) => {
    const response = await request.get('/api/vehicle/status');

    expect(response.ok()).toBeTruthy();
    expect(response.status()).toBe(200);

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('text/html');

    const body = await response.text();
    expect(body).toContain('Vehicle Status');
    expect(body).toContain('%'); // Battery percentage
    expect(body).toContain('km'); // Range
  });

  test('GET /api/charging/status should return charging info', async ({ request }) => {
    const response = await request.get('/api/charging/status');

    expect(response.ok()).toBeTruthy();
    expect(response.status()).toBe(200);

    const body = await response.text();
    expect(body).toContain('Charging');
    expect(body).toContain('kW'); // Power
  });

  test('GET /api/range should return range information', async ({ request }) => {
    const response = await request.get('/api/range');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('km');
    expect(body).toContain('Range');
  });

  test('GET /api/battery/health should return health metrics', async ({ request }) => {
    const response = await request.get('/api/battery/health');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('Health');
    expect(body).toContain('%');
    expect(body).toContain('Cycle'); // Charge cycles
    expect(body).toContain('°C'); // Temperature
  });

  test('GET /api/charging/history should return session list', async ({ request }) => {
    const response = await request.get('/api/charging/history');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('history');
    expect(body).toContain('%'); // Battery levels
    expect(body).toContain('kWh'); // Energy
  });

  test('GET /api/alerts/active should return active alerts', async ({ request }) => {
    const response = await request.get('/api/alerts/active');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('alert');
  });

  test('GET /api/analytics/weekly should return weekly stats', async ({ request }) => {
    const response = await request.get('/api/analytics/weekly');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('Weekly');
    expect(body).toContain('kWh');
  });

  test('GET /api/analytics/costs should return cost analysis', async ({ request }) => {
    const response = await request.get('/api/analytics/costs');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('Cost');
    expect(body).toContain('€');
  });

  test('GET /api/analytics/efficiency should return efficiency metrics', async ({ request }) => {
    const response = await request.get('/api/analytics/efficiency');

    expect(response.ok()).toBeTruthy();

    const body = await response.text();
    expect(body).toContain('Efficiency');
    expect(body).toContain('%');
  });

  test('POST /api/charging/start should return success', async ({ request }) => {
    const response = await request.post('/api/charging/start');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body.success).toBeTruthy();
    expect(body.message).toContain('started');
  });

  test('POST /api/charging/stop should return success', async ({ request }) => {
    const response = await request.post('/api/charging/stop');

    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.success).toBeTruthy();
    expect(body.message).toContain('stopped');
  });

  test('POST /api/precondition should return success', async ({ request }) => {
    const response = await request.post('/api/precondition');

    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.success).toBeTruthy();
    expect(body.message).toContain('started');
  });

  test('POST /api/alerts/save should accept form data', async ({ request }) => {
    const response = await request.post('/api/alerts/save', {
      data: {
        charge_complete: true,
        optimal_charge: true,
        custom_level: 90,
      },
    });

    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.success).toBeTruthy();
  });

  test('GET /styles.css should return CSS', async ({ request }) => {
    const response = await request.get('/styles.css');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('text/css');

    const body = await response.text();
    expect(body).toContain(':root'); // CSS variables
    expect(body).toContain('--primary-color');
  });

  test('GET /app.js should return JavaScript', async ({ request }) => {
    const response = await request.get('/app.js');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/javascript');

    const body = await response.text();
    expect(body.length).toBeGreaterThan(0);
  });

  test('GET / should return index page', async ({ request }) => {
    const response = await request.get('/');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('text/html');

    const body = await response.text();
    expect(body).toContain('<!DOCTYPE html>');
    expect(body).toContain('Toyota MyT'); // Title contains "Toyota MyT2ABRP"
  });

  test('GET /nonexistent should return 404', async ({ request }) => {
    const response = await request.get('/nonexistent-path-12345');

    expect(response.status()).toBe(404);
  });

  test('API responses should be fast (< 100ms)', async ({ request }) => {
    const start = Date.now();
    await request.get('/api/vehicle/status');
    const duration = Date.now() - start;

    expect(duration).toBeLessThan(100);
  });

  test('API should handle concurrent requests', async ({ request }) => {
    // Make 10 concurrent requests
    const promises = Array.from({ length: 10 }, () =>
      request.get('/api/vehicle/status')
    );

    const responses = await Promise.all(promises);

    // All should succeed
    responses.forEach(response => {
      expect(response.ok()).toBeTruthy();
    });
  });

  test('Static files should have cache headers', async ({ request }) => {
    const response = await request.get('/styles.css');

    const cacheControl = response.headers()['cache-control'];
    expect(cacheControl).toBeTruthy();
    expect(cacheControl).toContain('max-age');
  });
});

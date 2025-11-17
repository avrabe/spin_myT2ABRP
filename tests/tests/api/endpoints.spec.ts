import { test, expect } from '@playwright/test';

/**
 * API Endpoint Tests
 * Tests all JSON API endpoints for correctness and performance
 * Architecture: Clean JSON API with client-side rendering (vanilla JavaScript)
 */

test.describe('API Endpoints', () => {
  test('GET /api/vehicle/status should return JSON', async ({ request }) => {
    const response = await request.get('/api/vehicle/status');

    expect(response.ok()).toBeTruthy();
    expect(response.status()).toBe(200);

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('vin');
    expect(body).toHaveProperty('battery_level');
    expect(body).toHaveProperty('range_km');
    expect(body).toHaveProperty('is_charging');
    expect(body).toHaveProperty('is_connected');
    expect(body.battery_level).toBeGreaterThanOrEqual(0);
    expect(body.battery_level).toBeLessThanOrEqual(100);
  });

  test('GET /api/charging/status should return JSON charging info', async ({ request }) => {
    const response = await request.get('/api/charging/status');

    expect(response.ok()).toBeTruthy();
    expect(response.status()).toBe(200);

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('is_charging');
    expect(body).toHaveProperty('current_level');
    expect(body).toHaveProperty('target_level');
    expect(body).toHaveProperty('power_kw');
    expect(body).toHaveProperty('charge_rate_kwh');
  });

  test('GET /api/range should return JSON range information', async ({ request }) => {
    const response = await request.get('/api/range');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('estimated_range_km');
    expect(body).toHaveProperty('range_at_80_percent_km');
    expect(body).toHaveProperty('range_at_100_percent_km');
  });

  test('GET /api/battery/health should return JSON health metrics', async ({ request }) => {
    const response = await request.get('/api/battery/health');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('capacity_percentage');
    expect(body).toHaveProperty('health_status');
    expect(body).toHaveProperty('cycles');
    expect(body).toHaveProperty('temperature_celsius');
  });

  test('GET /api/charging/history should return JSON session list', async ({ request }) => {
    const response = await request.get('/api/charging/history');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(Array.isArray(body)).toBeTruthy();
    if (body.length > 0) {
      expect(body[0]).toHaveProperty('date');
      expect(body[0]).toHaveProperty('start_level');
      expect(body[0]).toHaveProperty('end_level');
      expect(body[0]).toHaveProperty('duration_minutes');
      expect(body[0]).toHaveProperty('energy_kwh');
    }
  });

  test('GET /api/alerts/active should return JSON active alerts', async ({ request }) => {
    const response = await request.get('/api/alerts/active');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(Array.isArray(body)).toBeTruthy();
    if (body.length > 0) {
      expect(body[0]).toHaveProperty('type');
      expect(body[0]).toHaveProperty('title');
      expect(body[0]).toHaveProperty('message');
      expect(body[0]).toHaveProperty('time_ago');
    }
  });

  test('GET /api/analytics/weekly should return JSON weekly stats', async ({ request }) => {
    const response = await request.get('/api/analytics/weekly');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('charging_sessions');
    expect(body).toHaveProperty('total_energy_kwh');
    expect(body).toHaveProperty('avg_duration_minutes');
  });

  test('GET /api/analytics/costs should return JSON cost analysis', async ({ request }) => {
    const response = await request.get('/api/analytics/costs');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('this_week_cost');
    expect(body).toHaveProperty('per_session_avg');
    expect(body).toHaveProperty('avg_price_per_kwh');
    expect(body).toHaveProperty('currency');
  });

  test('GET /api/analytics/efficiency should return JSON efficiency metrics', async ({ request }) => {
    const response = await request.get('/api/analytics/efficiency');

    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('charging_efficiency_percent');
    expect(body).toHaveProperty('avg_consumption_kwh_per_100km');
    expect(body).toHaveProperty('battery_health_percent');
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

// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('Performance Tests', () => {

  test('measure response time for health endpoint', async ({ request }) => {
    const iterations = 100;
    const times = [];

    for (let i = 0; i < iterations; i++) {
      const start = Date.now();
      await request.get('/health');
      times.push(Date.now() - start);
    }

    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    const max = Math.max(...times);
    const min = Math.min(...times);

    console.log(`Performance Stats (${iterations} requests):`);
    console.log(`  Average: ${avg.toFixed(2)}ms`);
    console.log(`  Min: ${min}ms`);
    console.log(`  Max: ${max}ms`);

    expect(avg).toBeLessThan(100); // Average should be under 100ms
    expect(max).toBeLessThan(500); // No request should take more than 500ms
  });

  test('measure throughput', async ({ request }) => {
    const duration = 5000; // 5 seconds
    const start = Date.now();
    let requests = 0;

    while (Date.now() - start < duration) {
      await request.get('/health');
      requests++;
    }

    const actualDuration = (Date.now() - start) / 1000;
    const rps = requests / actualDuration;

    console.log(`Throughput: ${rps.toFixed(2)} req/sec`);
    expect(rps).toBeGreaterThan(10); // Should handle at least 10 req/sec
  });

  test('measure memory stability under load', async ({ request }) => {
    // Make 1000 requests and ensure consistent response times
    const requests = [];
    const batchSize = 50;

    for (let batch = 0; batch < 20; batch++) {
      const batchStart = Date.now();
      const batchRequests = [];

      for (let i = 0; i < batchSize; i++) {
        batchRequests.push(request.get('/health'));
      }

      await Promise.all(batchRequests);
      const batchDuration = Date.now() - batchStart;
      requests.push(batchDuration);

      // Small delay between batches
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    // Check that later batches aren't significantly slower (memory leak indicator)
    const firstHalf = requests.slice(0, 10).reduce((a, b) => a + b, 0) / 10;
    const secondHalf = requests.slice(10).reduce((a, b) => a + b, 0) / 10;
    const degradation = ((secondHalf - firstHalf) / firstHalf) * 100;

    console.log(`First half avg: ${firstHalf.toFixed(2)}ms`);
    console.log(`Second half avg: ${secondHalf.toFixed(2)}ms`);
    console.log(`Degradation: ${degradation.toFixed(2)}%`);

    expect(degradation).toBeLessThan(50); // Less than 50% degradation
  });

  test('measure component composition overhead', async ({ request }) => {
    // Compare simple endpoint vs composed endpoint
    const simpleRequests = 50;
    const simpleTimes = [];

    for (let i = 0; i < simpleRequests; i++) {
      const start = Date.now();
      await request.get('/health');
      simpleTimes.push(Date.now() - start);
    }

    const composedTimes = [];
    for (let i = 0; i < simpleRequests; i++) {
      const start = Date.now();
      await request.post('/validate', {
        data: { vin: 'TEST', data: {} }
      });
      composedTimes.push(Date.now() - start);
    }

    const simpleAvg = simpleTimes.reduce((a, b) => a + b, 0) / simpleTimes.length;
    const composedAvg = composedTimes.reduce((a, b) => a + b, 0) / composedTimes.length;

    console.log(`Simple endpoint avg: ${simpleAvg.toFixed(2)}ms`);
    console.log(`Composed endpoint avg: ${composedAvg.toFixed(2)}ms`);
    console.log(`Composition overhead: ${((composedAvg - simpleAvg) / simpleAvg * 100).toFixed(2)}%`);

    // Composed endpoints should be slower but not excessively
    expect(composedAvg).toBeLessThan(simpleAvg * 5);
  });
});

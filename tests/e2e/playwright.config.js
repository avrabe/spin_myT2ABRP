// Playwright configuration for Toyota MyT2ABRP E2E tests
// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './tests',

  // Maximum time one test can run
  timeout: 30 * 1000,

  // Test execution settings
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,

  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['junit', { outputFile: 'test-results/junit.xml' }],
    ['list']
  ],

  use: {
    // Base URL for Spin application
    baseURL: process.env.BASE_URL || 'http://127.0.0.1:3000',

    // Collect trace on failure
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',

    // Network settings
    ignoreHTTPSErrors: true,
  },

  // Configure projects for different scenarios
  projects: [
    {
      name: 'api-tests',
      testMatch: /api\.spec\.js/,
      use: {
        ...devices['Desktop Chrome'],
        // API tests don't need browser features
        headless: true,
      },
    },
    {
      name: 'performance-tests',
      testMatch: /performance\.spec\.js/,
      use: {
        ...devices['Desktop Chrome'],
        headless: true,
      },
    },
  ],

  // Web server configuration for Spin
  webServer: {
    command: 'cd ../.. && spin up',
    url: 'http://127.0.0.1:3000',
    timeout: 30 * 1000,
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
  },
});

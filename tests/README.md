# MyT2ABRP Testing Suite

Comprehensive end-to-end and integration tests using Playwright for the MyT2ABRP web UI and API components.

## Overview

This test suite validates:
- **Web UI functionality** (HTMX dashboard, user interactions)
- **API endpoints** (correctness, performance, error handling)
- **Component integration** (web-ui + main API)
- **Cross-browser compatibility** (Chrome, Firefox, Safari, Mobile)
- **Performance** (response times, memory usage)
- **Accessibility** (responsive design, offline handling)

## Setup

### Prerequisites

- Node.js 18+ installed
- Spin CLI installed (`spin --version`)
- Project built (`spin build`)

### Installation

```bash
cd tests
npm install
npx playwright install  # Install browser drivers
```

## Running Tests

### All Tests

```bash
npm test
```

### Specific Test Suites

```bash
# Web UI tests only
npm run test:web-ui

# API endpoint tests only
npm run test:api

# Integration tests only
npm run test:integration
```

### Interactive Debugging

```bash
# Run tests with browser visible
npm run test:headed

# Step-through debugging
npm run test:debug

# Interactive UI mode
npm run test:ui
```

### View Test Reports

```bash
npm run test:report
```

## Test Structure

```
tests/
├── package.json              # Dependencies and scripts
├── playwright.config.ts      # Playwright configuration
├── tests/
│   ├── web-ui/              # Web UI end-to-end tests
│   │   ├── dashboard.spec.ts
│   │   └── charging.spec.ts
│   ├── api/                 # API endpoint tests
│   │   └── endpoints.spec.ts
│   └── integration/         # Component integration tests
│       └── components.spec.ts
└── test-results/            # Generated test results
```

## Test Categories

### 1. Web UI Tests (`tests/web-ui/`)

Tests the HTMX-based web dashboard.

**Dashboard Tests** (`dashboard.spec.ts`):
- Page loading and rendering
- Vehicle status display
- Charging status updates
- Real-time auto-refresh (HTMX polling)
- Range information
- Quick actions functionality
- Section navigation
- Mobile responsiveness
- Offline handling

**Charging Tests** (`charging.spec.ts`):
- Start/stop charging controls
- Charging progress display
- Alert configuration
- Charging history
- Real-time status updates
- Weekly statistics
- Cost calculations
- Pre-conditioning
- Settings persistence

### 2. API Endpoint Tests (`tests/api/`)

Validates all API endpoints for correctness and performance.

**Endpoint Tests** (`endpoints.spec.ts`):
- GET `/api/vehicle/status` - Vehicle status HTML fragment
- GET `/api/charging/status` - Charging information
- GET `/api/range` - Range calculations
- GET `/api/battery/health` - Battery health metrics
- GET `/api/charging/history` - Charging session history
- GET `/api/alerts/active` - Active alerts
- GET `/api/analytics/weekly` - Weekly statistics
- GET `/api/analytics/costs` - Cost analysis
- GET `/api/analytics/efficiency` - Efficiency metrics
- POST `/api/charging/start` - Start charging
- POST `/api/charging/stop` - Stop charging
- POST `/api/precondition` - Pre-condition vehicle
- POST `/api/alerts/save` - Save alert settings
- GET `/styles.css` - Static CSS file
- GET `/app.js` - Static JavaScript file
- GET `/` - Index page
- 404 handling
- Response time validation (< 100ms)
- Concurrent request handling
- Cache headers validation

### 3. Integration Tests (`tests/integration/`)

Tests interaction between components.

**Component Integration Tests** (`components.spec.ts`):
- Static file serving (web-ui component)
- HTMX loading and activation
- HTMX polling functionality
- HTMX content swapping
- Navigation state preservation
- Form submissions via HTMX
- Error handling
- CSS application
- Responsive design
- PWA installability
- Same-origin API calls
- CORS configuration
- Concurrent access
- Long-polling behavior
- Memory stability

## Browser Coverage

Tests run on:
- **Desktop**: Chrome, Firefox, Safari
- **Mobile**: Mobile Chrome (Pixel 5), Mobile Safari (iPhone 12)

## Performance Benchmarks

- **API Response Time**: < 100ms
- **Page Load**: < 2s
- **HTMX Refresh**: < 500ms
- **Memory Growth**: < 2x during 10 navigation cycles

## CI/CD Integration

Tests are configured to run in CI with:
- Retries on failure (2 retries)
- Single worker (sequential execution)
- Trace collection on failure
- Screenshots on failure
- Video recording on failure

## Writing New Tests

### Example Test

```typescript
import { test, expect } from '@playwright/test';

test.describe('My Feature', () => {
  test('should do something', async ({ page }) => {
    await page.goto('/');

    const element = page.locator('.my-element');
    await expect(element).toBeVisible();
    await expect(element).toContainText('Expected Text');
  });
});
```

### Best Practices

1. **Use data-testid**: Add `data-testid` attributes for stable selectors
2. **Wait for network**: Use `page.waitForResponse()` for API calls
3. **Mobile testing**: Test responsiveness with viewport changes
4. **Accessibility**: Include ARIA attributes in assertions
5. **Performance**: Measure critical user flows
6. **Error states**: Test failure scenarios
7. **Isolation**: Each test should be independent

## Debugging Failed Tests

### Local Debugging

```bash
# Run failing test in debug mode
npm run test:debug -- tests/web-ui/dashboard.spec.ts

# Run with traces
npm test -- --trace on

# Generate report
npm run test:report
```

### CI Debugging

1. Download test artifacts from CI
2. View screenshots in `test-results/`
3. Watch videos of failed tests
4. Inspect traces with `npx playwright show-trace trace.zip`

## Known Issues

### Timeouts

If tests timeout waiting for Spin to start:
- Increase `webServer.timeout` in `playwright.config.ts`
- Ensure `spin build` completes successfully
- Check that port 3000 is not in use

### HTMX Timing

Some HTMX auto-refresh tests may be flaky due to timing:
- Increase `waitForTimeout` values if needed
- Use `page.waitForResponse()` instead of fixed timeouts

### Mobile Tests

Mobile viewport tests may fail if:
- CSS media queries are incorrect
- Touch events are not properly handled

## Maintenance

### Updating Dependencies

```bash
cd tests
npm update
npx playwright install  # Update browsers
```

### Adding New Test Suites

1. Create new `.spec.ts` file in appropriate directory
2. Follow existing test patterns
3. Run tests locally before committing
4. Update this README with new test descriptions

## Continuous Improvement

### Test Coverage Goals

- [ ] 100% API endpoint coverage
- [ ] All user workflows tested
- [ ] All error states validated
- [ ] Performance benchmarks for all critical paths
- [x] Cross-browser compatibility
- [x] Mobile responsiveness
- [ ] Accessibility compliance (WCAG 2.1 AA)

### Future Enhancements

- Visual regression testing
- Load testing with k6
- Security testing (OWASP)
- Accessibility audits (axe-core)
- Contract testing between components
- Mutation testing for code quality

## Resources

- [Playwright Documentation](https://playwright.dev)
- [HTMX Testing Guide](https://htmx.org/docs/#testing)
- [Spin Documentation](https://developer.fermyon.com/spin)

## Support

For issues or questions:
1. Check test output and traces
2. Review this README
3. Open an issue on GitHub
4. Contact: Ralf Anton Beier <ralf_beier@me.com>

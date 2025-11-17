# Testing Guide

Comprehensive testing documentation for MyT2ABRP.

## Table of Contents

- [Testing Philosophy](#testing-philosophy)
- [Test Types](#test-types)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [CI/CD Integration](#cicd-integration)
- [Performance Testing](#performance-testing)
- [Security Testing](#security-testing)

## Testing Philosophy

MyT2ABRP follows a **comprehensive testing approach**:

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete user workflows
4. **Performance Tests**: Ensure speed and scalability
5. **Security Tests**: Verify security controls

### Coverage Goals

- **Unit Tests**: 80%+ coverage
- **Integration Tests**: All critical paths
- **E2E Tests**: All user workflows
- **Performance**: Meet SLA targets
- **Security**: Zero critical vulnerabilities

## Test Types

### 1. Unit Tests (Rust)

**Location**: `web-ui/src/`

**Run**:
```bash
cd web-ui
cargo test

# With coverage
cargo tarpaulin --out Html
```

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_status_serialization() {
        let status = VehicleStatus {
            vin: "TEST123".to_string(),
            battery_level: 85,
            range_km: 320,
            is_charging: true,
            is_connected: true,
            location: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("TEST123"));
        assert!(json.contains("85"));
    }
}
```

### 2. Integration Tests (Rust)

**Location**: `web-ui/tests/`

**Run**:
```bash
cd web-ui
cargo test --test integration_tests
```

### 3. End-to-End Tests (Playwright)

**Location**: `tests/e2e/`

**Run**:
```bash
cd tests
npm test

# Specific test file
npx playwright test web-ui.spec.ts

# With headed browser
npm run test:headed

# Debug mode
npm run test:debug
```

**Example**:
```typescript
test('vehicle status loads correctly', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Check battery level displayed
  await expect(page.locator('.battery-percentage'))
    .toContainText('85%');

  // Check charging status
  await expect(page.locator('.stat-value'))
    .toContainText('⚡ Charging');
});
```

### 4. Load Tests

**Run**:
```bash
./loadtest.sh full

# Specific test
./loadtest.sh load http://localhost:3000/api/vehicle/status
```

### 5. Smoke Tests

**Run**:
```bash
./smoke-test.sh

# Against production
./smoke-test.sh --url https://myt2abrp.example.com
```

### 6. Health Checks

**Run**:
```bash
./health-check.sh

# Verbose output
./health-check.sh --verbose

# JSON output
./health-check.sh --json
```

## Running Tests

### Quick Test (All)

```bash
make test
```

### Individual Test Suites

```bash
# Rust unit tests
cd web-ui && cargo test

# Playwright E2E tests
cd tests && npm test

# Load tests
./loadtest.sh full

# Smoke tests
./smoke-test.sh

# Health check
./health-check.sh
```

### CI/CD Tests

```bash
# Run full CI pipeline locally
make ci

# Or manually
make lint
make audit
make build
make test
```

## Writing Tests

### Best Practices

1. **Test naming**: Use descriptive names
   ```rust
   #[test]
   fn test_battery_level_validation_rejects_over_100() { }
   ```

2. **Arrange-Act-Assert pattern**:
   ```rust
   #[test]
   fn test_example() {
       // Arrange
       let input = create_test_input();

       // Act
       let result = function_under_test(input);

       // Assert
       assert_eq!(result, expected_output);
   }
   ```

3. **Test one thing per test**:
   ```rust
   // ✅ Good
   #[test]
   fn test_battery_level_validation() {
       assert!(is_valid_battery_level(85));
   }

   #[test]
   fn test_battery_level_over_100_invalid() {
       assert!(!is_valid_battery_level(101));
   }

   // ❌ Bad
   #[test]
   fn test_battery_validation() {
       assert!(is_valid_battery_level(85));
       assert!(!is_valid_battery_level(101));
       assert!(is_valid_battery_level(0));
   }
   ```

4. **Use fixtures for common data**:
   ```rust
   fn create_test_vehicle_status() -> VehicleStatus {
       VehicleStatus {
           vin: "TEST123".to_string(),
           battery_level: 85,
           range_km: 320,
           is_charging: true,
           is_connected: true,
           location: None,
       }
   }
   ```

5. **Mock external dependencies**:
   ```rust
   #[test]
   fn test_api_call_with_mock() {
       let mock_server = MockServer::start();
       // Configure mock responses
       // Test your code
   }
   ```

### Test Coverage

**Generate coverage report**:
```bash
cd web-ui
cargo tarpaulin --out Html --output-dir target/coverage

# Open report
open target/coverage/index.html
```

**Coverage goals by module**:
- Core business logic: 90%+
- API handlers: 80%+
- Utilities: 70%+

## CI/CD Integration

### GitHub Actions

Tests run automatically on:
- **Every push** to any branch
- **Every pull request**
- **Scheduled** (daily at 00:00 UTC)

**Pipeline**:
```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: make test
```

### Pre-commit Hooks

Install pre-commit hooks:
```bash
ln -s ../../git-hooks/pre-commit .git/hooks/pre-commit
```

Runs automatically before each commit:
- Rust formatting check
- Clippy lints
- Secret detection
- TODO detection
- Debug statement check

## Performance Testing

### Load Testing

**Full test suite**:
```bash
./loadtest.sh full
```

**Individual test modes**:

1. **Load Test** (concurrent users):
   ```bash
   ./loadtest.sh load http://localhost:3000/ 100 30
   # 100 concurrent users for 30 seconds
   ```

2. **Ramp-up Test** (gradual increase):
   ```bash
   ./loadtest.sh ramp http://localhost:3000/
   # Gradually increase load
   ```

3. **Stress Test** (find breaking point):
   ```bash
   ./loadtest.sh stress http://localhost:3000/
   # Increase until failure
   ```

4. **Endurance Test** (sustained load):
   ```bash
   ./loadtest.sh endurance http://localhost:3000/ 60
   # Sustained load for 60 minutes
   ```

### Benchmarking

```bash
./benchmark.sh
```

**Measures**:
- Request latency (P50, P95, P99)
- Throughput (requests per second)
- Memory usage
- CPU usage
- WASM binary size

### Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Cold Start | < 10ms | ~8ms |
| P50 Latency | < 10ms | 5ms |
| P95 Latency | < 50ms | 15ms |
| P99 Latency | < 100ms | 50ms |
| Throughput | 1000+ req/s | 1,200 req/s |
| Memory/Request | < 1MB | 0.8MB |

## Security Testing

### Automated Security Scans

**Run security audit**:
```bash
./security-audit.sh

# With auto-fix
./security-audit.sh --fix
```

**Checks**:
- Dependency vulnerabilities (cargo-audit)
- Secret detection
- License compliance
- Code quality (Clippy)
- File permissions
- Docker security
- WASM binary analysis
- Configuration validation

### Manual Security Testing

**1. Authentication Testing**:
```bash
# Test with invalid token
curl -H "Authorization: Bearer invalid" \
     http://localhost:3000/api/vehicle/status

# Test without token
curl http://localhost:3000/api/vehicle/status
```

**2. CORS Testing**:
```bash
curl -H "Origin: https://evil.com" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     http://localhost:3000/api/vehicle/status
```

**3. Rate Limiting**:
```bash
# Test rate limits
for i in {1..20}; do
  curl http://localhost:3000/api/vehicle/status
done
```

**4. Input Validation**:
```bash
# Test with malicious input
curl -X POST \
     -H "Content-Type: application/json" \
     -d '{"vin":"<script>alert(1)</script>"}' \
     http://localhost:3000/api/vehicle
```

### Penetration Testing

**Recommended tools**:
- **OWASP ZAP**: Web application security scanner
- **Burp Suite**: Security testing platform
- **sqlmap**: SQL injection testing
- **nikto**: Web server scanner

## Test Data

### Mock Data

**Location**: `tests/fixtures/`

**Example**:
```json
{
  "vehicle_status": {
    "vin": "TEST123456789",
    "battery_level": 85,
    "range_km": 320,
    "is_charging": true
  }
}
```

### Test Database

For integration tests requiring database:
```bash
# Start test database
docker run -d -p 5433:5432 \
  -e POSTGRES_PASSWORD=test \
  postgres:15-alpine

# Run tests
DATABASE_URL=postgresql://postgres:test@localhost:5433/test make test
```

## Troubleshooting Tests

### Common Issues

**1. Tests fail locally but pass in CI**:
- Check environment variables
- Ensure clean state between tests
- Check file paths (absolute vs relative)

**2. Flaky E2E tests**:
- Add explicit waits
- Increase timeouts
- Check for race conditions

**3. Performance tests inconsistent**:
- Run on dedicated hardware
- Close other applications
- Use multiple runs and average

**4. Coverage drops**:
- Add tests for new code
- Remove dead code
- Check coverage report for gaps

### Debug Tests

**Rust tests**:
```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run with backtraces
RUST_BACKTRACE=1 cargo test
```

**Playwright tests**:
```bash
# Debug mode
npx playwright test --debug

# Headed mode
npx playwright test --headed

# Trace mode
npx playwright test --trace on
```

## Test Reports

### Generate Reports

**Test results**:
```bash
# Rust tests (JUnit XML)
cargo test -- -Z unstable-options --format json --report-time

# Playwright tests (HTML report)
npx playwright test --reporter=html
```

### View Reports

**Playwright HTML report**:
```bash
npx playwright show-report
```

**Coverage report**:
```bash
open web-ui/target/coverage/index.html
```

## Continuous Improvement

### Test Metrics to Track

1. **Code coverage** (target: 80%+)
2. **Test execution time** (target: < 5 minutes)
3. **Flaky test rate** (target: < 1%)
4. **Test maintenance burden**

### Review Process

- **Weekly**: Review flaky tests
- **Monthly**: Update test targets
- **Quarterly**: Performance test review
- **Annually**: Test strategy review

## Resources

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Playwright Docs](https://playwright.dev)
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [Performance Testing Best Practices](https://www.nginx.com/blog/testing-the-performance-of-nginx-and-nginx-plus-web-servers/)

---

**Questions?** See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) or open an issue.

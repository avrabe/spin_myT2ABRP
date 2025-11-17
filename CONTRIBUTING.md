# Contributing to MyT2ABRP

First off, thank you for considering contributing to MyT2ABRP! It's people like you that make MyT2ABRP such a great tool.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Testing Guidelines](#testing-guidelines)
- [Code Style](#code-style)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Project Structure](#project-structure)

## Code of Conduct

This project and everyone participating in it is governed by basic principles of respect and professionalism. By participating, you are expected to uphold this code. Please report unacceptable behavior to ralf_beier@me.com.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples**
- **Describe the behavior you observed** and **explain which behavior you expected to see** instead
- **Include screenshots** if relevant
- **Include your environment details**: OS, Rust version, Spin version, browser (for web UI)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use a clear and descriptive title**
- **Provide a detailed description** of the suggested enhancement
- **Explain why this enhancement would be useful** to most MyT2ABRP users
- **List some examples** of how this enhancement would be used

### Your First Code Contribution

Unsure where to begin? You can start by looking through `good-first-issue` and `help-wanted` issues:

- **Good first issues** - issues which should only require a few lines of code
- **Help wanted issues** - issues which are a bit more involved

### Pull Requests

- Fill in the required template
- Follow the [code style](#code-style) guidelines
- Include tests when adding new features
- Update documentation when changing functionality
- End all files with a newline

## Development Setup

### Prerequisites

```bash
# Required
rust >= 1.70
spin >= 2.7.0
node.js >= 20 (for tests)

# Optional
bazel >= 7.0 (for main API component)
xcode >= 15 (for iOS/watchOS development)
```

### Initial Setup

1. **Fork and clone the repository**

```bash
git clone https://github.com/<your-username>/spin_myT2ABRP.git
cd spin_myT2ABRP
```

2. **Install Rust toolchain**

```bash
rustup target add wasm32-wasip2
```

3. **Install Spin CLI**

```bash
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv spin /usr/local/bin/
```

4. **Set up environment**

```bash
cp .env.example .env
# Edit .env with your configuration
```

5. **Build the project**

```bash
spin build
```

6. **Run the development server**

```bash
spin up
```

7. **Install test dependencies**

```bash
cd tests
npm install
npx playwright install chromium
```

### Development Environment

We recommend using:
- **VS Code** with Rust Analyzer extension
- **Xcode** for iOS/watchOS development
- **Playwright Test for VSCode** extension for debugging tests

## Development Workflow

### Making Changes

1. **Create a feature branch**

```bash
git checkout -b feature/amazing-feature
```

2. **Make your changes**
   - Write code
   - Add/update tests
   - Update documentation

3. **Test your changes**

```bash
# Run Rust tests
cd web-ui
cargo test

# Run E2E tests
cd ../tests
npm test

# Check code format
cargo fmt -- --check
cargo clippy -- -D warnings
```

4. **Commit your changes**

```bash
git add .
git commit -m "feat: add amazing feature"
```

5. **Push to your fork**

```bash
git push origin feature/amazing-feature
```

6. **Open a Pull Request**

## Testing Guidelines

### Test Coverage

All new features should include tests:

- **Unit tests** in Rust (inline in source files)
- **E2E tests** using Playwright (in `tests/` directory)
- **Integration tests** for component interactions

### Writing Tests

#### Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

#### Playwright E2E Tests

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test('should do something', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Test actions
    await page.click('button');

    // Assertions
    await expect(page.locator('.result')).toBeVisible();
  });
});
```

### Running Tests

```bash
# All tests
cd tests && npm test

# Specific suite
npm run test:web-ui
npm run test:api
npm run test:integration

# With UI
npm run test:ui

# Debug mode
npm run test:debug
```

## Code Style

### Rust Code Style

We follow standard Rust conventions:

```rust
// Use rustfmt for formatting
cargo fmt

// Check with clippy
cargo clippy -- -D warnings

// Naming conventions
const MAX_RETRIES: u32 = 3;
struct VehicleStatus { ... }
fn get_battery_level() -> u8 { ... }

// Documentation
/// Returns the current battery level as a percentage.
///
/// # Examples
///
/// ```
/// let level = get_battery_level();
/// assert!(level <= 100);
/// ```
pub fn get_battery_level() -> u8 {
    // Implementation
}
```

### TypeScript Code Style

```typescript
// Use Prettier for formatting (runs automatically)

// Naming conventions
const MAX_RETRIES = 3;
interface VehicleStatus { ... }
function getBatteryLevel(): number { ... }

// Comments
// Single-line comments for inline explanations

/**
 * Multi-line comments for functions and complex logic
 */
```

### HTML/CSS

- Use semantic HTML5 elements
- Follow BEM naming convention for CSS classes
- Maintain Toyota brand colors and styling
- Ensure accessibility (ARIA labels, semantic structure)

## Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Code style changes (formatting, missing semi-colons, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Performance improvement
- **test**: Adding or updating tests
- **chore**: Changes to build process, dependencies, etc.

### Examples

```bash
feat(web-ui): add 90% custom charge level alert

- Added configurable alert at any percentage 50-100%
- Updated settings UI with slider
- Added tests for new alert functionality

Closes #123
```

```bash
fix(charging): resolve incorrect kWh calculation

The calculation was using wrong multiplier for AC charging.

Fixes #456
```

### Commit Message Guidelines

- Use the imperative mood ("add feature" not "added feature")
- First line should be 50 characters or less
- Reference issues and pull requests liberally
- Use the body to explain what and why vs. how

## Pull Request Process

1. **Update Documentation**
   - Update README.md with any new functionality
   - Update API documentation (openapi-web-ui.yaml)
   - Update ARCHITECTURE.md if needed

2. **Add Tests**
   - Ensure all tests pass: `npm test`
   - Add tests for new features
   - Update tests for bug fixes

3. **Update Changelog**
   - Add entry to CHANGELOG.md (if it exists)
   - Follow Keep a Changelog format

4. **PR Description**
   - Use the PR template
   - Describe what was changed and why
   - Include screenshots for UI changes
   - Link related issues

5. **Code Review**
   - Address review comments
   - Keep discussions focused and professional
   - Push changes as additional commits (don't force push)

6. **Merge**
   - PRs require at least one approval
   - All CI checks must pass
   - Squash and merge preferred for feature branches

## Project Structure

```
spin_myT2ABRP/
├── web-ui/                 # HTMX Web Dashboard (Cargo)
│   ├── src/
│   │   └── lib.rs          # Main HTTP handler
│   ├── static/
│   │   ├── index.html      # Dashboard HTML
│   │   ├── styles.css      # Toyota branding
│   │   └── app.js          # PWA + minimal JS
│   └── Cargo.toml
│
├── components/             # Bazel-built components (optional)
│   ├── gateway/            # API gateway
│   ├── business-logic/     # Business rules
│   ├── data-transform/     # Data transformation
│   └── ...
│
├── ios-app/                # iOS Native App
│   └── MyT2ABRP/
│       ├── ViewModels/     # Business logic
│       └── Views/          # SwiftUI views
│
├── watchos-app/            # Apple Watch App
│
├── tests/                  # Playwright E2E Tests
│   ├── tests/
│   │   ├── web-ui/
│   │   ├── api/
│   │   └── integration/
│   └── playwright.config.ts
│
├── .github/                # GitHub Actions CI/CD
│   └── workflows/
│       └── test.yml
│
├── docs/                   # Additional documentation
│
├── spin.toml               # Spin configuration
├── Dockerfile              # Container deployment
├── openapi-web-ui.yaml     # API specification
├── README.md               # Main documentation
├── ARCHITECTURE.md         # Technical design
├── TROUBLESHOOTING.md      # Common issues
└── CONTRIBUTING.md         # This file
```

## Component Contribution Guidelines

### Web UI (Rust/HTMX)

- Keep handlers focused and single-purpose
- Return HTML fragments for HTMX endpoints
- Use proper error handling with Result types
- Add security headers to responses
- Cache static files appropriately

### iOS/watchOS (Swift)

- Follow MVVM architecture
- Use SwiftUI for all views
- Implement proper error handling
- Support offline mode
- Test on real devices when possible

### Tests (Playwright)

- Use proper waiting strategies (networkidle, not arbitrary timeouts)
- Test across multiple browsers when relevant
- Keep tests independent and isolated
- Use descriptive test names
- Add comments for complex test logic

## Getting Help

- **Documentation**: Check README.md, ARCHITECTURE.md, and TROUBLESHOOTING.md
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Email**: ralf_beier@me.com

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to MyT2ABRP!** 🚗⚡

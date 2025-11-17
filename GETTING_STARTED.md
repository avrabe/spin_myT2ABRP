# Getting Started with MyT2ABRP Development

Quick start guide for developers who want to run, develop, and contribute to MyT2ABRP.

## Table of Contents

- [Quick Start (5 minutes)](#quick-start-5-minutes)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [First Run](#first-run)
- [Development Workflow](#development-workflow)
- [Common Tasks](#common-tasks)
- [Next Steps](#next-steps)

## Quick Start (5 minutes)

Get the web dashboard running locally:

```bash
# 1. Install Spin CLI
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv spin /usr/local/bin/

# 2. Clone and enter directory
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP

# 3. Add Rust target
rustup target add wasm32-wasip2

# 4. Build and run
spin build && spin up

# 5. Open browser
open http://localhost:3000
```

That's it! The dashboard should be running with demo data.

## Prerequisites

### Required

- **Rust** 1.70 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add wasm32-wasip2
  ```

- **Spin CLI** 2.7.0 or later
  ```bash
  curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
  ```

- **Git**
  ```bash
  # macOS
  xcode-select --install

  # Ubuntu/Debian
  sudo apt-get install git

  # Windows
  # Download from https://git-scm.com/downloads
  ```

### Optional (for full development)

- **Node.js** 20+ (for testing)
  ```bash
  # Using nvm (recommended)
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
  nvm install 20
  nvm use 20
  ```

- **Bazel** 7.0+ (for main API component - optional)
  ```bash
  # macOS
  brew install bazelisk

  # npm
  npm install -g @bazel/bazelisk
  ```

- **Xcode** 15+ (for iOS/watchOS development)
  ```bash
  # Download from Mac App Store or https://developer.apple.com/xcode/
  ```

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP
```

### 2. Set Up Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit .env (optional - defaults work for development)
nano .env
```

**Key environment variables:**

```bash
# Development defaults (already set in .env.example)
JWT_SECRET=dev-secret-change-in-production
HMAC_KEY=dev-hmac-key-change-in-production
CORS_ORIGIN=*
PORT=3000
LOG_LEVEL=info
```

### 3. Install Dependencies

```bash
# Rust WASM target
rustup target add wasm32-wasip2

# Test dependencies (optional)
cd tests
npm install
npx playwright install chromium
cd ..
```

## First Run

### Build the Web UI

```bash
# Build web-ui component
spin build

# Should see:
# Building component web-ui with `cd web-ui && cargo build --target wasm32-wasip2 --release`
# Finished building all Spin components
```

**Troubleshooting build issues:** See [TROUBLESHOOTING.md](TROUBLESHOOTING.md#build-issues)

### Start the Server

```bash
spin up

# Should see:
# Serving http://127.0.0.1:3000
# Available Routes:
#   web-ui: http://127.0.0.1:3000
#   web-ui: http://127.0.0.1:3000/styles.css
#   web-ui: http://127.0.0.1:3000/app.js
#   web-ui: http://127.0.0.1:3000/api (wildcard)
```

### Open the Dashboard

```bash
# macOS
open http://localhost:3000

# Linux
xdg-open http://localhost:3000

# Windows
start http://localhost:3000

# Or just navigate to http://localhost:3000 in your browser
```

You should see the Toyota-branded dashboard with:
- Vehicle status card (demo data)
- Charging controls
- Battery health metrics
- Quick actions

## Development Workflow

### Making Changes

#### 1. Edit Source Files

The main files you'll work with:

```
web-ui/
├── src/
│   └── lib.rs              # Main HTTP handler - edit API logic here
├── static/
│   ├── index.html          # Dashboard HTML
│   ├── styles.css          # Styling
│   └── app.js              # JavaScript
└── Cargo.toml              # Dependencies
```

#### 2. Rebuild and Test

```bash
# Rebuild (fast - only changed files)
spin build

# Restart server
# Ctrl+C to stop, then:
spin up

# Or use watch mode (rebuilds automatically)
spin watch
```

#### 3. View Changes

Refresh your browser (http://localhost:3000) to see changes.

**Tip:** Use browser dev tools (F12) to debug:
- Console for JavaScript errors
- Network tab for API calls
- Elements to inspect HTMXrequest/responses

### Hot Reload Tips

For faster development:

```bash
# Terminal 1: Watch mode (auto-rebuilds on file changes)
spin watch

# Terminal 2: Run tests on changes
cd tests
npm run test:ui  # Interactive test runner
```

## Common Tasks

### Adding a New API Endpoint

1. **Edit `web-ui/src/lib.rs`**:

```rust
fn handle_request(req: Request) -> Result<Response> {
    let path = req.path_and_query().unwrap_or("/");
    let method = req.method();

    match (method, path) {
        // Add your new endpoint here
        (Method::Get, "/api/your-endpoint") => {
            Ok(ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(r#"{"message": "Hello!"}"#)
                .build())
        }

        // ... existing endpoints ...
    }
}
```

2. **Rebuild and test**:

```bash
spin build && spin up

# In another terminal
curl http://localhost:3000/api/your-endpoint
```

3. **Add tests** in `tests/tests/api/endpoints.spec.ts`:

```typescript
test('GET /api/your-endpoint should return data', async ({ request }) => {
  const response = await request.get('/api/your-endpoint');
  expect(response.ok()).toBeTruthy();

  const data = await response.json();
  expect(data.message).toBe('Hello!');
});
```

### Updating the UI

1. **Edit `web-ui/static/index.html`**:

```html
<!-- Add new card to dashboard -->
<div class="card my-card">
  <h2>My New Feature</h2>
  <div hx-get="/api/my-data"
       hx-trigger="load, every 10s"
       hx-swap="innerHTML">
    Loading...
  </div>
</div>
```

2. **Add corresponding API endpoint** (see above)

3. **Style in `web-ui/static/styles.css`**:

```css
.my-card {
    background: var(--toyota-white);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
}
```

4. **Test manually** or add E2E test

### Running Tests

```bash
# Run all tests
cd tests
npm test

# Run specific test file
npm test -- tests/api/endpoints.spec.ts

# Run in headed mode (see browser)
npm run test:headed

# Debug mode (step through)
npm run test:debug

# Interactive UI
npm run test:ui
```

### Code Quality Checks

```bash
# Format Rust code
cd web-ui
cargo fmt

# Lint Rust code
cargo clippy -- -D warnings

# Run Rust tests (if any)
cargo test
```

## Project Structure Quick Reference

```
spin_myT2ABRP/
├── web-ui/                 # 👈 START HERE for web development
│   ├── src/lib.rs          # Main API handler
│   └── static/             # HTML, CSS, JS
│
├── components/             # Optional Bazel components
│
├── ios-app/                # iOS app (Xcode project)
│
├── watchos-app/            # Watch app
│
├── tests/                  # Playwright E2E tests
│   ├── tests/              # Test suites
│   └── playwright.config.ts
│
├── .github/workflows/      # CI/CD automation
│
├── spin.toml               # Spin app configuration
├── Dockerfile              # Container deployment
├── openapi-web-ui.yaml     # API specification
│
└── *.md                    # Documentation
```

## Understanding the Architecture

### Request Flow

```
Browser Request
    ↓
Spin Runtime (localhost:3000)
    ↓
web-ui component (Rust WASM)
    ↓
lib.rs handle_request()
    ↓
Route matching (/, /api/*, /styles.css, etc.)
    ↓
Handler function
    ↓
Response (HTML fragment, JSON, or static file)
    ↓
Browser (HTMX swaps content if applicable)
```

### HTMX Pattern

The UI uses HTMX for dynamic updates without JavaScript frameworks:

```html
<!-- Element auto-refreshes every 5 seconds -->
<div hx-get="/api/vehicle/status"
     hx-trigger="load, every 5s"
     hx-swap="innerHTML">
  Initial content
</div>
```

**Key concepts:**
- `hx-get`: Fetch from this endpoint
- `hx-trigger`: When to fetch (load, click, every Xs, etc.)
- `hx-swap`: How to update (innerHTML, outerHTML, etc.)
- Server returns HTML fragments (not JSON)

## Next Steps

### For Contributors

1. **Read [CONTRIBUTING.md](CONTRIBUTING.md)** - contribution guidelines
2. **Check [Issues](https://github.com/avrabe/spin_myT2ABRP/issues)** - find something to work on
3. **Join [Discussions](https://github.com/avrabe/spin_myT2ABRP/discussions)** - ask questions

### For Learning More

1. **[ARCHITECTURE.md](ARCHITECTURE.md)** - detailed technical design
2. **[openapi-web-ui.yaml](openapi-web-ui.yaml)** - complete API specification
3. **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - common issues and solutions
4. **[Fermyon Spin Docs](https://developer.fermyon.com/spin)** - Spin framework
5. **[HTMX Docs](https://htmx.org/docs/)** - HTMX library

### Building Other Components

#### iOS App

```bash
open ios-app/MyT2ABRP.xcodeproj

# In Xcode:
# - Select scheme: MyT2ABRP
# - Select destination: iPhone 15 Pro (simulator)
# - Press Cmd+R to build and run
```

#### Docker Deployment

```bash
# Build image
docker build -t myt2abrp .

# Run container
docker run -p 3000:3000 myt2abrp

# With environment file
docker run -p 3000:3000 --env-file .env myt2abrp
```

## Common Issues

### "spin: command not found"

```bash
# Verify installation
which spin

# If not found, reinstall
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv spin /usr/local/bin/
```

### Build fails with "target not found"

```bash
rustup target add wasm32-wasip2
```

### Port 3000 already in use

```bash
# Find and kill process
lsof -i :3000
kill -9 <PID>

# Or use different port
spin up --listen 127.0.0.1:3001
```

### Tests fail to start server

```bash
# Make sure Spin is in PATH
export PATH="$HOME/bin:$PATH"

# Or update playwright.config.ts with full path to spin
```

For more issues, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

## Getting Help

- **Documentation**: Start with README.md
- **Issues**: https://github.com/avrabe/spin_myT2ABRP/issues
- **Discussions**: https://github.com/avrabe/spin_myT2ABRP/discussions
- **Email**: ralf_beier@me.com

## Quick Command Reference

```bash
# Development
spin build              # Build all components
spin up                 # Start server
spin watch              # Auto-rebuild on changes

# Testing
cd tests && npm test    # Run all tests
npm run test:ui         # Interactive test UI
npm run test:debug      # Debug mode

# Code quality
cargo fmt               # Format code
cargo clippy            # Lint code
cargo test              # Run Rust tests

# Deployment
docker build -t myt2abrp .  # Build container
spin deploy             # Deploy to Fermyon Cloud
```

---

**Happy coding! 🚗⚡**

Found an issue with this guide? Please [open an issue](https://github.com/avrabe/spin_myT2ABRP/issues/new) or submit a pull request.

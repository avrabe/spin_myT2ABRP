# MyT2ABRP Quick Start Guide

Get MyT2ABRP up and running in **5 minutes** or less!

## Prerequisites Check

Before starting, ensure you have:

```bash
# Check if installed (should all return versions)
rustc --version    # Rust 1.70+
spin --version     # Spin CLI 2.7.0+
node --version     # Node.js 20+ (optional, for tests)
docker --version   # Docker (optional, for deployment)
```

**Don't have them?** Run: `make install` (it will guide you)

## Option 1: Ultra-Fast Start (< 2 minutes)

```bash
# 1. Clone
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP

# 2. Setup
cp .env.example .env

# 3. Run
make run

# ✅ Access at http://localhost:3000
```

**That's it!** You're running MyT2ABRP locally.

## Option 2: Production Deployment (< 5 minutes)

```bash
# 1. Clone and setup
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP
cp .env.example .env.prod

# 2. Edit secrets (IMPORTANT!)
nano .env.prod
# Change JWT_SECRET and HMAC_KEY to random values:
#   JWT_SECRET=$(openssl rand -base64 32)
#   HMAC_KEY=$(openssl rand -hex 32)

# 3. Deploy with monitoring
make deploy-prod

# ✅ Access:
#   - App: http://localhost (or your domain)
#   - Grafana: http://localhost:3001 (admin/admin)
#   - Prometheus: http://localhost:9090
```

**You now have a production-ready stack with monitoring!**

## Option 3: Step-by-Step Guide

### Step 1: Install Prerequisites

#### Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2
```

#### Spin CLI
```bash
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv spin /usr/local/bin/
```

#### Node.js (for testing)
```bash
# macOS
brew install node

# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### Step 2: Clone and Setup

```bash
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP

# Auto-setup (checks prerequisites, installs dependencies)
make install
```

### Step 3: Configure

```bash
# Edit .env with your settings
nano .env

# Key settings:
# - JWT_SECRET: Strong random secret (use: openssl rand -base64 32)
# - HMAC_KEY: Strong random key (use: openssl rand -hex 32)
# - CORS_ORIGIN: Your domain (or http://localhost:3000 for dev)
```

### Step 4: Build

```bash
make build

# This compiles:
# - Rust web-ui to WASM (optimized for size and speed)
# - All Spin components
```

### Step 5: Run

```bash
make run

# Or for more control:
spin up --listen 0.0.0.0:3000
```

### Step 6: Test

```bash
# Run automated tests
make test

# Run load tests
make loadtest

# Run security audit
make audit
```

## Common Commands

```bash
# Development
make build          # Build everything
make run            # Build and run
make test           # Run all tests
make clean          # Clean build artifacts
make format         # Format code
make lint           # Run linters

# Deployment
make deploy-local   # Deploy locally
make deploy-docker  # Deploy with Docker
make deploy-prod    # Deploy production stack
make deploy-fermyon # Deploy to Fermyon Cloud

# Monitoring
make monitor-install    # Install monitoring stack
make monitor-status     # Check monitoring status

# Utilities
make help           # Show all commands
make version        # Show versions
make status         # Show project status
```

## Verification Checklist

After starting, verify everything works:

- [ ] **Web UI loads**: http://localhost:3000
- [ ] **Health check passes**: `curl http://localhost:3000/health`
- [ ] **Metrics available**: `curl http://localhost:3000/api/metrics`
- [ ] **Vehicle status loads**: Check dashboard
- [ ] **No errors in logs**: `make logs` (if using Docker)

## Troubleshooting

### Issue: "spin: command not found"

```bash
# Install Spin CLI
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv spin /usr/local/bin/
spin --version
```

### Issue: "rustc: command not found"

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add wasm32-wasip2
```

### Issue: Build fails with "wasm32-wasip2 not found"

```bash
rustup target add wasm32-wasip2
```

### Issue: Port 3000 already in use

```bash
# Find what's using it
lsof -ti:3000

# Kill it
kill -9 $(lsof -ti:3000)

# Or use different port
spin up --listen 0.0.0.0:3001
```

### Issue: "Permission denied" when running scripts

```bash
# Make scripts executable
chmod +x *.sh
```

### Issue: Docker deployment fails

```bash
# Check Docker is running
docker ps

# Start Docker
# macOS: Open Docker Desktop
# Linux: sudo systemctl start docker

# Retry deployment
make deploy-docker
```

## What's Next?

### 🚀 For Developers

1. **Read the docs**: `./DOCS_INDEX.md`
2. **Understand architecture**: `./ARCHITECTURE.md`
3. **Contributing guide**: `./CONTRIBUTING.md`
4. **Run tests**: `make test`
5. **Watch mode**: `make watch` (auto-rebuild on changes)

### 🌐 For Deployment

1. **Deployment guide**: `./DEPLOYMENT.md`
2. **Production checklist**: See README.md
3. **Monitoring setup**: `make monitor-install`
4. **Security audit**: `make audit`
5. **Backup setup**: `./backup.sh backup`

### 🔧 For Operations

1. **Monitoring**: `make monitor-install`
2. **Backups**: `./backup.sh backup`
3. **Load testing**: `make loadtest`
4. **Performance guide**: `./PERFORMANCE.md`
5. **Troubleshooting**: `./TROUBLESHOOTING.md`

## Quick Reference

### File Locations

```
spin_myT2ABRP/
├── web-ui/           # Rust web component
├── tests/            # Playwright E2E tests
├── .env.example      # Configuration template
├── spin.toml         # Spin configuration
├── Makefile          # All commands
└── *.sh              # Automation scripts
```

### Important URLs (when running)

- **App**: http://localhost:3000
- **Health**: http://localhost:3000/health
- **Metrics**: http://localhost:3000/api/metrics
- **Grafana**: http://localhost:3001 (if monitoring stack installed)
- **Prometheus**: http://localhost:9090

### Environment Variables

```bash
# Required
JWT_SECRET=your-secret-here              # JWT signing key
HMAC_KEY=your-key-here                   # HMAC key
CORS_ORIGIN=http://localhost:3000        # Allowed origins

# Optional
RUST_LOG=info                            # Log level
SPIN_HTTP_LISTEN_ADDR=0.0.0.0:3000      # Bind address
```

## Performance Tips

- **WASM binary**: ~3.2MB (optimized for size)
- **Cold start**: < 10ms
- **Response time**: < 10ms (P50)
- **Throughput**: 1,200+ req/s (single instance)

See `./PERFORMANCE.md` for optimization guide.

## Security Notes

- **Change default secrets!** Never use .env.example values in production
- **Use HTTPS** in production (see nginx.prod.conf)
- **Enable rate limiting** (configured in nginx.prod.conf)
- **Run security audit**: `make audit`
- **Keep dependencies updated**: `make update-deps`

## Support

- **Documentation**: `./DOCS_INDEX.md`
- **Issues**: https://github.com/avrabe/spin_myT2ABRP/issues
- **Discussions**: https://github.com/avrabe/spin_myT2ABRP/discussions
- **Email**: ralf_beier@me.com

## Success Indicators

You've successfully set up MyT2ABRP when:

1. ✅ Web UI loads at http://localhost:3000
2. ✅ Health check returns `{"status":"healthy"}`
3. ✅ Dashboard shows vehicle status
4. ✅ Tests pass: `make test`
5. ✅ No errors in console/logs

---

**🎉 Congratulations!** You're now running MyT2ABRP!

For more details, see:
- Full documentation: `./DOCS_INDEX.md`
- Architecture: `./ARCHITECTURE.md`
- Deployment options: `./DEPLOYMENT.md`
- Contributing: `./CONTRIBUTING.md`

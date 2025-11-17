# Docker Development Guide - Toyota MyT2ABRP

## Quick Start

```bash
# Start development environment
docker-compose up dev

# Run tests
docker-compose up test

# Start production-like environment
docker-compose up prod

# Full stack with monitoring
docker-compose up dev prometheus grafana jaeger
```

## Services

### Development (`dev`)
- Hot-reload enabled with `cargo-watch`
- Full development toolchain
- Spin running on port 3000
- Volume-mounted source code for live editing

**Usage:**
```bash
docker-compose up dev

# Access API
curl http://localhost:3000/health

# Logs
docker-compose logs -f dev
```

### Production (`prod`)
- Optimized WASM builds
- Minimal runtime image
- Health checks configured
- Auto-restart on failure

**Usage:**
```bash
docker-compose up -d prod

# Check health
docker-compose ps
curl http://localhost:3000/health

# View logs
docker-compose logs -f prod
```

### Testing (`test`)
- Runs complete test suite
- Includes E2E tests with Playwright
- Component validation
- Performance benchmarks

**Usage:**
```bash
docker-compose up test

# View test results
docker-compose logs test
```

### Monitoring Stack

#### Prometheus (`:9090`)
- Metrics collection
- Alert management
- Query interface

```bash
# Access Prometheus UI
open http://localhost:9090

# Query example
rate(http_requests_total[5m])
```

#### Grafana (`:3001`)
- Metrics visualization
- Pre-configured dashboards
- Alert visualization

```bash
# Access Grafana
open http://localhost:3001
# Login: admin / admin
```

#### Jaeger (`:16686`)
- Distributed tracing
- Performance analysis
- Request flow visualization

```bash
# Access Jaeger UI
open http://localhost:16686
```

## Development Workflows

### Local Development with Hot Reload

```bash
# Start dev environment
docker-compose up dev

# In another terminal, make changes to code
# Changes are automatically rebuilt

# View logs
docker-compose logs -f dev
```

### Running Tests

```bash
# All tests
docker-compose up test

# Specific test suite
docker-compose run test npm test --prefix tests/e2e

# Unit tests only
docker-compose run test cargo test
```

### Building for Production

```bash
# Build optimized image
docker-compose build prod

# Run production container
docker-compose up -d prod

# Verify
curl http://localhost:3000/health
```

## Multi-Stage Build Explanation

The Dockerfile uses multi-stage builds for efficiency:

1. **base**: Core dependencies (Rust, Node.js, tools)
2. **dependencies**: Cached Cargo dependencies
3. **development**: Full dev environment with tools
4. **builder**: Production build stage
5. **production**: Minimal WASM artifacts only
6. **runtime**: Slim runtime with Spin CLI

**Benefits:**
- Fast rebuilds (cached layers)
- Small production images
- Consistent environments
- Optimized for CI/CD

## Environment Variables

### Development
```bash
RUST_LOG=debug              # Logging level
RUST_BACKTRACE=1            # Enable backtraces
SPIN_VERSION=v3.0.0         # Spin CLI version
```

### Production
```bash
RUST_LOG=info               # Less verbose logging
PORT=3000                   # Server port
```

### Testing
```bash
BASE_URL=http://dev:3000    # Test target URL
CI=true                     # CI mode
```

## Volume Mounts

### Development Volumes
- `.:/app` - Source code (live reload)
- `cargo-cache` - Cargo registry cache
- `target-cache` - Build artifacts cache

### Persistent Volumes
- `prometheus-data` - Prometheus metrics data
- `grafana-data` - Grafana dashboards and config

## Networking

All services are on the same Docker network:

```
dev:3000        → API
prometheus:9090 → Metrics UI
grafana:3001    → Visualization
jaeger:16686    → Tracing UI
```

Services can communicate by service name:
```yaml
# Prometheus scrape config
- targets: ['dev:3000']
```

## Performance Optimization

### Build Cache

```bash
# Pre-populate cache
docker-compose build --no-cache base

# Use cache for fast builds
docker-compose build dev
```

### Resource Limits

```yaml
# Add to docker-compose.yml
services:
  dev:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
```

### Build Optimization

```bash
# Use BuildKit for faster builds
DOCKER_BUILDKIT=1 docker-compose build

# Parallel builds
docker-compose build --parallel
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker-compose logs dev

# Inspect container
docker-compose ps
docker inspect toyota-myt2abrp_dev_1

# Rebuild from scratch
docker-compose down -v
docker-compose build --no-cache
docker-compose up
```

### Port Already in Use

```bash
# Change port in docker-compose.yml
ports:
  - "3001:3000"  # Use port 3001 instead

# Or kill existing process
lsof -ti:3000 | xargs kill -9
```

### Volume Permission Issues

```bash
# Fix permissions
sudo chown -R $USER:$USER .

# Or run as current user
docker-compose run --user $(id -u):$(id -g) dev
```

### Out of Disk Space

```bash
# Clean up
docker system prune -a --volumes

# Remove specific volumes
docker volume rm toyota-myt2abrp_target-cache
docker volume rm toyota-myt2abrp_cargo-cache
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Docker Build

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build images
        run: docker-compose build

      - name: Run tests
        run: docker-compose up test

      - name: Push to registry
        run: |
          docker tag toyota-myt2abrp_prod ghcr.io/avrabe/toyota-myt2abrp:latest
          docker push ghcr.io/avrabe/toyota-myt2abrp:latest
```

### GitLab CI

```yaml
build:
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker-compose build
    - docker-compose up test
```

## Production Deployment

### Docker Swarm

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml toyota-myt2abrp

# Scale services
docker service scale toyota-myt2abrp_prod=3
```

### Kubernetes

```bash
# Generate Kubernetes manifests
kompose convert -f docker-compose.yml

# Apply to cluster
kubectl apply -f .
```

## Security Best Practices

1. **Non-root user** (add to Dockerfile):
   ```dockerfile
   RUN useradd -m -u 1000 app
   USER app
   ```

2. **Scan for vulnerabilities**:
   ```bash
   docker scan toyota-myt2abrp_prod
   ```

3. **Use specific base image versions**:
   ```dockerfile
   FROM rust:1.91.1-slim
   # Not: FROM rust:latest
   ```

4. **Minimize attack surface**:
   - Multi-stage builds
   - Minimal production images
   - Remove unnecessary tools

## Monitoring Setup

### Prometheus Configuration

Create `monitoring/prometheus.yml`:
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'toyota-myt2abrp'
    static_configs:
      - targets: ['dev:3000']
```

### Grafana Dashboards

Dashboards auto-load from `monitoring/grafana/dashboards/`.

Create `dashboard.json` with:
- Request rate graphs
- Error rate graphs
- Latency percentiles
- Component health

## Next Steps

1. Customize `docker-compose.yml` for your environment
2. Add environment-specific configs
3. Set up production registry
4. Configure CI/CD pipeline
5. Add health checks and monitoring
6. Implement backup strategy
7. Document deployment procedures

---

**Last Updated**: 2025-11-16
**Docker Version**: 24.0+
**Docker Compose Version**: 2.0+

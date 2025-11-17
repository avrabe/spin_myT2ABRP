# Environment Variables Configuration Guide

Complete reference for all environment variables used by MyT2ABRP.

## Required Variables

### JWT_SECRET
**Required**: Yes
**Description**: Secret key for signing JWT tokens
**Type**: String (base64 encoded recommended)
**Default**: None (must be set)
**Example**:
```bash
JWT_SECRET=$(openssl rand -base64 32)
# Example output: 8vX2kR9mP4nL7sT1wQ6eY3uI5oA0bC8dF2gH4jK6mN9p
```

**Security**:
- MUST be different in each environment
- NEVER commit to version control
- Minimum 32 characters
- Use cryptographically secure random generation

---

### HMAC_KEY
**Required**: Yes
**Description**: HMAC key for username hashing
**Type**: String (hex encoded recommended)
**Default**: None (must be set)
**Example**:
```bash
HMAC_KEY=$(openssl rand -hex 32)
# Example output: a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890
```

**Security**:
- MUST be different in each environment
- NEVER commit to version control
- Minimum 32 bytes (64 hex characters)
- Use cryptographically secure random generation

---

### CORS_ORIGIN
**Required**: Yes
**Description**: Allowed CORS origins (comma-separated for multiple)
**Type**: String (URL)
**Default**: `*` (not recommended for production)
**Examples**:
```bash
# Development
CORS_ORIGIN=http://localhost:3000

# Production (single origin)
CORS_ORIGIN=https://myt2abrp.example.com

# Production (multiple origins)
CORS_ORIGIN=https://myt2abrp.example.com,https://app.example.com
```

**Security**:
- NEVER use `*` in production
- Use specific origins only
- Include protocol (http:// or https://)
- No trailing slashes

---

## Optional Variables

### RUST_LOG
**Required**: No
**Description**: Logging level for Rust components
**Type**: String (log level)
**Default**: `info`
**Valid Values**: `error`, `warn`, `info`, `debug`, `trace`
**Example**:
```bash
# Production
RUST_LOG=warn

# Development
RUST_LOG=debug

# Per-module logging
RUST_LOG=info,web_ui=debug,myt2abrp=trace
```

**Performance Impact**: `trace` and `debug` can significantly impact performance

---

### SPIN_HTTP_LISTEN_ADDR
**Required**: No
**Description**: Address and port for Spin to listen on
**Type**: String (IP:PORT)
**Default**: `127.0.0.1:3000`
**Examples**:
```bash
# Listen on all interfaces
SPIN_HTTP_LISTEN_ADDR=0.0.0.0:3000

# Custom port
SPIN_HTTP_LISTEN_ADDR=0.0.0.0:8080

# Specific interface
SPIN_HTTP_LISTEN_ADDR=192.168.1.100:3000
```

**Security**: Use `127.0.0.1` for local-only access, `0.0.0.0` when behind reverse proxy

---

### TOYOTA_API_URL
**Required**: No (if using mock data)
**Description**: URL for Toyota API endpoint
**Type**: String (URL)
**Default**: None (mock data used)
**Example**:
```bash
TOYOTA_API_URL=https://api.toyota-europe.com/
```

---

### DATABASE_URL
**Required**: No
**Description**: PostgreSQL database connection string
**Type**: String (connection URL)
**Default**: None (optional feature)
**Example**:
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/myt2abrp
```

**Format**: `postgresql://[user[:password]@][host][:port][/database][?parameters]`

---

### REDIS_URL
**Required**: No
**Description**: Redis connection URL for caching
**Type**: String (connection URL)
**Default**: None (caching disabled)
**Examples**:
```bash
# Local Redis
REDIS_URL=redis://localhost:6379

# With password
REDIS_URL=redis://:password@localhost:6379

# Redis Cluster
REDIS_URL=redis://localhost:6379,localhost:6380,localhost:6381
```

---

### GRAFANA_ADMIN_PASSWORD
**Required**: No (for monitoring stack)
**Description**: Grafana admin password
**Type**: String
**Default**: `admin`
**Example**:
```bash
GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 16)
```

**Security**: Change default password immediately

---

### REDIS_PASSWORD
**Required**: No (for monitoring stack)
**Description**: Redis password
**Type**: String
**Default**: None
**Example**:
```bash
REDIS_PASSWORD=$(openssl rand -base64 16)
```

---

### ALERT_EMAIL
**Required**: No
**Description**: Email address for AlertManager notifications
**Type**: String (email)
**Default**: None
**Example**:
```bash
ALERT_EMAIL=alerts@example.com
```

---

### ALERT_SLACK_WEBHOOK
**Required**: No
**Description**: Slack webhook URL for alerts
**Type**: String (URL)
**Default**: None
**Example**:
```bash
ALERT_SLACK_WEBHOOK=https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXX
```

---

## Environment-Specific Configurations

### Development (.env)
```bash
# Required
JWT_SECRET=dev-secret-change-in-production
HMAC_KEY=dev-hmac-key-change-in-production
CORS_ORIGIN=http://localhost:3000

# Optional
RUST_LOG=debug
SPIN_HTTP_LISTEN_ADDR=127.0.0.1:3000
```

### Production (.env.prod)
```bash
# Required - MUST be unique and secure
JWT_SECRET=<generated-with-openssl-rand-base64-32>
HMAC_KEY=<generated-with-openssl-rand-hex-32>
CORS_ORIGIN=https://yourdomain.com

# Optional
RUST_LOG=warn
SPIN_HTTP_LISTEN_ADDR=0.0.0.0:3000
DATABASE_URL=postgresql://user:pass@db:5432/myt2abrp
REDIS_URL=redis://:password@redis:6379
GRAFANA_ADMIN_PASSWORD=<secure-password>
REDIS_PASSWORD=<secure-password>
ALERT_EMAIL=alerts@yourdomain.com
```

### Testing (.env.test)
```bash
# Required
JWT_SECRET=test-secret
HMAC_KEY=test-hmac-key
CORS_ORIGIN=http://localhost:3000

# Optional
RUST_LOG=info
SPIN_HTTP_LISTEN_ADDR=127.0.0.1:3000
```

---

## Security Best Practices

### 1. Never Commit Secrets
```bash
# ✅ Good - Generate on deployment
JWT_SECRET=$(openssl rand -base64 32)

# ❌ Bad - Hardcoded
JWT_SECRET=my-secret-key
```

### 2. Use Different Secrets Per Environment
```bash
# Development
JWT_SECRET=dev-secret-12345

# Production
JWT_SECRET=$(openssl rand -base64 32)  # Different!
```

### 3. Rotate Secrets Regularly
- Production secrets: Every 90 days
- Development secrets: Every release
- Compromised secrets: Immediately

### 4. Use Strong Random Generation
```bash
# ✅ Good
openssl rand -base64 32
openssl rand -hex 32

# ❌ Bad
echo "mysecret" | base64
```

### 5. Restrict CORS Origins
```bash
# ✅ Good (production)
CORS_ORIGIN=https://yourdomain.com

# ❌ Bad (production)
CORS_ORIGIN=*
```

---

## Validation

### Check Required Variables
```bash
./security-audit.sh
```

### Test Configuration
```bash
# Start application
make run

# Check health
curl http://localhost:3000/health

# Verify CORS
curl -H "Origin: https://yourdomain.com" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     http://localhost:3000/api/vehicle/status
```

---

## Troubleshooting

### "JWT_SECRET not set"
**Solution**: Add `JWT_SECRET` to `.env` file

### "CORS error in browser"
**Solution**: Ensure `CORS_ORIGIN` matches your frontend URL exactly

### "Cannot connect to database"
**Solution**: Check `DATABASE_URL` format and database is running

### "Redis connection failed"
**Solution**: Verify `REDIS_URL` and Redis server is accessible

---

## Environment Variable Precedence

1. System environment variables (highest priority)
2. `.env.local` (git-ignored, for local overrides)
3. `.env.{environment}` (e.g., `.env.prod`, `.env.test`)
4. `.env` (default values)
5. Application defaults (lowest priority)

---

## Migration Guide

### From Version 0.x to 1.0

**Breaking Changes**:
- `CORS_ALLOWED_ORIGINS` renamed to `CORS_ORIGIN`
- `JWT_KEY` renamed to `JWT_SECRET`

**Migration**:
```bash
# Update .env file
sed -i 's/CORS_ALLOWED_ORIGINS/CORS_ORIGIN/g' .env
sed -i 's/JWT_KEY/JWT_SECRET/g' .env
```

---

## Quick Reference

### Generate All Secrets
```bash
echo "JWT_SECRET=$(openssl rand -base64 32)"
echo "HMAC_KEY=$(openssl rand -hex 32)"
echo "GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 16)"
echo "REDIS_PASSWORD=$(openssl rand -base64 16)"
```

### Minimal .env
```bash
JWT_SECRET=$(openssl rand -base64 32)
HMAC_KEY=$(openssl rand -hex 32)
CORS_ORIGIN=http://localhost:3000
```

### Production .env
```bash
JWT_SECRET=$(openssl rand -base64 32)
HMAC_KEY=$(openssl rand -hex 32)
CORS_ORIGIN=https://yourdomain.com
RUST_LOG=warn
SPIN_HTTP_LISTEN_ADDR=0.0.0.0:3000
DATABASE_URL=postgresql://user:pass@db:5432/myt2abrp
REDIS_URL=redis://:password@redis:6379
GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 16)
REDIS_PASSWORD=$(openssl rand -base64 16)
ALERT_EMAIL=alerts@yourdomain.com
```

---

## Support

For questions about environment variables:
- See [QUICKSTART.md](./QUICKSTART.md) for setup
- See [DEPLOYMENT.md](./DEPLOYMENT.md) for production configuration
- See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for common issues
- Report issues: https://github.com/avrabe/spin_myT2ABRP/issues

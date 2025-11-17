# MyT2ABRP Deployment Guide

Complete guide for deploying MyT2ABRP to production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Deployment](#local-deployment)
- [Docker Deployment](#docker-deployment)
- [Fermyon Cloud Deployment](#fermyon-cloud-deployment)
- [Self-Hosted Deployment](#self-hosted-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Environment Configuration](#environment-configuration)
- [Security Hardening](#security-hardening)
- [Monitoring & Logging](#monitoring--logging)
- [Backup & Recovery](#backup--recovery)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Tools

- **Spin CLI** 2.7.0+
- **Rust** 1.70+
- **Docker** (for containerized deployment)
- **kubectl** (for Kubernetes deployment)

### Required Accounts

- **Fermyon Cloud** account (for cloud deployment)
- **Domain name** and SSL certificate (for production)
- **Cloud provider** account (AWS/GCP/Azure for self-hosted)

## Local Deployment

### Development Server

```bash
# Clone repository
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP

# Install Rust target
rustup target add wasm32-wasip2

# Build and run
spin build && spin up

# Server starts at http://localhost:3000
```

### Production Build

```bash
# Build with optimizations
cd web-ui
cargo build --target wasm32-wasip2 --release

# Verify build
ls -lh target/wasm32-wasip2/release/web_ui.wasm

# Run production server
cd ..
spin up --listen 0.0.0.0:3000
```

## Docker Deployment

### Build Docker Image

```bash
# Build image
docker build -t myt2abrp:latest .

# Verify image
docker images | grep myt2abrp
```

### Run Container

```bash
# Create environment file
cat > .env.prod <<EOF
JWT_SECRET=$(openssl rand -base64 32)
HMAC_KEY=$(openssl rand -hex 32)
CORS_ORIGIN=https://your-domain.com
VIN=YOUR_VIN_HERE
PORT=3000
LOG_LEVEL=info
EOF

# Run container
docker run -d \
  --name myt2abrp \
  -p 3000:3000 \
  --env-file .env.prod \
  --restart unless-stopped \
  myt2abrp:latest

# Check logs
docker logs -f myt2abrp

# Check health
curl http://localhost:3000/health
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  myt2abrp:
    build: .
    ports:
      - "3000:3000"
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - HMAC_KEY=${HMAC_KEY}
      - CORS_ORIGIN=${CORS_ORIGIN}
      - VIN=${VIN}
      - PORT=3000
      - LOG_LEVEL=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - myt2abrp-network

  # Optional: Nginx reverse proxy
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - myt2abrp
    networks:
      - myt2abrp-network

networks:
  myt2abrp-network:
    driver: bridge
```

Deploy:

```bash
# Generate secrets
echo "JWT_SECRET=$(openssl rand -base64 32)" > .env.prod
echo "HMAC_KEY=$(openssl rand -hex 32)" >> .env.prod
echo "CORS_ORIGIN=https://your-domain.com" >> .env.prod

# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Fermyon Cloud Deployment

### Prerequisites

```bash
# Install Spin CLI
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

# Login to Fermyon Cloud
spin login
```

### Deploy Application

```bash
# Build application
spin build

# Deploy to Fermyon Cloud
spin deploy

# Output will show deployed URL:
# Deployed myT2ABRP version 0.1.0+r1234567
# Available Routes:
#   myt2abrp: https://myt2abrp-xyz.fermyon.app
```

### Configure Custom Domain

```bash
# Add custom domain
spin cloud domain add your-domain.com

# Configure DNS (add CNAME record):
# your-domain.com -> myt2abrp-xyz.fermyon.app
```

### Set Environment Variables

```bash
# Set variables
spin cloud variables set JWT_SECRET "$(openssl rand -base64 32)"
spin cloud variables set HMAC_KEY "$(openssl rand -hex 32)"
spin cloud variables set CORS_ORIGIN "https://your-domain.com"

# List variables
spin cloud variables list
```

### Monitor Deployment

```bash
# View logs
spin cloud logs -f

# Check status
spin cloud status

# View metrics
spin cloud metrics
```

## Self-Hosted Deployment

### AWS EC2

#### 1. Launch EC2 Instance

```bash
# Launch Ubuntu 22.04 instance
aws ec2 run-instances \
  --image-id ami-0c55b159cbfafe1f0 \
  --instance-type t3.small \
  --key-name your-key \
  --security-group-ids sg-xxx \
  --subnet-id subnet-xxx \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=MyT2ABRP}]'
```

#### 2. Install Dependencies

```bash
# SSH to instance
ssh -i your-key.pem ubuntu@<instance-ip>

# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker ubuntu

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

#### 3. Deploy Application

```bash
# Clone repository
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP

# Configure environment
cp .env.example .env
nano .env  # Set production values

# Start application
docker-compose up -d

# Configure firewall
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### Digital Ocean

#### Using Droplet

```bash
# Create droplet (via web UI or CLI)
doctl compute droplet create myt2abrp \
  --size s-1vcpu-1gb \
  --image ubuntu-22-04-x64 \
  --region nyc3 \
  --ssh-keys your-key-id

# SSH and install
ssh root@<droplet-ip>

# Follow same installation steps as EC2
```

#### Using App Platform

Create `app.yaml`:

```yaml
name: myt2abrp
services:
  - name: web
    dockerfile_path: Dockerfile
    source_dir: /
    github:
      repo: avrabe/spin_myT2ABRP
      branch: main
      deploy_on_push: true
    envs:
      - key: JWT_SECRET
        scope: RUN_TIME
        type: SECRET
      - key: HMAC_KEY
        scope: RUN_TIME
        type: SECRET
      - key: CORS_ORIGIN
        value: https://myt2abrp.your-domain.com
    health_check:
      http_path: /health
    http_port: 3000
    instance_count: 1
    instance_size_slug: basic-xxs
```

Deploy:

```bash
doctl apps create --spec app.yaml
```

## Kubernetes Deployment

### Prerequisites

```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Configure cluster access
kubectl config use-context your-cluster
```

### Create Kubernetes Resources

#### 1. Secret Configuration

```bash
# Create secrets
kubectl create secret generic myt2abrp-secrets \
  --from-literal=jwt-secret=$(openssl rand -base64 32) \
  --from-literal=hmac-key=$(openssl rand -hex 32) \
  --namespace=default
```

#### 2. Deployment Manifest

Create `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myt2abrp
  labels:
    app: myt2abrp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myt2abrp
  template:
    metadata:
      labels:
        app: myt2abrp
    spec:
      containers:
      - name: myt2abrp
        image: your-registry/myt2abrp:latest
        ports:
        - containerPort: 3000
          protocol: TCP
        env:
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: myt2abrp-secrets
              key: jwt-secret
        - name: HMAC_KEY
          valueFrom:
            secretKeyRef:
              name: myt2abrp-secrets
              key: hmac-key
        - name: CORS_ORIGIN
          value: "https://your-domain.com"
        - name: PORT
          value: "3000"
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: myt2abrp-service
spec:
  selector:
    app: myt2abrp
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: myt2abrp-ingress
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - your-domain.com
    secretName: myt2abrp-tls
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: myt2abrp-service
            port:
              number: 80
```

#### 3. Deploy

```bash
# Apply manifests
kubectl apply -f k8s/deployment.yaml

# Check status
kubectl get pods
kubectl get services
kubectl get ingress

# View logs
kubectl logs -f deployment/myt2abrp

# Scale deployment
kubectl scale deployment myt2abrp --replicas=5
```

## Environment Configuration

### Production Environment Variables

```bash
# Required
JWT_SECRET="<strong-random-secret>"
HMAC_KEY="<strong-random-key>"
CORS_ORIGIN="https://your-domain.com"

# Optional
VIN="YOUR_VIN"
PORT="3000"
LOG_LEVEL="info"  # debug, info, warn, error
RATE_LIMIT_ENABLED="true"
CACHE_ENABLED="true"
```

### Generate Secure Secrets

```bash
# JWT Secret (256-bit)
openssl rand -base64 32

# HMAC Key (256-bit hex)
openssl rand -hex 32

# Strong password (if needed)
openssl rand -base64 24
```

## Security Hardening

### SSL/TLS Configuration

#### Using Let's Encrypt

```bash
# Install certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal
sudo certbot renew --dry-run
```

#### Nginx Configuration

Create `nginx.conf`:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check endpoint (no auth required)
    location /health {
        proxy_pass http://localhost:3000/health;
        access_log off;
    }
}
```

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# Check status
sudo ufw status verbose
```

### Security Checklist

- [ ] Use strong, randomly generated secrets
- [ ] Enable HTTPS with valid SSL certificate
- [ ] Configure CORS properly (not `*` in production)
- [ ] Set up firewall rules
- [ ] Enable security headers
- [ ] Regular security updates
- [ ] Implement rate limiting
- [ ] Use non-root user in container
- [ ] Scan images for vulnerabilities
- [ ] Enable audit logging

## Monitoring & Logging

### Application Metrics

```bash
# View metrics
curl https://your-domain.com/api/metrics

# Response:
# {
#   "uptime_seconds": 3600,
#   "requests_total": 1500,
#   "requests_success": 1495,
#   "requests_error": 5,
#   "cache_hit_rate": 0.85
# }
```

### Health Checks

```bash
# Manual health check
curl https://your-domain.com/health

# Automated monitoring with curl
while true; do
  curl -f https://your-domain.com/health || echo "Health check failed!"
  sleep 60
done
```

### Docker Logging

```bash
# View logs
docker logs -f myt2abrp

# Configure log rotation
docker run -d \
  --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  myt2abrp:latest
```

### Centralized Logging

Using Promtail + Loki + Grafana:

```yaml
# docker-compose.yml (add to existing)
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki-config.yaml:/etc/loki/local-config.yaml

  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/log:/var/log
      - ./promtail-config.yaml:/etc/promtail/config.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

## Backup & Recovery

### Database Backup (Future)

When database is added:

```bash
# Backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
docker exec myt2abrp-db pg_dump -U postgres myt2abrp > backup_$DATE.sql
aws s3 cp backup_$DATE.sql s3://your-bucket/backups/
```

### Configuration Backup

```bash
# Backup environment and configs
tar -czf myt2abrp-config-$(date +%Y%m%d).tar.gz \
  .env \
  nginx.conf \
  docker-compose.yml

# Upload to S3
aws s3 cp myt2abrp-config-*.tar.gz s3://your-bucket/configs/
```

## Troubleshooting

### Application Won't Start

```bash
# Check logs
docker logs myt2abrp

# Check environment variables
docker exec myt2abrp env

# Verify port availability
netstat -tulpn | grep 3000
```

### High Memory Usage

```bash
# Check container stats
docker stats myt2abrp

# Restart container
docker restart myt2abrp

# Adjust memory limits in docker-compose.yml
```

### SSL Certificate Issues

```bash
# Test SSL
openssl s_client -connect your-domain.com:443

# Renew certificate
sudo certbot renew

# Check certificate expiry
echo | openssl s_client -servername your-domain.com -connect your-domain.com:443 2>/dev/null | openssl x509 -noout -dates
```

## Performance Optimization

### CDN Configuration

Use Cloudflare for caching static assets:

1. Add domain to Cloudflare
2. Configure Page Rules:
   - Cache Level: Standard
   - Browser Cache TTL: 4 hours
   - Edge Cache TTL: 2 hours
3. Enable Auto Minify (HTML, CSS, JS)

### Caching Headers

Already configured in code:

```
Cache-Control: public, max-age=3600  # Static files
Cache-Control: no-store, no-cache    # API endpoints
```

## Support

- **Documentation**: https://github.com/avrabe/spin_myT2ABRP
- **Issues**: https://github.com/avrabe/spin_myT2ABRP/issues
- **Email**: ralf_beier@me.com

---

**Last Updated**: 2025-11-17
**Version**: 1.0.0

#!/usr/bin/env bash
#
# Deployment Automation Script for MyT2ABRP
#
# Simplifies deployment to various platforms with pre-flight checks
# and post-deployment verification.
#
# Usage:
#   ./deploy.sh [target] [options]
#
# Targets:
#   local       - Deploy to local environment (spin up)
#   fermyon     - Deploy to Fermyon Cloud
#   docker      - Build and run Docker container
#   prod        - Deploy production stack with monitoring
#   k8s         - Deploy to Kubernetes cluster
#
# Options:
#   --skip-tests    Skip running tests before deployment
#   --skip-build    Skip build step (use existing binaries)
#   --dry-run       Show what would be deployed without deploying

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Options
SKIP_TESTS=false
SKIP_BUILD=false
DRY_RUN=false

# Parse options
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            TARGET="$1"
            shift
            ;;
    esac
done

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

section() {
    echo ""
    echo -e "${BLUE}$1${NC}"
    echo "================================================================"
}

# Pre-flight checks
preflight_checks() {
    section "Pre-flight Checks"

    # Check git status
    if [ -n "$(git status --porcelain)" ]; then
        log_warn "Uncommitted changes detected"
        if [ "$DRY_RUN" = false ]; then
            read -p "Continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_error "Deployment cancelled"
            fi
        fi
    else
        log_success "Git working directory clean"
    fi

    # Check environment file
    if [ ! -f ".env" ]; then
        log_warn ".env file not found"
        if [ -f ".env.example" ]; then
            log_info "Copy .env.example to .env and configure it"
        fi
    else
        log_success ".env file exists"

        # Check for default secrets
        if grep -q "change-this-in-production" .env 2>/dev/null; then
            log_error "Default secrets detected in .env! Update before deploying."
        fi
    fi

    # Run tests unless skipped
    if [ "$SKIP_TESTS" = false ]; then
        log_info "Running tests..."
        if ./dev.sh test > /dev/null 2>&1; then
            log_success "Tests passed"
        else
            log_error "Tests failed! Fix before deploying."
        fi
    else
        log_warn "Skipping tests (--skip-tests)"
    fi

    # Security audit
    log_info "Running security audit..."
    if ./security-audit.sh > /dev/null 2>&1; then
        log_success "Security audit passed"
    else
        log_warn "Security audit found issues"
        if [ "$DRY_RUN" = false ]; then
            read -p "Continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_error "Deployment cancelled"
            fi
        fi
    fi
}

# Build application
build_app() {
    if [ "$SKIP_BUILD" = true ]; then
        log_warn "Skipping build (--skip-build)"
        return
    fi

    section "Building Application"

    log_info "Building web-ui component..."
    cd web-ui
    cargo build --target wasm32-wasip2 --release
    cd ..

    log_info "Building with Spin..."
    spin build

    # Check WASM size
    WASM_FILE="web-ui/target/wasm32-wasip2/release/web_ui.wasm"
    if [ -f "$WASM_FILE" ]; then
        SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
        SIZE_MB=$((SIZE / 1024 / 1024))
        log_info "WASM binary size: ${SIZE_MB}MB"

        if [ $SIZE -gt 10000000 ]; then
            log_warn "WASM binary is larger than 10MB"
        fi
    fi

    log_success "Build complete"
}

# Deploy to local environment
deploy_local() {
    section "Deploying to Local Environment"

    preflight_checks
    build_app

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would start Spin server on http://localhost:3000"
        return
    fi

    log_info "Starting Spin server on http://localhost:3000"
    log_warn "Press Ctrl+C to stop"
    spin up
}

# Deploy to Fermyon Cloud
deploy_fermyon() {
    section "Deploying to Fermyon Cloud"

    if ! command -v spin &> /dev/null; then
        log_error "Spin CLI not found. Install from https://developer.fermyon.com/spin"
    fi

    preflight_checks
    build_app

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would deploy to Fermyon Cloud"
        return
    fi

    log_info "Deploying to Fermyon Cloud..."

    # Check if logged in
    if ! spin cloud apps list &> /dev/null; then
        log_info "Not logged in. Please log in to Fermyon Cloud"
        spin cloud login
    fi

    spin deploy

    log_success "Deployed to Fermyon Cloud"
    log_info "View logs: spin cloud logs"
    log_info "View apps: spin cloud apps list"
}

# Deploy with Docker
deploy_docker() {
    section "Deploying with Docker"

    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Install from https://docker.com"
    fi

    preflight_checks
    build_app

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would build and run Docker container"
        return
    fi

    log_info "Building Docker image..."
    docker build -t myt2abrp:latest .

    log_info "Stopping existing container..."
    docker stop myt2abrp 2>/dev/null || true
    docker rm myt2abrp 2>/dev/null || true

    log_info "Starting container..."
    docker run -d \
        --name myt2abrp \
        -p 3000:3000 \
        --env-file .env \
        --restart unless-stopped \
        myt2abrp:latest

    log_success "Deployed with Docker"
    log_info "Access at: http://localhost:3000"
    log_info "View logs: docker logs -f myt2abrp"
}

# Deploy production stack
deploy_prod() {
    section "Deploying Production Stack"

    if ! command -v docker-compose &> /dev/null; then
        log_error "docker-compose not found. Install from https://docs.docker.com/compose/"
    fi

    # Check for production environment file
    if [ ! -f ".env.prod" ]; then
        log_warn ".env.prod not found. Using .env"
        if [ ! -f ".env" ]; then
            log_error "Neither .env.prod nor .env found!"
        fi
    fi

    preflight_checks
    build_app

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would deploy production stack with monitoring"
        return
    fi

    log_info "Deploying production stack with monitoring..."
    docker-compose -f docker-compose.prod.yml up -d

    log_success "Production stack deployed"
    echo ""
    log_info "Services running:"
    log_info "  - Application: https://localhost (or your configured domain)"
    log_info "  - Grafana: http://localhost:3001 (admin/admin)"
    log_info "  - Prometheus: http://localhost:9090"
    echo ""
    log_info "View logs: docker-compose -f docker-compose.prod.yml logs -f"
    log_info "Check status: docker-compose -f docker-compose.prod.yml ps"
}

# Deploy to Kubernetes
deploy_k8s() {
    section "Deploying to Kubernetes"

    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found. Install from https://kubernetes.io/docs/tasks/tools/"
    fi

    preflight_checks

    # Check kubectl connectivity
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
    fi

    log_info "Connected to: $(kubectl config current-context)"

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would deploy to Kubernetes"
        return
    fi

    # Check for k8s directory
    if [ ! -d "k8s" ]; then
        log_error "k8s/ directory not found. See DEPLOYMENT.md for Kubernetes manifests."
    fi

    log_info "Creating namespace (if not exists)..."
    kubectl create namespace myt2abrp --dry-run=client -o yaml | kubectl apply -f -

    log_info "Creating secrets..."
    kubectl create secret generic myt2abrp-secrets \
        --from-literal=jwt-secret=$(openssl rand -base64 32) \
        --namespace=myt2abrp \
        --dry-run=client -o yaml | kubectl apply -f -

    log_info "Applying manifests..."
    kubectl apply -f k8s/ --namespace=myt2abrp

    log_success "Deployed to Kubernetes"
    log_info "Check status: kubectl get pods -n myt2abrp"
    log_info "View logs: kubectl logs -f -n myt2abrp -l app=myt2abrp"
}

# Health check
health_check() {
    local url="$1"
    local max_attempts=30
    local attempt=0

    log_info "Waiting for service to be healthy..."

    while [ $attempt -lt $max_attempts ]; do
        if curl -sf "${url}/health" > /dev/null 2>&1; then
            log_success "Service is healthy!"
            return 0
        fi

        attempt=$((attempt + 1))
        sleep 2
    done

    log_error "Service did not become healthy after ${max_attempts} attempts"
}

# Main deployment logic
main() {
    case "${TARGET:-help}" in
        local)
            deploy_local
            ;;
        fermyon)
            deploy_fermyon
            ;;
        docker)
            deploy_docker
            health_check "http://localhost:3000"
            ;;
        prod)
            deploy_prod
            health_check "http://localhost:3000"
            ;;
        k8s|kubernetes)
            deploy_k8s
            ;;
        help|--help|-h)
            cat << EOF
MyT2ABRP Deployment Automation

Usage:
    ./deploy.sh [target] [options]

Targets:
    local       Deploy to local environment (spin up)
    fermyon     Deploy to Fermyon Cloud
    docker      Build and run Docker container
    prod        Deploy production stack with monitoring
    k8s         Deploy to Kubernetes cluster
    help        Show this help message

Options:
    --skip-tests    Skip running tests before deployment
    --skip-build    Skip build step (use existing binaries)
    --dry-run       Show what would be deployed without deploying

Examples:
    ./deploy.sh local                    # Deploy locally
    ./deploy.sh fermyon                  # Deploy to Fermyon Cloud
    ./deploy.sh docker                   # Deploy with Docker
    ./deploy.sh prod                     # Deploy production stack
    ./deploy.sh k8s                      # Deploy to Kubernetes
    ./deploy.sh prod --dry-run           # Show what would be deployed
    ./deploy.sh fermyon --skip-tests     # Deploy without running tests

For more information, see DEPLOYMENT.md
EOF
            ;;
        *)
            log_error "Unknown target: $TARGET\nRun './deploy.sh help' for usage"
            ;;
    esac
}

main

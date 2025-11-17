#!/usr/bin/env bash
#
# Monitoring Stack Setup Script for MyT2ABRP
#
# Automatically configures and validates:
# - Prometheus (metrics collection)
# - Grafana (visualization)
# - Loki (log aggregation)
# - AlertManager (alerting)
# - Dashboards and datasources
#
# Usage:
#   ./setup-monitoring.sh install
#   ./setup-monitoring.sh configure
#   ./setup-monitoring.sh test
#   ./setup-monitoring.sh clean

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Check prerequisites
check_prerequisites() {
    section "Checking Prerequisites"

    local missing=0

    # Check Docker
    if command -v docker &> /dev/null; then
        log_success "Docker installed: $(docker --version | cut -d' ' -f3 | cut -d',' -f1)"
    else
        log_error "Docker not found. Install from https://docker.com"
        missing=1
    fi

    # Check Docker Compose
    if command -v docker-compose &> /dev/null; then
        log_success "Docker Compose installed: $(docker-compose --version | cut -d' ' -f3 | cut -d',' -f1)"
    else
        log_error "Docker Compose not found. Install from https://docs.docker.com/compose/"
        missing=1
    fi

    # Check if Docker is running
    if docker ps &> /dev/null; then
        log_success "Docker daemon is running"
    else
        log_error "Docker daemon is not running. Start Docker first."
        missing=1
    fi

    if [ $missing -eq 1 ]; then
        exit 1
    fi
}

# Install and configure monitoring stack
install_monitoring() {
    section "Installing Monitoring Stack"

    check_prerequisites

    # Check if configuration files exist
    local required_files=(
        "docker-compose.prod.yml"
        "prometheus.yml"
        "alertmanager.yml"
        "loki-config.yml"
        "promtail-config.yml"
        "grafana-dashboard.json"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Required file not found: $file"
        fi
    done
    log_success "All configuration files present"

    # Create necessary directories
    log_info "Creating directories..."
    mkdir -p ./{prometheus_data,grafana_data,loki_data}
    log_success "Directories created"

    # Set permissions
    log_info "Setting permissions..."
    chmod 777 ./{prometheus_data,grafana_data,loki_data}

    # Check environment file
    if [ ! -f ".env.prod" ] && [ ! -f ".env" ]; then
        log_warn "No environment file found. Creating from example..."
        if [ -f ".env.example" ]; then
            cp .env.example .env
        fi
    fi

    # Start monitoring stack
    log_info "Starting monitoring stack..."
    docker-compose -f docker-compose.prod.yml up -d \
        prometheus \
        grafana \
        loki \
        promtail \
        alertmanager \
        node-exporter

    # Wait for services to be ready
    log_info "Waiting for services to start..."
    sleep 10

    # Check service health
    local services=("prometheus:9090" "grafana:3000" "loki:3100")
    for service in "${services[@]}"; do
        name=$(echo "$service" | cut -d':' -f1)
        port=$(echo "$service" | cut -d':' -f2)

        if curl -sf "http://localhost:$port" > /dev/null 2>&1; then
            log_success "$name is running"
        else
            log_warn "$name may not be ready yet"
        fi
    done

    log_success "Monitoring stack installed!"
    echo ""
    log_info "Access points:"
    log_info "  - Grafana: http://localhost:3001 (admin/admin)"
    log_info "  - Prometheus: http://localhost:9090"
    log_info "  - AlertManager: http://localhost:9093"
    echo ""
    log_warn "Change Grafana admin password on first login!"
}

# Configure monitoring
configure_monitoring() {
    section "Configuring Monitoring"

    # Check if Grafana is running
    if ! docker ps | grep -q grafana; then
        log_error "Grafana is not running. Run 'install' first."
    fi

    log_info "Configuring Grafana datasources..."

    # Wait for Grafana to be ready
    local max_attempts=30
    local attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if curl -sf http://localhost:3001/api/health > /dev/null 2>&1; then
            break
        fi
        attempt=$((attempt + 1))
        sleep 2
    done

    if [ $attempt -eq $max_attempts ]; then
        log_error "Grafana did not start in time"
    fi

    log_success "Grafana is ready"

    # Add Prometheus datasource
    log_info "Adding Prometheus datasource..."
    curl -sf -X POST \
        -H "Content-Type: application/json" \
        -d '{
            "name": "Prometheus",
            "type": "prometheus",
            "url": "http://prometheus:9090",
            "access": "proxy",
            "isDefault": true
        }' \
        http://admin:admin@localhost:3001/api/datasources > /dev/null 2>&1 && \
        log_success "Prometheus datasource added" || log_warn "Prometheus datasource may already exist"

    # Add Loki datasource
    log_info "Adding Loki datasource..."
    curl -sf -X POST \
        -H "Content-Type: application/json" \
        -d '{
            "name": "Loki",
            "type": "loki",
            "url": "http://loki:3100",
            "access": "proxy"
        }' \
        http://admin:admin@localhost:3001/api/datasources > /dev/null 2>&1 && \
        log_success "Loki datasource added" || log_warn "Loki datasource may already exist"

    # Import dashboard
    if [ -f "grafana-dashboard.json" ]; then
        log_info "Importing Grafana dashboard..."
        dashboard_json=$(cat grafana-dashboard.json)
        curl -sf -X POST \
            -H "Content-Type: application/json" \
            -d "{
                \"dashboard\": $dashboard_json,
                \"overwrite\": true
            }" \
            http://admin:admin@localhost:3001/api/dashboards/db > /dev/null 2>&1 && \
            log_success "Dashboard imported" || log_warn "Dashboard import may have failed"
    fi

    log_success "Configuration complete!"
}

# Test monitoring stack
test_monitoring() {
    section "Testing Monitoring Stack"

    # Check if services are running
    log_info "Checking service status..."

    services=("prometheus" "grafana" "loki" "promtail" "alertmanager")
    all_running=true

    for service in "${services[@]}"; do
        if docker ps | grep -q "$service"; then
            log_success "$service is running"
        else
            log_error "$service is not running"
            all_running=false
        fi
    done

    if [ "$all_running" = false ]; then
        log_error "Some services are not running"
    fi

    # Test Prometheus
    log_info "Testing Prometheus..."
    if curl -sf http://localhost:9090/-/healthy > /dev/null; then
        log_success "Prometheus is healthy"

        # Check if it's scraping metrics
        metrics=$(curl -sf "http://localhost:9090/api/v1/targets" | grep -o '"up"' | wc -l)
        log_info "Prometheus is monitoring $metrics targets"
    else
        log_warn "Prometheus health check failed"
    fi

    # Test Grafana
    log_info "Testing Grafana..."
    if curl -sf http://localhost:3001/api/health > /dev/null; then
        log_success "Grafana is healthy"

        # Check datasources
        datasources=$(curl -sf http://admin:admin@localhost:3001/api/datasources | grep -o '"name"' | wc -l)
        log_info "Grafana has $datasources datasources configured"
    else
        log_warn "Grafana health check failed"
    fi

    # Test Loki
    log_info "Testing Loki..."
    if curl -sf http://localhost:3100/ready > /dev/null; then
        log_success "Loki is healthy"
    else
        log_warn "Loki health check failed"
    fi

    # Test application metrics endpoint
    log_info "Testing application metrics..."
    if curl -sf http://localhost:3000/api/metrics > /dev/null 2>&1; then
        log_success "Application metrics endpoint is accessible"
    else
        log_warn "Application is not running or metrics endpoint not accessible"
    fi

    # Test AlertManager
    log_info "Testing AlertManager..."
    if curl -sf http://localhost:9093/-/healthy > /dev/null; then
        log_success "AlertManager is healthy"
    else
        log_warn "AlertManager health check failed"
    fi

    log_success "All tests complete!"
    echo ""
    log_info "View logs:"
    log_info "  docker-compose -f docker-compose.prod.yml logs -f prometheus"
    log_info "  docker-compose -f docker-compose.prod.yml logs -f grafana"
}

# Show monitoring status
show_status() {
    section "Monitoring Stack Status"

    docker-compose -f docker-compose.prod.yml ps

    echo ""
    log_info "Resource usage:"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep -E "prometheus|grafana|loki|promtail|alertmanager"
}

# Clean monitoring data
clean_monitoring() {
    section "Cleaning Monitoring Data"

    log_warn "This will remove all monitoring data!"
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cancelled"
        exit 0
    fi

    log_info "Stopping monitoring stack..."
    docker-compose -f docker-compose.prod.yml down \
        prometheus grafana loki promtail alertmanager node-exporter 2>/dev/null || true

    log_info "Removing volumes..."
    docker volume rm -f prometheus_data grafana_data loki_data 2>/dev/null || true

    log_info "Removing data directories..."
    rm -rf ./{prometheus_data,grafana_data,loki_data}

    log_success "Monitoring data cleaned"
}

# Main
main() {
    case "${1:-help}" in
        install)
            install_monitoring
            ;;
        configure)
            configure_monitoring
            ;;
        test)
            test_monitoring
            ;;
        status)
            show_status
            ;;
        clean)
            clean_monitoring
            ;;
        help|--help|-h)
            cat << EOF
MyT2ABRP Monitoring Setup Tool

Usage:
    ./setup-monitoring.sh <command>

Commands:
    install       Install and start monitoring stack
    configure     Configure Grafana datasources and dashboards
    test          Run health checks on monitoring services
    status        Show current status of monitoring stack
    clean         Remove all monitoring data and containers
    help          Show this help message

Examples:
    ./setup-monitoring.sh install        # Install monitoring stack
    ./setup-monitoring.sh configure      # Configure Grafana
    ./setup-monitoring.sh test           # Test all services
    ./setup-monitoring.sh status         # Check status
    ./setup-monitoring.sh clean          # Clean everything

Monitoring Stack:
    - Prometheus (metrics):  http://localhost:9090
    - Grafana (dashboards):  http://localhost:3001 (admin/admin)
    - Loki (logs):           http://localhost:3100
    - AlertManager:          http://localhost:9093

For more information, see DEPLOYMENT.md
EOF
            ;;
        *)
            log_error "Unknown command: $1\nRun './setup-monitoring.sh help' for usage"
            ;;
    esac
}

main "$@"

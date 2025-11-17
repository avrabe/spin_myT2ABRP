#!/usr/bin/env bash
#
# Backup and Restore Script for MyT2ABRP
#
# Handles backup and restoration of:
# - Application configuration (.env files)
# - Database dumps (if using PostgreSQL)
# - Monitoring data (Prometheus, Grafana)
# - Docker volumes
# - SSL certificates
#
# Usage:
#   ./backup.sh backup [--output /path/to/backup]
#   ./backup.sh restore /path/to/backup.tar.gz
#   ./backup.sh list
#   ./backup.sh clean [--keep 5]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default settings
BACKUP_DIR="${BACKUP_DIR:-./.backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="myt2abrp_backup_${TIMESTAMP}"

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

# Create backup
backup() {
    local output_dir="${1:-$BACKUP_DIR}"

    section "Creating Backup"

    # Create backup directory
    mkdir -p "$output_dir"
    local backup_path="$output_dir/$BACKUP_NAME"
    mkdir -p "$backup_path"

    log_info "Backup location: $backup_path"

    # Backup configuration files
    log_info "Backing up configuration..."
    cp .env "$backup_path/.env" 2>/dev/null || log_warn ".env not found"
    cp .env.prod "$backup_path/.env.prod" 2>/dev/null || log_warn ".env.prod not found"
    cp spin.toml "$backup_path/spin.toml" 2>/dev/null || true

    # Backup SSL certificates
    if [ -d "ssl" ]; then
        log_info "Backing up SSL certificates..."
        cp -r ssl "$backup_path/"
    fi

    # Backup Docker volumes (if using Docker Compose)
    if command -v docker &> /dev/null && docker-compose ps &> /dev/null 2>&1; then
        log_info "Backing up Docker volumes..."

        # List of volumes to backup
        VOLUMES=(
            "prometheus_data"
            "grafana_data"
            "loki_data"
            "postgres_data"
        )

        mkdir -p "$backup_path/volumes"

        for volume in "${VOLUMES[@]}"; do
            if docker volume inspect "$volume" &> /dev/null; then
                log_info "  - Backing up volume: $volume"
                docker run --rm \
                    -v "$volume:/source:ro" \
                    -v "$(pwd)/$backup_path/volumes:/backup" \
                    alpine tar czf "/backup/${volume}.tar.gz" -C /source .
            fi
        done
    fi

    # Backup PostgreSQL database (if running)
    if docker ps | grep -q postgres; then
        log_info "Backing up PostgreSQL database..."
        docker exec postgres pg_dumpall -U postgres > "$backup_path/postgres_dump.sql" || log_warn "PostgreSQL backup failed"
    fi

    # Backup Grafana dashboards
    if docker ps | grep -q grafana; then
        log_info "Backing up Grafana dashboards..."
        mkdir -p "$backup_path/grafana"
        cp -r grafana-provisioning "$backup_path/grafana/" 2>/dev/null || true
        cp grafana-dashboard.json "$backup_path/grafana/" 2>/dev/null || true
    fi

    # Create metadata file
    cat > "$backup_path/metadata.txt" << EOF
Backup created: $(date)
Host: $(hostname)
User: $(whoami)
Git branch: $(git branch --show-current 2>/dev/null || echo "N/A")
Git commit: $(git rev-parse HEAD 2>/dev/null || echo "N/A")
EOF

    # Compress backup
    log_info "Compressing backup..."
    tar czf "${backup_path}.tar.gz" -C "$output_dir" "$BACKUP_NAME"
    rm -rf "$backup_path"

    log_success "Backup created: ${backup_path}.tar.gz"

    # Calculate size
    SIZE=$(du -h "${backup_path}.tar.gz" | cut -f1)
    log_info "Backup size: $SIZE"
}

# Restore from backup
restore() {
    local backup_file="$1"

    if [ -z "$backup_file" ]; then
        log_error "Please specify backup file to restore"
    fi

    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
    fi

    section "Restoring from Backup"

    log_warn "This will overwrite existing configuration!"
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Restore cancelled"
        exit 0
    fi

    # Extract backup
    local temp_dir="/tmp/myt2abrp_restore_$$"
    mkdir -p "$temp_dir"

    log_info "Extracting backup..."
    tar xzf "$backup_file" -C "$temp_dir"

    # Find the backup directory
    local backup_dir=$(find "$temp_dir" -name "myt2abrp_backup_*" -type d | head -1)

    if [ -z "$backup_dir" ]; then
        log_error "Invalid backup file"
    fi

    # Show metadata
    if [ -f "$backup_dir/metadata.txt" ]; then
        log_info "Backup metadata:"
        cat "$backup_dir/metadata.txt"
        echo ""
    fi

    # Restore configuration files
    log_info "Restoring configuration..."
    cp "$backup_dir/.env" . 2>/dev/null && log_success "  - Restored .env" || log_warn "  - .env not found in backup"
    cp "$backup_dir/.env.prod" . 2>/dev/null && log_success "  - Restored .env.prod" || true

    # Restore SSL certificates
    if [ -d "$backup_dir/ssl" ]; then
        log_info "Restoring SSL certificates..."
        cp -r "$backup_dir/ssl" .
        log_success "  - Restored SSL certificates"
    fi

    # Restore Docker volumes
    if [ -d "$backup_dir/volumes" ]; then
        log_info "Restoring Docker volumes..."
        for volume_tar in "$backup_dir/volumes"/*.tar.gz; do
            if [ -f "$volume_tar" ]; then
                volume_name=$(basename "$volume_tar" .tar.gz)
                log_info "  - Restoring volume: $volume_name"

                # Create volume if doesn't exist
                docker volume create "$volume_name" &> /dev/null || true

                # Restore data
                docker run --rm \
                    -v "$volume_name:/target" \
                    -v "$volume_tar:/backup.tar.gz:ro" \
                    alpine sh -c "cd /target && tar xzf /backup.tar.gz"
            fi
        done
    fi

    # Restore PostgreSQL database
    if [ -f "$backup_dir/postgres_dump.sql" ]; then
        if docker ps | grep -q postgres; then
            log_info "Restoring PostgreSQL database..."
            docker exec -i postgres psql -U postgres < "$backup_dir/postgres_dump.sql"
            log_success "  - Restored PostgreSQL database"
        else
            log_warn "PostgreSQL container not running, skipping database restore"
        fi
    fi

    # Restore Grafana dashboards
    if [ -d "$backup_dir/grafana" ]; then
        log_info "Restoring Grafana configuration..."
        cp -r "$backup_dir/grafana/grafana-provisioning" . 2>/dev/null && log_success "  - Restored Grafana provisioning" || true
        cp "$backup_dir/grafana/grafana-dashboard.json" . 2>/dev/null && log_success "  - Restored Grafana dashboard" || true
    fi

    # Cleanup
    rm -rf "$temp_dir"

    log_success "Restore complete!"
    log_warn "Restart services for changes to take effect"
}

# List backups
list_backups() {
    section "Available Backups"

    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A $BACKUP_DIR 2>/dev/null)" ]; then
        log_warn "No backups found in $BACKUP_DIR"
        return
    fi

    echo ""
    printf "%-40s %10s %20s\n" "BACKUP FILE" "SIZE" "DATE"
    echo "----------------------------------------------------------------"

    for backup in "$BACKUP_DIR"/*.tar.gz; do
        if [ -f "$backup" ]; then
            name=$(basename "$backup")
            size=$(du -h "$backup" | cut -f1)
            date=$(stat -c %y "$backup" 2>/dev/null || stat -f %Sm "$backup" 2>/dev/null | cut -d' ' -f1-2)
            printf "%-40s %10s %20s\n" "$name" "$size" "$date"
        fi
    done

    echo ""
    total_size=$(du -sh "$BACKUP_DIR" 2>/dev/null | cut -f1)
    log_info "Total backup size: $total_size"
}

# Clean old backups
clean_backups() {
    local keep="${1:-5}"

    section "Cleaning Old Backups"

    if [ ! -d "$BACKUP_DIR" ]; then
        log_warn "No backup directory found"
        return
    fi

    # Count backups
    backup_count=$(find "$BACKUP_DIR" -name "*.tar.gz" | wc -l)

    log_info "Found $backup_count backups, keeping $keep most recent"

    if [ "$backup_count" -le "$keep" ]; then
        log_info "No backups to clean"
        return
    fi

    # Delete old backups
    find "$BACKUP_DIR" -name "*.tar.gz" -type f -printf '%T@ %p\n' | \
        sort -n | \
        head -n -"$keep" | \
        cut -d' ' -f2- | \
        while read -r file; do
            log_info "Deleting: $(basename "$file")"
            rm -f "$file"
        done

    log_success "Cleanup complete"
}

# Verify backup
verify_backup() {
    local backup_file="$1"

    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
    fi

    section "Verifying Backup"

    log_info "Checking archive integrity..."
    if tar tzf "$backup_file" > /dev/null 2>&1; then
        log_success "Archive is valid"
    else
        log_error "Archive is corrupted!"
    fi

    log_info "Backup contents:"
    tar tzf "$backup_file" | head -20

    local file_count=$(tar tzf "$backup_file" | wc -l)
    log_info "Total files in backup: $file_count"
}

# Main
main() {
    case "${1:-help}" in
        backup)
            shift
            backup "$@"
            ;;
        restore)
            shift
            restore "$@"
            ;;
        list)
            list_backups
            ;;
        clean)
            shift
            keep="${1:-5}"
            clean_backups "$keep"
            ;;
        verify)
            shift
            verify_backup "$@"
            ;;
        help|--help|-h)
            cat << EOF
MyT2ABRP Backup and Restore Tool

Usage:
    ./backup.sh <command> [options]

Commands:
    backup [--output DIR]     Create a new backup
    restore FILE              Restore from backup file
    list                      List all available backups
    clean [--keep N]          Delete old backups (keep N most recent, default 5)
    verify FILE               Verify backup integrity
    help                      Show this help message

Examples:
    ./backup.sh backup                           # Create backup
    ./backup.sh backup --output /mnt/backups     # Backup to specific location
    ./backup.sh list                             # List backups
    ./backup.sh restore ./backups/backup.tar.gz  # Restore from backup
    ./backup.sh clean --keep 10                  # Keep 10 most recent backups
    ./backup.sh verify backup.tar.gz             # Verify backup file

Backup includes:
    - Configuration files (.env, .env.prod)
    - SSL certificates
    - Docker volumes (Prometheus, Grafana, Loki, PostgreSQL)
    - Database dumps
    - Grafana dashboards and provisioning

Backups are stored in: $BACKUP_DIR
EOF
            ;;
        *)
            log_error "Unknown command: $1\nRun './backup.sh help' for usage"
            ;;
    esac
}

main "$@"

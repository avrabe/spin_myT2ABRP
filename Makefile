# Makefile for MyT2ABRP
#
# Provides convenient make targets for common development tasks.
# All targets wrap the dev.sh, deploy.sh, and other automation scripts.

.PHONY: help build run test clean lint format audit deploy backup monitor install

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
NC := \033[0m

##@ General

help: ## Display this help message
	@echo "$(BLUE)MyT2ABRP - Makefile Targets$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make $(GREEN)<target>$(NC)\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(BLUE)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Development

build: ## Build all components
	@echo "$(BLUE)Building...$(NC)"
	@./dev.sh build

run: ## Build and run the application
	@echo "$(BLUE)Starting application...$(NC)"
	@./dev.sh run

test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	@./dev.sh test

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning...$(NC)"
	@./dev.sh clean

lint: ## Run linters (Clippy + rustfmt check)
	@echo "$(BLUE)Running linters...$(NC)"
	@./dev.sh lint

format: ## Format code (Rust + TypeScript)
	@echo "$(BLUE)Formatting code...$(NC)"
	@./dev.sh format

watch: ## Watch for changes and rebuild
	@echo "$(BLUE)Starting watch mode...$(NC)"
	@./dev.sh watch

docs: ## Generate and open documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	@./dev.sh docs

##@ Security & Quality

audit: ## Run security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	@./security-audit.sh

audit-fix: ## Run security audit with auto-fix
	@echo "$(BLUE)Running security audit with auto-fix...$(NC)"
	@./security-audit.sh --fix

##@ Deployment

deploy-local: ## Deploy to local environment
	@echo "$(BLUE)Deploying locally...$(NC)"
	@./deploy.sh local

deploy-docker: ## Deploy with Docker
	@echo "$(BLUE)Deploying with Docker...$(NC)"
	@./deploy.sh docker

deploy-prod: ## Deploy production stack
	@echo "$(BLUE)Deploying production stack...$(NC)"
	@./deploy.sh prod

deploy-fermyon: ## Deploy to Fermyon Cloud
	@echo "$(BLUE)Deploying to Fermyon Cloud...$(NC)"
	@./deploy.sh fermyon

deploy-k8s: ## Deploy to Kubernetes
	@echo "$(BLUE)Deploying to Kubernetes...$(NC)"
	@./deploy.sh k8s

##@ Backup & Restore

backup: ## Create backup
	@echo "$(BLUE)Creating backup...$(NC)"
	@./backup.sh backup

backup-list: ## List available backups
	@./backup.sh list

backup-clean: ## Clean old backups (keep 5 most recent)
	@echo "$(BLUE)Cleaning old backups...$(NC)"
	@./backup.sh clean

restore: ## Restore from backup (usage: make restore BACKUP=path/to/backup.tar.gz)
	@if [ -z "$(BACKUP)" ]; then \
		echo "$(YELLOW)Error: Please specify BACKUP=path/to/backup.tar.gz$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Restoring from $(BACKUP)...$(NC)"
	@./backup.sh restore $(BACKUP)

##@ Monitoring

monitor-install: ## Install monitoring stack
	@echo "$(BLUE)Installing monitoring stack...$(NC)"
	@./setup-monitoring.sh install

monitor-configure: ## Configure monitoring (datasources, dashboards)
	@echo "$(BLUE)Configuring monitoring...$(NC)"
	@./setup-monitoring.sh configure

monitor-test: ## Test monitoring services
	@echo "$(BLUE)Testing monitoring services...$(NC)"
	@./setup-monitoring.sh test

monitor-status: ## Show monitoring status
	@./setup-monitoring.sh status

monitor-clean: ## Clean monitoring data
	@echo "$(BLUE)Cleaning monitoring data...$(NC)"
	@./setup-monitoring.sh clean

##@ Testing & Performance

loadtest: ## Run full load test suite
	@echo "$(BLUE)Running load tests...$(NC)"
	@./loadtest.sh full

loadtest-quick: ## Run quick load test
	@./loadtest.sh load http://localhost:3000/

benchmark: ## Run performance benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	@./benchmark.sh

##@ Installation

install: ## Install project dependencies
	@echo "$(BLUE)Installing dependencies...$(NC)"
	@echo "Checking prerequisites..."
	@command -v rustc > /dev/null || (echo "$(YELLOW)Rust not found. Install from https://rustup.rs$(NC)" && exit 1)
	@command -v spin > /dev/null || (echo "$(YELLOW)Spin not found. Install from https://developer.fermyon.com/spin$(NC)" && exit 1)
	@echo "$(GREEN)✓ Rust installed:$(NC) $$(rustc --version)"
	@echo "$(GREEN)✓ Spin installed:$(NC) $$(spin --version)"
	@if [ -d "tests" ]; then \
		echo "Installing test dependencies..."; \
		cd tests && npm install; \
		npx playwright install chromium; \
	fi
	@if [ ! -f ".env" ]; then \
		echo "Creating .env from .env.example..."; \
		cp .env.example .env; \
		echo "$(YELLOW)⚠ Please edit .env with your configuration$(NC)"; \
	fi
	@echo "$(GREEN)✓ Installation complete!$(NC)"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Edit .env with your configuration"
	@echo "  2. Run: make build"
	@echo "  3. Run: make run"

##@ Maintenance

update-deps: ## Update dependencies
	@echo "$(BLUE)Updating Rust dependencies...$(NC)"
	@cd web-ui && cargo update
	@if [ -d "tests" ]; then \
		echo "Updating test dependencies..."; \
		cd tests && npm update; \
	fi

check: ## Run all checks (format, lint, audit, test)
	@echo "$(BLUE)Running all checks...$(NC)"
	@$(MAKE) format
	@$(MAKE) lint
	@$(MAKE) audit
	@$(MAKE) test
	@echo "$(GREEN)✓ All checks passed!$(NC)"

ci: ## Run CI pipeline locally
	@echo "$(BLUE)Running CI pipeline locally...$(NC)"
	@$(MAKE) lint
	@$(MAKE) audit
	@$(MAKE) build
	@$(MAKE) test
	@echo "$(GREEN)✓ CI pipeline complete!$(NC)"

##@ Docker Compose Shortcuts

up: ## Start services with docker-compose
	@docker-compose up -d

down: ## Stop services with docker-compose
	@docker-compose down

logs: ## View docker-compose logs
	@docker-compose logs -f

ps: ## Show docker-compose status
	@docker-compose ps

restart: ## Restart docker-compose services
	@docker-compose restart

##@ Utilities

version: ## Show project version and tool versions
	@echo "$(BLUE)MyT2ABRP Version Information$(NC)"
	@echo ""
	@echo "Project:"
	@echo "  Version: $$(git describe --tags 2>/dev/null || echo 'dev')"
	@echo "  Commit:  $$(git rev-parse --short HEAD)"
	@echo "  Branch:  $$(git branch --show-current)"
	@echo ""
	@echo "Tools:"
	@echo "  Rust:   $$(rustc --version 2>/dev/null || echo 'not installed')"
	@echo "  Cargo:  $$(cargo --version 2>/dev/null || echo 'not installed')"
	@echo "  Spin:   $$(spin --version 2>/dev/null || echo 'not installed')"
	@echo "  Docker: $$(docker --version 2>/dev/null || echo 'not installed')"
	@echo "  Node:   $$(node --version 2>/dev/null || echo 'not installed')"

status: ## Show project status
	@echo "$(BLUE)MyT2ABRP Status$(NC)"
	@echo ""
	@echo "Git:"
	@git status --short
	@echo ""
	@echo "Services:"
	@docker ps --filter "name=myt2abrp" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || echo "  No Docker services running"

info: ## Display project information
	@echo "$(BLUE)MyT2ABRP Project Information$(NC)"
	@echo ""
	@echo "Repository: https://github.com/avrabe/spin_myT2ABRP"
	@echo "Documentation: ./DOCS_INDEX.md"
	@echo ""
	@echo "Quick Start:"
	@echo "  make install     # Install dependencies"
	@echo "  make build       # Build the project"
	@echo "  make run         # Run the application"
	@echo "  make test        # Run tests"
	@echo ""
	@echo "For more commands, run: make help"

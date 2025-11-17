# MyT2ABRP Documentation Index

Complete guide to all documentation available for the MyT2ABRP project.

## Quick Links

- **🚀 [Get Started in 5 Minutes](GETTING_STARTED.md)** - New developers start here
- **📖 [README](README.md)** - Project overview and features
- **🏗️ [Architecture Guide](ARCHITECTURE.md)** - Technical design deep-dive
- **🚢 [Deployment Guide](DEPLOYMENT.md)** - Production deployment

## Documentation by Category

### Getting Started

| Document | Purpose | Audience |
|----------|---------|----------|
| [README.md](README.md) | Project overview, quick start, features list | Everyone |
| [GETTING_STARTED.md](GETTING_STARTED.md) | 5-minute setup, development workflow | New developers |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute, code style, PR process | Contributors |

### Architecture & Design

| Document | Purpose | Audience |
|----------|---------|----------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Comprehensive technical architecture | Developers, architects |
| [openapi-web-ui.yaml](openapi-web-ui.yaml) | Complete API specification | API consumers, developers |
| [SESSION_2_SUMMARY.md](SESSION_2_SUMMARY.md) | Implementation details from Session 2 | Project maintainers |

### Deployment & Operations

| Document | Purpose | Audience |
|----------|---------|----------|
| [DEPLOYMENT.md](DEPLOYMENT.md) | Complete deployment guide (Docker, K8s, Cloud) | DevOps, SRE |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Common issues and solutions | Everyone |
| [docker-compose.yml](docker-compose.yml) | Development environment | Developers |
| [docker-compose.prod.yml](docker-compose.prod.yml) | Production environment with monitoring | DevOps |
| [Dockerfile](Dockerfile) | Container image definition | DevOps |

### Configuration

| Document | Purpose | Audience |
|----------|---------|----------|
| [.env.example](.env.example) | Environment variable template | Developers, DevOps |
| [spin.toml](spin.toml) | Spin application configuration | Developers |
| [nginx.prod.conf](nginx.prod.conf) | Production nginx configuration | DevOps |
| [prometheus.yml](prometheus.yml) | Metrics collection configuration | SRE |
| [alertmanager.yml](alertmanager.yml) | Alert routing configuration | SRE |
| [loki-config.yml](loki-config.yml) | Log aggregation configuration | SRE |
| [promtail-config.yml](promtail-config.yml) | Log shipping configuration | SRE |

### Testing & Quality

| Document | Purpose | Audience |
|----------|---------|----------|
| [tests/README.md](tests/README.md) | Testing guide (Playwright E2E) | QA, developers |
| [tests/playwright.config.ts](tests/playwright.config.ts) | Test configuration | QA |
| [benchmark.sh](benchmark.sh) | Performance benchmarking tool | Performance engineers |
| [loadtest.sh](loadtest.sh) | Load testing tool | Performance engineers |

### Monitoring & Observability

| Document | Purpose | Audience |
|----------|---------|----------|
| [grafana-dashboard.json](grafana-dashboard.json) | Pre-configured Grafana dashboard | SRE |
| [grafana-provisioning/](grafana-provisioning/) | Grafana auto-provisioning | SRE |

### Component-Specific

| Document | Purpose | Audience |
|----------|---------|----------|
| [web-ui/README.md](web-ui/README.md) | Web UI component documentation | Frontend developers |
| [components/README.md](components/README.md) | Bazel component architecture | Backend developers |
| [web-ui/static/](web-ui/static/) | Frontend assets (HTML/CSS/JS) | Frontend developers |

## Documentation by User Journey

### 🎯 I want to...

#### ...run the application locally

1. Start with [GETTING_STARTED.md](GETTING_STARTED.md)
2. Check [.env.example](.env.example) for configuration
3. If issues arise, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

#### ...understand the architecture

1. Read [README.md](README.md) - High-level overview
2. Study [ARCHITECTURE.md](ARCHITECTURE.md) - Detailed design
3. Review [openapi-web-ui.yaml](openapi-web-ui.yaml) - API contracts

#### ...deploy to production

1. Read [DEPLOYMENT.md](DEPLOYMENT.md) - Choose your platform
2. Configure using [.env.example](.env.example)
3. Set up monitoring with [docker-compose.prod.yml](docker-compose.prod.yml)
4. Configure nginx using [nginx.prod.conf](nginx.prod.conf)
5. Monitor issues via [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

#### ...contribute to the project

1. Read [CONTRIBUTING.md](CONTRIBUTING.md) - Guidelines
2. Setup dev environment with [GETTING_STARTED.md](GETTING_STARTED.md)
3. Run tests using [tests/README.md](tests/README.md)
4. Understand architecture from [ARCHITECTURE.md](ARCHITECTURE.md)

#### ...test the application

1. Unit tests: See component READMEs (e.g., [web-ui/README.md](web-ui/README.md))
2. E2E tests: See [tests/README.md](tests/README.md)
3. Performance tests: Run [benchmark.sh](benchmark.sh)
4. Load tests: Run [loadtest.sh](loadtest.sh)

#### ...monitor in production

1. Deploy monitoring stack from [docker-compose.prod.yml](docker-compose.prod.yml)
2. Configure Prometheus using [prometheus.yml](prometheus.yml)
3. Set up alerts with [alertmanager.yml](alertmanager.yml)
4. View metrics in Grafana (auto-provisioned dashboard)
5. Check logs in Loki via [loki-config.yml](loki-config.yml)

#### ...integrate with the API

1. Read [openapi-web-ui.yaml](openapi-web-ui.yaml) - Complete API spec
2. Check [ARCHITECTURE.md](ARCHITECTURE.md) - Communication patterns
3. Review [web-ui/src/lib.rs](web-ui/src/lib.rs) - Implementation

#### ...troubleshoot issues

1. Start with [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
2. Check health endpoint: `curl http://localhost:3000/health`
3. View metrics: `curl http://localhost:3000/api/metrics`
4. Review logs in `docker logs` or Loki
5. Search [GitHub Issues](https://github.com/avrabe/spin_myT2ABRP/issues)

## File Organization

```
spin_myT2ABRP/
├── 📄 Documentation (Markdown)
│   ├── README.md                  # Start here
│   ├── GETTING_STARTED.md         # Developer quick start
│   ├── ARCHITECTURE.md            # Technical design
│   ├── DEPLOYMENT.md              # Production deployment
│   ├── CONTRIBUTING.md            # Contribution guidelines
│   ├── TROUBLESHOOTING.md         # Common issues
│   └── DOCS_INDEX.md             # This file
│
├── 🔧 Configuration
│   ├── .env.example               # Environment template
│   ├── spin.toml                  # Spin app config
│   ├── docker-compose.yml         # Dev environment
│   ├── docker-compose.prod.yml    # Prod environment
│   ├── Dockerfile                 # Container definition
│   ├── nginx.prod.conf            # Reverse proxy
│   ├── prometheus.yml             # Metrics config
│   ├── alertmanager.yml           # Alerts config
│   ├── loki-config.yml            # Logs aggregation
│   └── promtail-config.yml        # Log shipping
│
├── 🧪 Testing
│   ├── tests/                     # Playwright E2E tests
│   ├── benchmark.sh               # Performance benchmarks
│   └── loadtest.sh                # Load testing
│
├── 📊 Monitoring
│   ├── grafana-dashboard.json     # Dashboard definition
│   └── grafana-provisioning/      # Auto-provisioning
│
├── 💻 Source Code
│   ├── web-ui/                    # Web UI component
│   ├── components/                # Bazel components
│   └── ios-app/                   # iOS application
│
└── 🔨 Build Scripts
    ├── build.sh                   # Main build script
    └── verify-bazel-setup.sh      # Bazel verification
```

## Documentation Standards

### For Developers

When creating or updating documentation:

1. **Use Markdown** - All docs should be `.md` format
2. **Include examples** - Show, don't just tell
3. **Keep it current** - Update docs with code changes
4. **Link liberally** - Cross-reference related docs
5. **Test instructions** - Verify all commands work

### Markdown Conventions

- Use `# Heading` for title (only one per file)
- Use `## Section` for major sections
- Use `### Subsection` for sub-sections
- Use code blocks with language:
  ````markdown
  ```bash
  command here
  ```
  ````
- Use tables for structured data
- Use emoji for quick visual scanning (sparingly)

## Finding Information

### Quick Search Tips

| Looking for... | Search in... |
|----------------|--------------|
| Setup instructions | GETTING_STARTED.md, README.md |
| API endpoints | openapi-web-ui.yaml, ARCHITECTURE.md |
| Build errors | TROUBLESHOOTING.md, web-ui/README.md |
| Deployment steps | DEPLOYMENT.md |
| Test failures | tests/README.md, TROUBLESHOOTING.md |
| Configuration options | .env.example, spin.toml |
| Performance issues | TROUBLESHOOTING.md, benchmark.sh |
| Contributing rules | CONTRIBUTING.md |

### Search All Docs

```bash
# Search all markdown files
grep -r "search term" *.md

# Search all documentation
find . -name "*.md" -o -name "*.yml" | xargs grep "search term"
```

## External Resources

- **Fermyon Spin Docs**: https://developer.fermyon.com/spin
- **HTMX Documentation**: https://htmx.org/docs/
- **Playwright Docs**: https://playwright.dev/
- **Prometheus Docs**: https://prometheus.io/docs/
- **Grafana Docs**: https://grafana.com/docs/
- **Project Repository**: https://github.com/avrabe/spin_myT2ABRP
- **Issue Tracker**: https://github.com/avrabe/spin_myT2ABRP/issues

## Documentation Changelog

### 2025-11-17
- Created comprehensive documentation index
- Added deployment guide with multiple platforms
- Created load testing and benchmarking tools
- Set up complete monitoring stack configurations
- Added Grafana dashboard and provisioning

### Previous Sessions
- See [SESSION_2_SUMMARY.md](SESSION_2_SUMMARY.md) for Session 2 details
- Initial documentation created with project

## Getting Help

### Documentation Issues

Found a documentation bug or unclear section?

1. **Quick Fix**: Open an issue at https://github.com/avrabe/spin_myT2ABRP/issues
2. **Contribute**: Submit a PR with improvements (see [CONTRIBUTING.md](CONTRIBUTING.md))
3. **Ask**: Use [GitHub Discussions](https://github.com/avrabe/spin_myT2ABRP/discussions)

### Support Channels

- **Bug Reports**: [GitHub Issues](https://github.com/avrabe/spin_myT2ABRP/issues)
- **Questions**: [GitHub Discussions](https://github.com/avrabe/spin_myT2ABRP/discussions)
- **Email**: ralf_beier@me.com

---

**Last Updated**: 2025-11-17
**Maintained By**: MyT2ABRP Team
**License**: MIT

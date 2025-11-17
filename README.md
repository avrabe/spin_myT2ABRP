# MyT2ABRP - A Better Route Planner for Toyota Electric Vehicles

[![Tests](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/test.yml/badge.svg)](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A comprehensive, multi-platform application that enhances the Toyota electric vehicle experience with smart charging management, battery health tracking, and intelligent route planning.

## 🚀 Features

### Web Dashboard (HTMX + Spin)
- ⚡ **Real-time Updates** - Auto-refreshing vehicle status every 5 seconds
- 🔋 **Battery Monitoring** - Live battery level, health, and temperature
- 🔌 **Charging Control** - Start/stop charging, set target levels
- 📊 **Analytics** - Cost tracking, efficiency metrics, charging history
- 📱 **Responsive Design** - Works on desktop and mobile
- 🌐 **PWA Support** - Install as a progressive web app

### iOS App (SwiftUI)
- 📱 **Native iOS Experience** - Built with SwiftUI for iOS 17+
- ⚡ **Smart Charging Alerts**:
  - 80% optimal charge notification (battery longevity)
  - Custom charge level alerts (user-defined)
  - Full charge notifications
  - Low battery warnings
  - Slow charging detection
- 🔋 **Battery Health Tracking** - Historical degradation monitoring
- 🗺️ **ABRP-Style Route Planning** - Intelligent charging stop recommendations
- 📊 **Comprehensive Analytics** - Cost, efficiency, environmental impact
- ⏰ **Quiet Hours** - Smart notification scheduling

### watchOS Companion App
- ⌚ **Quick Glance** - Battery level on watch face
- 🔌 **One-Tap Actions** - Start/stop charging from wrist
- 💫 **Complications** - All watch face styles supported
- 📳 **Haptic Feedback** - Tactile confirmation on actions
- 🔄 **Real-time Sync** - Updates every 15 minutes

## 🏗️ Architecture

```
MyT2ABRP
├── Spin Server (WebAssembly)
│   ├── web-ui component (HTMX dashboard)
│   └── myt2abrp component (Toyota API integration)
├── iOS Native App (SwiftUI + MVVM)
└── watchOS Companion App (SwiftUI + Complications)
```

### Technology Stack

**Backend**:
- **Runtime**: Fermyon Spin (WebAssembly serverless)
- **Language**: Rust
- **Build**: Bazel + Cargo

**Web Frontend**:
- **Framework**: HTMX (hypermedia-driven)
- **Styling**: Custom CSS (Toyota brand)
- **JavaScript**: Minimal (~1KB)

**Mobile**:
- **Language**: Swift
- **Framework**: SwiftUI
- **Architecture**: MVVM
- **Platforms**: iOS 17+, watchOS 10+

**Testing**:
- **E2E**: Playwright (57+ tests)
- **Coverage**: Web UI, API, Integration
- **CI/CD**: GitHub Actions

## 📋 Prerequisites

- **Rust** 1.70+
- **Spin CLI** 2.7.0+
- **Node.js** 20+ (for tests)
- **Xcode** 15+ (for iOS/watchOS)
- **Bazel** 7.0+ (for main API component)

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/avrabe/spin_myT2ABRP.git
cd spin_myT2ABRP
```

### 2. Set Up Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit .env and set your configuration
# IMPORTANT: Change JWT_SECRET and HMAC_KEY in production!
nano .env
```

### 3. Build and Run

```bash
# Build all Spin components
spin build

# Start the server
spin up

# Access web UI at http://localhost:3000
```

### 4. Run Tests

```bash
cd tests
npm install
npx playwright install
npm test
```

## 📱 iOS App Setup

```bash
# Open iOS project in Xcode
open ios-app/MyT2ABRP.xcodeproj

# Build and run (Cmd+R)
# - Simulator: Works out of the box
# - Device: Requires Apple Developer account
```

## 📚 Documentation

- [Architecture Guide](ARCHITECTURE.md) - Comprehensive technical overview
- [Testing Guide](tests/README.md) - E2E and integration testing
- [Deployment Guide](#deployment) - Production deployment instructions
- [API Documentation](#api-endpoints) - All API endpoints
- [Session 2 Summary](SESSION_2_SUMMARY.md) - Latest implementation details

## 🔌 API Endpoints

### Vehicle Status
```
GET /api/vehicle/status       # Vehicle status (HTML fragment for HTMX)
GET /api/range                 # Range information
GET /api/battery/health        # Battery health metrics
```

### Charging Management
```
GET  /api/charging/status      # Charging status
POST /api/charging/start       # Start charging
POST /api/charging/stop        # Stop charging
POST /api/precondition         # Pre-condition cabin
GET  /api/charging/history     # Charging session history
```

### Analytics
```
GET /api/analytics/weekly      # Weekly statistics
GET /api/analytics/costs       # Cost analysis
GET /api/analytics/efficiency  # Efficiency metrics
```

### System
```
GET /health                    # Health check
GET /api/health                # Health check (alias)
GET /api/metrics               # Performance metrics
```

### Static Files
```
GET /                          # Main dashboard
GET /styles.css                # CSS stylesheet
GET /app.js                    # JavaScript
```

## 🎯 Innovative Features

Features **NOT** in the official Toyota app:

1. ✅ **Custom Charge Level Alerts** - Alert at any % (50-100%)
2. ✅ **80% Optimal Charge** - Battery longevity recommendation
3. ✅ **Battery Health Tracking** - Historical degradation
4. ✅ **Charge Cycle Counting** - Track cycles over time
5. ✅ **Cost per kWh Tracking** - Detailed cost analytics
6. ✅ **Charging Efficiency** - 92% efficiency monitoring
7. ✅ **Savings vs. Gasoline** - Cost comparison
8. ✅ **CO₂ Emissions Tracking** - Environmental impact
9. ✅ **ABRP-Style Route Planning** - Intelligent charging stops
10. ✅ **Charging History** - Session-by-session breakdown
11. ✅ **Slow Charging Detection** - Alert if power < 20kW
12. ✅ **Quiet Hours** - Smart notification scheduling
13. ✅ **Apple Watch App** - Full companion app
14. ✅ **Watch Complications** - Battery on watch face
15. ✅ **Haptic Alerts** - Tactile notifications
16. ✅ **One-Tap Watch Actions** - Control from wrist
17. ✅ **Custom Charge Targets** - Precise control
18. ✅ **Real-time Web Dashboard** - HTMX auto-updates
19. ✅ **Temperature Monitoring** - Battery thermal tracking
20. ✅ **Energy Consumption** - kWh/100km charts

## 🧪 Testing

### Run All Tests
```bash
cd tests
npm test
```

### Test Specific Suites
```bash
npm run test:web-ui        # Web UI tests
npm run test:api           # API endpoint tests
npm run test:integration   # Integration tests
```

### Debug Tests
```bash
npm run test:headed        # Run with visible browser
npm run test:debug         # Step-through debugging
npm run test:ui            # Interactive UI mode
```

### Test Coverage
- **57+ automated tests**
- **5 browser configurations** (Chrome, Firefox, Safari, Mobile)
- **Performance benchmarks** (< 100ms response times)
- **Memory leak detection**

## 🚢 Deployment

### Local Development
```bash
spin build && spin up
```

### Fermyon Cloud
```bash
spin deploy
```

### Self-Hosted (Docker)
```bash
docker build -t myt2abrp .
docker run -p 3000:3000 -v $(pwd)/.env:/app/.env myt2abrp
```

### Production Checklist
- [ ] Set strong `JWT_SECRET` (use `openssl rand -base64 32`)
- [ ] Set strong `HMAC_KEY` (use `openssl rand -hex 32`)
- [ ] Configure `CORS_ORIGIN` to your domain
- [ ] Enable HTTPS/TLS
- [ ] Set up monitoring and logging
- [ ] Configure backups
- [ ] Set up rate limiting
- [ ] Review security headers

## 🔒 Security

- **CSP**: Content Security Policy headers
- **CORS**: Configurable origin restrictions
- **JWT**: Secure token-based authentication
- **HMAC**: Username hashing
- **HTTPS**: TLS for production
- **Input Validation**: All user inputs sanitized
- **Security Headers**: X-Content-Type-Options, X-Frame-Options, etc.

## 📊 Performance

- **API Response Time**: < 100ms average
- **Page Load**: < 2s
- **HTMX Refresh**: < 500ms
- **Memory**: Stable (< 2x growth over 10 cycles)
- **WASM**: Near-native performance

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`npm test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Ralf Anton Beier**
- Email: ralf_beier@me.com
- GitHub: [@avrabe](https://github.com/avrabe)

## 🙏 Acknowledgments

- [Fermyon Spin](https://www.fermyon.com/spin) - WebAssembly serverless platform
- [HTMX](https://htmx.org/) - High-power tools for HTML
- [Playwright](https://playwright.dev/) - End-to-end testing
- [Toyota](https://www.toyota.com/) - For creating great electric vehicles

## 📈 Project Stats

- **Lines of Code**: ~5,000+
- **Components**: 3 (Web, iOS, watchOS)
- **Test Cases**: 57+
- **Platforms**: 5+ (Web Desktop, Mobile Web, iOS, watchOS, API)
- **Languages**: Rust, Swift, TypeScript, HTML/CSS
- **First Release**: 2024-11-17

## 🗺️ Roadmap

### Short Term
- [ ] Real Toyota API integration
- [ ] User authentication and multi-user support
- [ ] Database persistence
- [ ] Push notifications (iOS)
- [ ] Offline support (PWA)

### Long Term
- [ ] Android app
- [ ] CarPlay integration
- [ ] Machine learning (charge predictions)
- [ ] Social features (route sharing)
- [ ] Home charging system integration
- [ ] Multi-vehicle support

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/avrabe/spin_myT2ABRP/issues)
- **Discussions**: [GitHub Discussions](https://github.com/avrabe/spin_myT2ABRP/discussions)
- **Email**: ralf_beier@me.com

---

**Made with ❤️ for Toyota bZ4X owners**

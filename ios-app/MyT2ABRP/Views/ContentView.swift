import SwiftUI

struct ContentView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @EnvironmentObject var chargingManager: ChargingManager
    @State private var selectedTab = 0

    var body: some View {
        TabView(selection: $selectedTab) {
            // Dashboard Tab
            DashboardView()
                .tabItem {
                    Label("Dashboard", systemImage: "gauge")
                }
                .tag(0)

            // Charging Tab
            ChargingView()
                .tabItem {
                    Label("Charging", systemImage: "bolt.fill")
                }
                .tag(1)

            // Routes Tab
            RoutesView()
                .tabItem {
                    Label("Routes", systemImage: "map.fill")
                }
                .tag(2)

            // Analytics Tab
            AnalyticsView()
                .tabItem {
                    Label("Analytics", systemImage: "chart.bar.fill")
                }
                .tag(3)

            // Settings Tab
            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gearshape.fill")
                }
                .tag(4)
        }
        .accentColor(.red)
    }
}

// MARK: - Dashboard View
struct DashboardView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Battery Status Card
                    BatteryStatusCard()

                    // Charging Status (if charging)
                    if chargingManager.isCharging {
                        ChargingStatusCard()
                            .transition(.scale.combined(with: .opacity))
                    }

                    // Range Information
                    RangeCard()

                    // Quick Actions
                    QuickActionsCard()

                    // Recent Alerts
                    RecentAlertsCard()
                }
                .padding()
            }
            .navigationTitle("My Toyota")
            .refreshable {
                await vehicleManager.refresh()
            }
        }
    }
}

// MARK: - Battery Status Card
struct BatteryStatusCard: View {
    @EnvironmentObject var vehicleManager: VehicleManager

    var body: some View {
        VStack(alignment: .leading, spacing: 15) {
            HStack {
                Text("Battery Status")
                    .font(.headline)
                Spacer()
                Text(vehicleManager.lastUpdated, style: .relative)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            // Large Battery Percentage
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("\(vehicleManager.batteryLevel)")
                    .font(.system(size: 72, weight: .bold, design: .rounded))
                    .foregroundColor(batteryColor)

                VStack(alignment: .leading, spacing: 2) {
                    Text("%")
                        .font(.title)
                        .foregroundColor(.secondary)
                    batteryHealthIndicator
                }
            }

            // Battery Visual
            BatteryVisual(level: vehicleManager.batteryLevel)
                .frame(height: 60)

            // Status Details
            HStack {
                StatusDetail(icon: "leaf.fill", text: "\(vehicleManager.rangeKm) km", color: .green)
                Spacer()
                StatusDetail(icon: "thermometer", text: "\(vehicleManager.batteryTemp)°C", color: .orange)
                Spacer()
                StatusDetail(icon: "heart.fill", text: "\(vehicleManager.batteryHealth)%", color: .red)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var batteryColor: Color {
        switch vehicleManager.batteryLevel {
        case 80...100: return .green
        case 50..<80: return .blue
        case 20..<50: return .orange
        default: return .red
        }
    }

    private var batteryHealthIndicator: some View {
        HStack(spacing: 4) {
            Image(systemName: "heart.fill")
                .font(.caption2)
                .foregroundColor(.red)
            Text("\(vehicleManager.batteryHealth)%")
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

// MARK: - Battery Visual Component
struct BatteryVisual: View {
    let level: Int

    var body: some View {
        GeometryReader { geometry in
            ZStack(alignment: .leading) {
                // Battery outline
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.gray.opacity(0.3), lineWidth: 3)

                // Battery fill
                RoundedRectangle(cornerRadius: 6)
                    .fill(
                        LinearGradient(
                            colors: [batteryColor.opacity(0.8), batteryColor],
                            startPoint: .leading,
                            endPoint: .trailing
                        )
                    )
                    .frame(width: geometry.size.width * CGFloat(level) / 100)
                    .padding(3)
                    .animation(.spring(response: 0.6, dampingFraction: 0.8), value: level)

                // Battery terminal
                Rectangle()
                    .fill(Color.gray.opacity(0.3))
                    .frame(width: 6, height: geometry.size.height * 0.4)
                    .offset(x: geometry.size.width + 3)
            }
        }
    }

    private var batteryColor: Color {
        switch level {
        case 80...100: return .green
        case 50..<80: return .blue
        case 20..<50: return .orange
        default: return .red
        }
    }
}

// MARK: - Charging Status Card
struct ChargingStatusCard: View {
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        VStack(alignment: .leading, spacing: 15) {
            HStack {
                HStack(spacing: 8) {
                    Image(systemName: "bolt.fill")
                        .foregroundColor(.yellow)
                        .symbolEffect(.pulse, options: .repeating)
                    Text("Charging")
                        .font(.headline)
                }
                Spacer()
                Text("\(chargingManager.powerKW, specifier: "%.1f") kW")
                    .font(.caption)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(Color.green.opacity(0.2))
                    .cornerRadius(8)
            }

            // Progress Ring
            ZStack {
                ChargingProgressRing(
                    progress: Double(chargingManager.currentLevel) / Double(chargingManager.targetLevel),
                    currentLevel: chargingManager.currentLevel,
                    targetLevel: chargingManager.targetLevel
                )
                .frame(height: 180)

                VStack(spacing: 4) {
                    Text("\(chargingManager.currentLevel)%")
                        .font(.system(size: 48, weight: .bold))
                    Image(systemName: "arrow.right")
                        .font(.title3)
                        .foregroundColor(.secondary)
                    Text("\(chargingManager.targetLevel)%")
                        .font(.title2)
                        .foregroundColor(.secondary)
                }
            }

            // Time and Stats
            HStack {
                VStack(alignment: .leading) {
                    Text("Time Remaining")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(chargingManager.timeRemaining)
                        .font(.title3.bold())
                }

                Spacer()

                VStack(alignment: .trailing) {
                    Text("Energy Added")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("\(chargingManager.energyAdded, specifier: "%.1f") kWh")
                        .font(.title3.bold())
                }
            }
            .padding(.top, 10)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(
                    LinearGradient(
                        colors: [Color.blue.opacity(0.2), Color.cyan.opacity(0.1)],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .shadow(color: Color.blue.opacity(0.2), radius: 10, x: 0, y: 5)
        )
    }
}

// MARK: - Charging Progress Ring
struct ChargingProgressRing: View {
    let progress: Double
    let currentLevel: Int
    let targetLevel: Int

    var body: some View {
        ZStack {
            // Background ring
            Circle()
                .stroke(Color.gray.opacity(0.2), lineWidth: 20)

            // Progress ring
            Circle()
                .trim(from: 0, to: progress)
                .stroke(
                    LinearGradient(
                        colors: [.green, .blue, .purple],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ),
                    style: StrokeStyle(lineWidth: 20, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))
                .animation(.spring(response: 1.0, dampingFraction: 0.8), value: progress)

            // Glow effect
            Circle()
                .trim(from: 0, to: progress)
                .stroke(Color.blue.opacity(0.3), lineWidth: 25)
                .blur(radius: 5)
                .rotationEffect(.degrees(-90))
        }
    }
}

// Helper Views
struct StatusDetail: View {
    let icon: String
    let text: String
    let color: Color

    var body: some View {
        HStack(spacing: 4) {
            Image(systemName: icon)
                .foregroundColor(color)
            Text(text)
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

struct RangeCard: View {
    @EnvironmentObject var vehicleManager: VehicleManager

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Range")
                .font(.headline)

            HStack {
                VStack(alignment: .leading) {
                    Text("\(vehicleManager.rangeKm)")
                        .font(.system(size: 42, weight: .bold))
                    + Text(" km")
                        .font(.title2)
                        .foregroundColor(.secondary)
                    Text("Current range")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                VStack(alignment: .trailing) {
                    Text("\(vehicleManager.rangeAt80)")
                        .font(.system(size: 28, weight: .semibold))
                    + Text(" km")
                        .font(.callout)
                        .foregroundColor(.secondary)
                    Text("@ 80% (optimal)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }
}

struct QuickActionsCard: View {
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Actions")
                .font(.headline)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                ActionButton(
                    icon: "bolt.fill",
                    title: chargingManager.isCharging ? "Stop" : "Start",
                    color: chargingManager.isCharging ? .orange : .green
                ) {
                    chargingManager.toggleCharging()
                }

                ActionButton(icon: "snowflake", title: "Pre-condition", color: .blue) {
                    chargingManager.precondition()
                }

                ActionButton(icon: "map.fill", title: "Plan Route", color: .purple) {
                    // Navigate to routes
                }

                ActionButton(icon: "bell.fill", title: "Alerts", color: .red) {
                    // Show alerts
                }
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }
}

struct ActionButton: View {
    let icon: String
    let title: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundColor(.white)
                Text(title)
                    .font(.caption.bold())
                    .foregroundColor(.white)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 16)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(color)
            )
        }
    }
}

struct RecentAlertsCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Recent Alerts")
                .font(.headline)

            AlertItem(
                icon: "checkmark.circle.fill",
                title: "Charging Complete",
                message: "Your vehicle is fully charged",
                time: "5 min ago",
                color: .green
            )

            AlertItem(
                icon: "bolt.circle.fill",
                title: "Optimal Level Reached",
                message: "Battery at 80% - best for longevity",
                time: "2 hours ago",
                color: .blue
            )
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }
}

struct AlertItem: View {
    let icon: String
    let title: String
    let message: String
    let time: String
    let color: Color

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(color)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.bold())
                Text(message)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text(time)
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 8)
    }
}

#Preview {
    ContentView()
        .environmentObject(VehicleManager())
        .environmentObject(ChargingManager())
        .environmentObject(NotificationManager())
}

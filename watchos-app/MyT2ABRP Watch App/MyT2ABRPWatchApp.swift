import SwiftUI
import WatchKit
import UserNotifications

/// watchOS companion app for MyT2ABRP
/// INNOVATIVE FEATURES:
/// - Quick glance battery status
/// - Charging control from wrist
/// - Haptic alerts for charge levels
/// - Complications showing battery/charging status
/// - Quick trip status check
@main
struct MyT2ABRPWatchApp: App {
    @StateObject private var vehicleManager = VehicleManager()
    @StateObject private var chargingManager = ChargingManager()

    init() {
        // Request notification permissions for watch
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { granted, error in
            if granted {
                print("✅ Watch notification permission granted")
            }
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(vehicleManager)
                .environmentObject(chargingManager)
        }
    }
}

/// Main watch view
struct ContentView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @EnvironmentObject var chargingManager: ChargingManager
    @State private var showingCharging = false

    var body: some View {
        TabView {
            // Battery Status Tab
            BatteryView()
                .environmentObject(vehicleManager)
                .environmentObject(chargingManager)

            // Charging Tab
            ChargingControlView()
                .environmentObject(chargingManager)

            // Quick Actions Tab
            QuickActionsView()
                .environmentObject(chargingManager)
        }
        .tabViewStyle(.page)
        .onAppear {
            Task {
                await vehicleManager.refresh()
                await chargingManager.updateStatus()
            }
        }
    }
}

// MARK: - Battery View

struct BatteryView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        ScrollView {
            VStack(spacing: 15) {
                // Large battery percentage
                VStack(spacing: 5) {
                    Text("\(vehicleManager.batteryLevel)")
                        .font(.system(size: 60, weight: .bold, design: .rounded))
                        .foregroundColor(batteryColor)

                    Text("%")
                        .font(.title3)
                        .foregroundColor(.secondary)

                    if chargingManager.isCharging {
                        HStack(spacing: 4) {
                            Image(systemName: "bolt.fill")
                                .foregroundColor(.yellow)
                                .font(.caption)
                            Text("Charging")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }

                Divider()

                // Range
                VStack(spacing: 4) {
                    Text("\(vehicleManager.rangeKm) km")
                        .font(.title3.bold())
                    Text("Range")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Divider()

                // Battery health
                HStack {
                    Image(systemName: "heart.fill")
                        .foregroundColor(.red)
                        .font(.caption)
                    Text("Health: \(vehicleManager.batteryHealth)%")
                        .font(.caption)
                }

                // Last updated
                Text("Updated \(vehicleManager.lastUpdated, style: .relative)")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding()
        }
        .refreshable {
            await vehicleManager.refresh()
        }
    }

    private var batteryColor: Color {
        switch vehicleManager.batteryLevel {
        case 80...100: return .green
        case 50..<80: return .blue
        case 20..<50: return .orange
        default: return .red
        }
    }
}

// MARK: - Charging Control View

struct ChargingControlView: View {
    @EnvironmentObject var chargingManager: ChargingManager
    @State private var hapticFeedback = WKHapticType.start

    var body: some View {
        ScrollView {
            VStack(spacing: 12) {
                if chargingManager.isCharging {
                    // Charging in progress
                    VStack(spacing: 10) {
                        Text("Charging")
                            .font(.headline)

                        // Progress ring
                        ZStack {
                            Circle()
                                .stroke(Color.gray.opacity(0.3), lineWidth: 8)
                                .frame(width: 100, height: 100)

                            Circle()
                                .trim(from: 0, to: Double(chargingManager.currentLevel) / 100.0)
                                .stroke(Color.green, style: StrokeStyle(lineWidth: 8, lineCap: .round))
                                .frame(width: 100, height: 100)
                                .rotationEffect(.degrees(-90))

                            Text("\(chargingManager.currentLevel)%")
                                .font(.title3.bold())
                        }

                        // Stats
                        VStack(spacing: 6) {
                            HStack {
                                Image(systemName: "bolt.fill")
                                    .foregroundColor(.yellow)
                                Text("\(String(format: "%.1f", chargingManager.powerKW)) kW")
                            }
                            .font(.caption)

                            Text(chargingManager.timeRemaining)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        // Stop button
                        Button(action: {
                            WKInterfaceDevice.current().play(.stop)
                            chargingManager.stopCharging()
                        }) {
                            Label("Stop", systemImage: "stop.fill")
                                .font(.caption.bold())
                        }
                        .tint(.red)
                        .buttonStyle(.borderedProminent)
                    }

                } else {
                    // Not charging
                    VStack(spacing: 15) {
                        Image(systemName: "bolt.slash.fill")
                            .font(.system(size: 50))
                            .foregroundColor(.gray)

                        Text("Not Charging")
                            .font(.headline)

                        Button(action: {
                            WKInterfaceDevice.current().play(.start)
                            chargingManager.startCharging()
                        }) {
                            Label("Start Charging", systemImage: "bolt.fill")
                                .font(.caption.bold())
                        }
                        .tint(.green)
                        .buttonStyle(.borderedProminent)
                    }
                }
            }
            .padding()
        }
    }
}

// MARK: - Quick Actions View

struct QuickActionsView: View {
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        ScrollView {
            VStack(spacing: 10) {
                Text("Quick Actions")
                    .font(.headline)
                    .padding(.bottom, 5)

                // Charging toggle
                Button(action: {
                    WKInterfaceDevice.current().play(.click)
                    chargingManager.toggleCharging()
                }) {
                    HStack {
                        Image(systemName: chargingManager.isCharging ? "stop.fill" : "bolt.fill")
                        Text(chargingManager.isCharging ? "Stop" : "Charge")
                    }
                }
                .tint(chargingManager.isCharging ? .red : .green)
                .buttonStyle(.borderedProminent)

                // Pre-condition
                Button(action: {
                    WKInterfaceDevice.current().play(.click)
                    chargingManager.precondition()
                }) {
                    HStack {
                        Image(systemName: "snowflake")
                        Text("Pre-condition")
                    }
                }
                .tint(.blue)
                .buttonStyle(.borderedProminent)

                // Set charge target
                Menu {
                    Button("80% (Optimal)") {
                        chargingManager.setChargeTarget(80)
                    }
                    Button("90%") {
                        chargingManager.setChargeTarget(90)
                    }
                    Button("100% (Full)") {
                        chargingManager.setChargeTarget(100)
                    }
                } label: {
                    HStack {
                        Image(systemName: "target")
                        Text("Target: \(chargingManager.targetLevel)%")
                    }
                }
                .tint(.purple)
                .buttonStyle(.borderedProminent)
            }
            .padding()
        }
    }
}

#Preview {
    ContentView()
        .environmentObject(VehicleManager())
        .environmentObject(ChargingManager())
}

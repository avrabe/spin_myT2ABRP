import SwiftUI

/// Detailed charging view with smart controls
/// INNOVATIVE FEATURES:
/// - Custom charge target slider
/// - Smart alert configuration
/// - Charging history with analytics
/// - Cost tracking
struct ChargingView: View {
    @EnvironmentObject var chargingManager: ChargingManager
    @EnvironmentObject var vehicleManager: VehicleManager
    @State private var showingHistory = false

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Current Charging Status
                    currentStatusSection

                    // Charge Target Control - INNOVATIVE
                    chargeTargetSection

                    // Smart Alerts Configuration - INNOVATIVE
                    smartAlertsSection

                    // Charging Controls
                    controlsSection

                    // Recent Sessions
                    recentSessionsSection
                }
                .padding()
            }
            .navigationTitle("Charging")
            .refreshable {
                await chargingManager.updateStatus()
            }
            .sheet(isPresented: $showingHistory) {
                ChargingHistoryView()
                    .environmentObject(chargingManager)
            }
        }
    }

    // MARK: - Current Status Section

    private var currentStatusSection: some View {
        VStack(spacing: 15) {
            if chargingManager.isCharging {
                // Charging in progress
                HStack {
                    Image(systemName: "bolt.fill")
                        .foregroundColor(.yellow)
                        .font(.title2)
                        .symbolEffect(.pulse, options: .repeating)
                    Text("Charging in Progress")
                        .font(.headline)
                    Spacer()
                }

                // Large progress display
                ZStack {
                    ChargingProgressRing(
                        progress: Double(chargingManager.currentLevel) / 100.0,
                        currentLevel: chargingManager.currentLevel,
                        targetLevel: chargingManager.targetLevel
                    )
                    .frame(height: 200)

                    VStack(spacing: 8) {
                        Text("\(chargingManager.currentLevel)%")
                            .font(.system(size: 56, weight: .bold))
                        Image(systemName: "arrow.down")
                            .font(.title3)
                            .foregroundColor(.secondary)
                        Text("\(chargingManager.targetLevel)%")
                            .font(.title)
                            .foregroundColor(.secondary)
                    }
                }

                // Charging stats grid
                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 15) {
                    StatBox(
                        icon: "bolt.fill",
                        value: "\(String(format: "%.1f", chargingManager.powerKW)) kW",
                        label: "Power"
                    )

                    StatBox(
                        icon: "clock.fill",
                        value: chargingManager.timeRemaining,
                        label: "Time Left"
                    )

                    StatBox(
                        icon: "battery.100.bolt",
                        value: "\(String(format: "%.1f", chargingManager.energyAdded)) kWh",
                        label: "Energy Added"
                    )
                }

                if let eta = chargingManager.estimatedFullTime {
                    HStack {
                        Image(systemName: "clock.badge.checkmark")
                            .foregroundColor(.green)
                        Text("Full charge at \(eta, style: .time)")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                }

            } else {
                // Not charging
                VStack(spacing: 15) {
                    Image(systemName: "bolt.slash.fill")
                        .font(.system(size: 60))
                        .foregroundColor(.gray)

                    Text("Not Charging")
                        .font(.title2.bold())

                    Text("Current battery: \(vehicleManager.batteryLevel)%")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 40)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Charge Target Section - INNOVATIVE

    private var chargeTargetSection: some View {
        VStack(alignment: .leading, spacing: 15) {
            HStack {
                Image(systemName: "target")
                    .foregroundColor(.blue)
                Text("Charge Target")
                    .font(.headline)
                Spacer()
                Text("\(chargingManager.targetLevel)%")
                    .font(.title3.bold())
                    .foregroundColor(.blue)
            }

            Slider(
                value: Binding(
                    get: { Double(chargingManager.targetLevel) },
                    set: { chargingManager.setChargeTarget(Int($0)) }
                ),
                in: 50...100,
                step: 5
            )
            .tint(.blue)

            // Preset buttons
            HStack(spacing: 10) {
                PresetButton(label: "80%", value: 80, current: chargingManager.targetLevel) {
                    chargingManager.setChargeTarget(80)
                }
                PresetButton(label: "90%", value: 90, current: chargingManager.targetLevel) {
                    chargingManager.setChargeTarget(90)
                }
                PresetButton(label: "100%", value: 100, current: chargingManager.targetLevel) {
                    chargingManager.setChargeTarget(100)
                }
            }

            // Recommendation
            if chargingManager.targetLevel == 80 {
                HStack(spacing: 8) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Optimal for battery longevity")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            } else if chargingManager.targetLevel == 100 {
                HStack(spacing: 8) {
                    Image(systemName: "info.circle.fill")
                        .foregroundColor(.orange)
                    Text("Best for long trips")
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

    // MARK: - Smart Alerts Section - INNOVATIVE

    private var smartAlertsSection: some View {
        VStack(alignment: .leading, spacing: 15) {
            HStack {
                Image(systemName: "bell.badge.fill")
                    .foregroundColor(.red)
                Text("Smart Alerts")
                    .font(.headline)
            }

            // 80% Optimal Charge Alert
            Toggle(isOn: $chargingManager.alertAt80Percent) {
                HStack {
                    Image(systemName: "leaf.fill")
                        .foregroundColor(.green)
                    VStack(alignment: .leading) {
                        Text("Optimal Charge (80%)")
                            .font(.subheadline)
                        Text("Best for battery health")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .tint(.green)

            Divider()

            // Custom Level Alert
            VStack(alignment: .leading, spacing: 10) {
                Toggle(isOn: $chargingManager.customAlertEnabled) {
                    HStack {
                        Image(systemName: "target")
                            .foregroundColor(.purple)
                        Text("Custom Level Alert")
                            .font(.subheadline)
                    }
                }
                .tint(.purple)

                if chargingManager.customAlertEnabled {
                    HStack {
                        Text("\(chargingManager.customAlertLevel)%")
                            .font(.title3.bold())
                            .foregroundColor(.purple)
                            .frame(width: 60)

                        Slider(
                            value: Binding(
                                get: { Double(chargingManager.customAlertLevel) },
                                set: { chargingManager.customAlertLevel = Int($0) }
                            ),
                            in: 50...100,
                            step: 5
                        )
                        .tint(.purple)
                    }
                    .padding(.leading, 30)
                }
            }

            Divider()

            // Full Charge Alert
            Toggle(isOn: $chargingManager.alertAtFullCharge) {
                HStack {
                    Image(systemName: "battery.100.bolt")
                        .foregroundColor(.blue)
                    VStack(alignment: .leading) {
                        Text("Full Charge (100%)")
                            .font(.subheadline)
                        Text("Notify when fully charged")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .tint(.blue)

            Divider()

            // Low Battery Alert
            Toggle(isOn: $chargingManager.lowBatteryAlert) {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.orange)
                    Text("Low Battery Warning")
                        .font(.subheadline)
                }
            }
            .tint(.orange)

            Divider()

            // Slow Charging Alert
            Toggle(isOn: $chargingManager.slowChargingAlert) {
                HStack {
                    Image(systemName: "tortoise.fill")
                        .foregroundColor(.gray)
                    Text("Slow Charging Detection")
                        .font(.subheadline)
                }
            }
            .tint(.gray)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Controls Section

    private var controlsSection: some View {
        VStack(spacing: 12) {
            if chargingManager.isCharging {
                Button(action: {
                    chargingManager.stopCharging()
                }) {
                    HStack {
                        Image(systemName: "stop.fill")
                        Text("Stop Charging")
                            .font(.headline)
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.red)
                    .foregroundColor(.white)
                    .cornerRadius(12)
                }
            } else {
                Button(action: {
                    chargingManager.startCharging()
                }) {
                    HStack {
                        Image(systemName: "bolt.fill")
                        Text("Start Charging")
                            .font(.headline)
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.green)
                    .foregroundColor(.white)
                    .cornerRadius(12)
                }
            }

            Button(action: {
                chargingManager.precondition()
            }) {
                HStack {
                    Image(systemName: "snowflake")
                    Text("Pre-condition Cabin")
                        .font(.headline)
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(Color.blue)
                .foregroundColor(.white)
                .cornerRadius(12)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Recent Sessions Section

    private var recentSessionsSection: some View {
        VStack(alignment: .leading, spacing: 15) {
            HStack {
                Image(systemName: "clock.arrow.circlepath")
                    .foregroundColor(.purple)
                Text("Recent Sessions")
                    .font(.headline)
                Spacer()
                Button("View All") {
                    showingHistory = true
                }
                .font(.subheadline)
            }

            if chargingManager.chargingSessions.isEmpty {
                Text("No charging sessions yet")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.vertical, 20)
            } else {
                ForEach(chargingManager.chargingSessions.prefix(3)) { session in
                    ChargingSessionRow(session: session)
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

// MARK: - Supporting Views

struct StatBox: View {
    let icon: String
    let value: String
    let label: String

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(.blue)
            Text(value)
                .font(.headline)
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(.secondarySystemBackground))
        )
    }
}

struct PresetButton: View {
    let label: String
    let value: Int
    let current: Int
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(label)
                .font(.subheadline.bold())
                .frame(maxWidth: .infinity)
                .padding(.vertical, 10)
                .background(current == value ? Color.blue : Color(.secondarySystemBackground))
                .foregroundColor(current == value ? .white : .primary)
                .cornerRadius(8)
        }
    }
}

struct ChargingSessionRow: View {
    let session: ChargingManager.ChargingSession

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(session.startTime, style: .date)
                    .font(.subheadline.bold())
                Text("\(session.startLevel)% → \(session.endLevel)%")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 4) {
                Text("\(session.durationFormatted)")
                    .font(.subheadline)
                Text("\(String(format: "%.1f", session.energyAdded)) kWh")
                    .font(.caption)
                    .foregroundColor(.secondary)
                if let cost = session.cost {
                    Text("€\(String(format: "%.2f", cost))")
                        .font(.caption)
                        .foregroundColor(.green)
                }
            }
        }
        .padding()
        .background(Color(.secondarySystemBackground))
        .cornerRadius(12)
    }
}

// MARK: - Charging History View

struct ChargingHistoryView: View {
    @Environment(\.dismiss) var dismiss
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 15) {
                    // Weekly stats
                    VStack(spacing: 15) {
                        HStack {
                            StatCard(
                                title: "Total Energy",
                                value: "\(String(format: "%.1f", chargingManager.totalEnergyThisWeek)) kWh",
                                icon: "bolt.fill",
                                color: .blue
                            )
                            StatCard(
                                title: "Total Cost",
                                value: "€\(String(format: "%.2f", chargingManager.totalCostThisWeek))",
                                icon: "eurosign.circle.fill",
                                color: .green
                            )
                        }

                        StatCard(
                            title: "Average Session",
                            value: formatDuration(chargingManager.averageChargingTime),
                            icon: "clock.fill",
                            color: .purple
                        )
                    }
                    .padding()

                    Divider()

                    // All sessions
                    VStack(alignment: .leading, spacing: 10) {
                        Text("All Sessions")
                            .font(.headline)
                            .padding(.horizontal)

                        ForEach(chargingManager.chargingSessions) { session in
                            ChargingSessionRow(session: session)
                                .padding(.horizontal)
                        }
                    }
                }
            }
            .navigationTitle("Charging History")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }

    private func formatDuration(_ duration: TimeInterval) -> String {
        let hours = Int(duration) / 3600
        let minutes = Int(duration) % 3600 / 60
        return "\(hours)h \(minutes)min"
    }
}

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title)
                .foregroundColor(color)
            Text(value)
                .font(.title2.bold())
            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(Color(.secondarySystemBackground))
        .cornerRadius(12)
    }
}

#Preview {
    ChargingView()
        .environmentObject(ChargingManager())
        .environmentObject(VehicleManager())
}

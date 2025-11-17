import SwiftUI

/// App settings and configuration
/// Features:
/// - Vehicle configuration
/// - API and server settings
/// - Notification preferences
/// - Alert customization
/// - App preferences
/// - About and help
struct SettingsView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @EnvironmentObject var chargingManager: ChargingManager
    @EnvironmentObject var notificationManager: NotificationManager

    @State private var apiEndpoint = "http://localhost:3000"
    @State private var vehicleName = "My Toyota bZ4X"
    @State private var showingAbout = false

    var body: some View {
        NavigationView {
            Form {
                // Vehicle Section
                vehicleSection

                // API Configuration
                apiSection

                // Notifications
                notificationsSection

                // App Preferences
                preferencesSection

                // About
                aboutSection
            }
            .navigationTitle("Settings")
            .sheet(isPresented: $showingAbout) {
                AboutView()
            }
        }
    }

    // MARK: - Vehicle Section

    private var vehicleSection: some View {
        Section("Vehicle") {
            HStack {
                Text("Name")
                Spacer()
                TextField("Vehicle Name", text: $vehicleName)
                    .multilineTextAlignment(.trailing)
            }

            NavigationLink {
                VehicleDetailsView()
                    .environmentObject(vehicleManager)
            } label: {
                HStack {
                    VStack(alignment: .leading) {
                        Text("Vehicle Details")
                        Text(vehicleManager.vin)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }

            HStack {
                Text("Battery Health")
                Spacer()
                Text("\(vehicleManager.batteryHealth)%")
                    .foregroundColor(.green)
            }

            HStack {
                Text("Last Updated")
                Spacer()
                Text(vehicleManager.lastUpdated, style: .relative)
                    .foregroundColor(.secondary)
            }
        }
    }

    // MARK: - API Section

    private var apiSection: some View {
        Section("Server Configuration") {
            VStack(alignment: .leading, spacing: 8) {
                Text("API Endpoint")
                    .font(.caption)
                    .foregroundColor(.secondary)
                TextField("https://api.example.com", text: $apiEndpoint)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .font(.system(.body, design: .monospaced))
            }

            Button("Test Connection") {
                testApiConnection()
            }

            Toggle("Auto-sync", isOn: .constant(true))

            HStack {
                Text("Sync Interval")
                Spacer()
                Text("5 minutes")
                    .foregroundColor(.secondary)
            }
        }
    }

    // MARK: - Notifications Section

    private var notificationsSection: some View {
        Section("Notifications") {
            NavigationLink {
                NotificationSettingsView()
                    .environmentObject(notificationManager)
                    .environmentObject(chargingManager)
            } label: {
                HStack {
                    Image(systemName: "bell.fill")
                        .foregroundColor(.red)
                    Text("Notification Settings")
                }
            }

            Toggle("Charging Alerts", isOn: $notificationManager.enableChargingAlerts)
                .onChange(of: notificationManager.enableChargingAlerts) { _, _ in
                    notificationManager.savePreferences()
                }

            Toggle("Battery Health Alerts", isOn: $notificationManager.enableBatteryHealthAlerts)
                .onChange(of: notificationManager.enableBatteryHealthAlerts) { _, _ in
                    notificationManager.savePreferences()
                }

            Toggle("Trip Reminders", isOn: $notificationManager.enableTripReminders)
                .onChange(of: notificationManager.enableTripReminders) { _, _ in
                    notificationManager.savePreferences()
                }

            NavigationLink {
                QuietHoursView()
                    .environmentObject(notificationManager)
            } label: {
                HStack {
                    Image(systemName: "moon.fill")
                        .foregroundColor(.purple)
                    Text("Quiet Hours")
                    Spacer()
                    if notificationManager.quietHoursEnabled {
                        Text("On")
                            .foregroundColor(.secondary)
                    }
                }
            }
        }
    }

    // MARK: - Preferences Section

    private var preferencesSection: some View {
        Section("App Preferences") {
            Toggle("Use Metric Units", isOn: .constant(true))

            Picker("Temperature Unit", selection: .constant("Celsius")) {
                Text("Celsius").tag("Celsius")
                Text("Fahrenheit").tag("Fahrenheit")
            }

            Picker("Currency", selection: .constant("EUR")) {
                Text("EUR (€)").tag("EUR")
                Text("USD ($)").tag("USD")
                Text("GBP (£)").tag("GBP")
            }

            Toggle("Show Battery Percentage", isOn: .constant(true))

            Toggle("Haptic Feedback", isOn: .constant(true))
        }
    }

    // MARK: - About Section

    private var aboutSection: some View {
        Section("About") {
            Button {
                showingAbout = true
            } label: {
                HStack {
                    Image(systemName: "info.circle")
                    Text("About MyT2ABRP")
                    Spacer()
                    Text("v1.0.0")
                        .foregroundColor(.secondary)
                }
            }

            Link(destination: URL(string: "https://github.com/yourusername/myt2abrp")!) {
                HStack {
                    Image(systemName: "link")
                    Text("GitHub Repository")
                }
            }

            Button {
                // Open privacy policy
            } label: {
                HStack {
                    Image(systemName: "hand.raised")
                    Text("Privacy Policy")
                }
            }

            Button {
                // Open help
            } label: {
                HStack {
                    Image(systemName: "questionmark.circle")
                    Text("Help & Support")
                }
            }
        }

        Section {
            Button("Clear Cache") {
                clearCache()
            }
            .foregroundColor(.orange)

            Button("Reset All Settings") {
                resetSettings()
            }
            .foregroundColor(.red)
        }
    }

    // MARK: - Methods

    private func testApiConnection() {
        // Test API connection
        print("Testing connection to: \(apiEndpoint)")
    }

    private func clearCache() {
        // Clear cached data
        print("Clearing cache...")
    }

    private func resetSettings() {
        // Reset all settings to defaults
        print("Resetting settings...")
    }
}

// MARK: - Vehicle Details View

struct VehicleDetailsView: View {
    @EnvironmentObject var vehicleManager: VehicleManager

    var body: some View {
        Form {
            Section("Identification") {
                LabeledRow(label: "VIN", value: vehicleManager.vin)
                LabeledRow(label: "Model", value: "Toyota bZ4X")
                LabeledRow(label: "Year", value: "2024")
                LabeledRow(label: "Color", value: "Platinum White")
            }

            Section("Battery") {
                LabeledRow(label: "Capacity", value: "71.4 kWh")
                LabeledRow(label: "Type", value: "Lithium-ion")
                LabeledRow(label: "Health", value: "\(vehicleManager.batteryHealth)%")
                LabeledRow(label: "Temperature", value: "\(String(format: "%.1f", vehicleManager.batteryTemp))°C")
            }

            Section("Performance") {
                LabeledRow(label: "Range (WLTP)", value: "450 km")
                LabeledRow(label: "Power", value: "150 kW (204 hp)")
                LabeledRow(label: "Top Speed", value: "160 km/h")
                LabeledRow(label: "0-100 km/h", value: "7.5 s")
            }

            Section("Charging") {
                LabeledRow(label: "AC Max Power", value: "11 kW")
                LabeledRow(label: "DC Max Power", value: "150 kW")
                LabeledRow(label: "Charge Port", value: "CCS Combo 2")
            }
        }
        .navigationTitle("Vehicle Details")
        .navigationBarTitleDisplayMode(.inline)
    }
}

// MARK: - Notification Settings View

struct NotificationSettingsView: View {
    @EnvironmentObject var notificationManager: NotificationManager
    @EnvironmentObject var chargingManager: ChargingManager

    var body: some View {
        Form {
            Section("Charging Alerts") {
                Toggle("Optimal Charge (80%)", isOn: $chargingManager.alertAt80Percent)
                    .tint(.green)

                Toggle("Full Charge (100%)", isOn: $chargingManager.alertAtFullCharge)
                    .tint(.blue)

                Toggle("Custom Level", isOn: $chargingManager.customAlertEnabled)
                    .tint(.purple)

                if chargingManager.customAlertEnabled {
                    HStack {
                        Text("Alert Level")
                        Spacer()
                        Text("\(chargingManager.customAlertLevel)%")
                            .foregroundColor(.purple)
                    }
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

                Toggle("Low Battery Warning", isOn: $chargingManager.lowBatteryAlert)
                    .tint(.orange)

                Toggle("Slow Charging Alert", isOn: $chargingManager.slowChargingAlert)
                    .tint(.gray)
            }

            Section("Other Alerts") {
                Toggle("Battery Health Changes", isOn: $notificationManager.enableBatteryHealthAlerts)
                Toggle("Trip Reminders", isOn: $notificationManager.enableTripReminders)
            }

            Section("Notification Style") {
                Picker("Sound", selection: .constant("Default")) {
                    Text("Default").tag("Default")
                    Text("Charging").tag("Charging")
                    Text("Silent").tag("Silent")
                }

                Toggle("Vibration", isOn: .constant(true))
                Toggle("Show on Lock Screen", isOn: .constant(true))
            }

            Section("Notification History") {
                NavigationLink {
                    NotificationHistoryView()
                        .environmentObject(notificationManager)
                } label: {
                    HStack {
                        Text("View History")
                        Spacer()
                        Text("\(notificationManager.recentNotifications.count)")
                            .foregroundColor(.secondary)
                    }
                }

                Button("Clear All Notifications") {
                    notificationManager.clearDeliveredNotifications()
                }
                .foregroundColor(.red)
            }
        }
        .navigationTitle("Notifications")
        .navigationBarTitleDisplayMode(.inline)
    }
}

// MARK: - Quiet Hours View

struct QuietHoursView: View {
    @EnvironmentObject var notificationManager: NotificationManager

    var body: some View {
        Form {
            Section {
                Toggle("Enable Quiet Hours", isOn: $notificationManager.quietHoursEnabled)
                    .onChange(of: notificationManager.quietHoursEnabled) { _, _ in
                        notificationManager.savePreferences()
                    }
            } footer: {
                Text("Suppress non-critical notifications during quiet hours")
            }

            if notificationManager.quietHoursEnabled {
                Section("Schedule") {
                    DatePicker(
                        "Start Time",
                        selection: $notificationManager.quietHoursStart,
                        displayedComponents: .hourAndMinute
                    )

                    DatePicker(
                        "End Time",
                        selection: $notificationManager.quietHoursEnd,
                        displayedComponents: .hourAndMinute
                    )
                }

                Section {
                    Toggle("Allow Critical Alerts", isOn: .constant(true))
                } footer: {
                    Text("Critical alerts like low battery warnings will still notify you")
                }
            }
        }
        .navigationTitle("Quiet Hours")
        .navigationBarTitleDisplayMode(.inline)
    }
}

// MARK: - Notification History View

struct NotificationHistoryView: View {
    @EnvironmentObject var notificationManager: NotificationManager

    var body: some View {
        List {
            if notificationManager.recentNotifications.isEmpty {
                ContentUnavailableView(
                    "No Notifications",
                    systemImage: "bell.slash",
                    description: Text("You haven't received any notifications yet")
                )
            } else {
                ForEach(notificationManager.recentNotifications) { notification in
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text(notification.title)
                                .font(.headline)
                            Spacer()
                            Text(notification.timestamp, style: .relative)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        Text(notification.message)
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        HStack {
                            typeIcon(for: notification.type)
                            Text(typeName(for: notification.type))
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
        }
        .navigationTitle("Notification History")
        .navigationBarTitleDisplayMode(.inline)
    }

    private func typeIcon(for type: NotificationManager.NotificationType) -> some View {
        let icon: String
        let color: Color

        switch type {
        case .chargingComplete:
            icon = "bolt.fill"
            color = .blue
        case .optimalCharge:
            icon = "leaf.fill"
            color = .green
        case .customLevel:
            icon = "target"
            color = .purple
        case .lowBattery:
            icon = "exclamationmark.triangle.fill"
            color = .orange
        case .slowCharging:
            icon = "tortoise.fill"
            color = .gray
        case .batteryHealth:
            icon = "heart.fill"
            color = .red
        case .tripReminder:
            icon = "map.fill"
            color = .blue
        case .general:
            icon = "bell.fill"
            color = .gray
        }

        return Image(systemName: icon)
            .foregroundColor(color)
    }

    private func typeName(for type: NotificationManager.NotificationType) -> String {
        switch type {
        case .chargingComplete: return "Charging Complete"
        case .optimalCharge: return "Optimal Charge"
        case .customLevel: return "Custom Level"
        case .lowBattery: return "Low Battery"
        case .slowCharging: return "Slow Charging"
        case .batteryHealth: return "Battery Health"
        case .tripReminder: return "Trip Reminder"
        case .general: return "General"
        }
    }
}

// MARK: - About View

struct AboutView: View {
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 30) {
                    // App Icon
                    Image(systemName: "bolt.car.fill")
                        .font(.system(size: 100))
                        .foregroundColor(.red)
                        .padding(.top, 40)

                    VStack(spacing: 10) {
                        Text("MyT2ABRP")
                            .font(.title.bold())

                        Text("Version 1.0.0")
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        Text("Build 2024.11.17")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }

                    // Description
                    VStack(alignment: .leading, spacing: 15) {
                        Text("A Better Route Planner for Toyota bZ4X")
                            .font(.headline)

                        Text("Smart charging management, battery health tracking, and intelligent route planning for your electric Toyota vehicle.")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal)

                    // Features
                    VStack(alignment: .leading, spacing: 15) {
                        Text("Innovative Features")
                            .font(.headline)
                            .padding(.horizontal)

                        FeatureRow(icon: "target", title: "Custom Charge Alerts", description: "Set your own charge level notifications")
                        FeatureRow(icon: "leaf.fill", title: "80% Optimal Alert", description: "Maximize battery longevity")
                        FeatureRow(icon: "heart.fill", title: "Battery Health Tracking", description: "Monitor degradation over time")
                        FeatureRow(icon: "map.fill", title: "Route Planning", description: "ABRP-style intelligent routing")
                        FeatureRow(icon: "chart.bar.fill", title: "Analytics", description: "Comprehensive usage insights")
                    }
                    .padding(.vertical)

                    // Credits
                    VStack(spacing: 10) {
                        Text("Built with ❤️ for Toyota bZ4X owners")
                            .font(.caption)
                            .foregroundColor(.secondary)

                        Text("© 2024 MyT2ABRP. All rights reserved.")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    .padding(.bottom, 40)
                }
            }
            .navigationTitle("About")
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
}

struct FeatureRow: View {
    let icon: String
    let title: String
    let description: String

    var body: some View {
        HStack(spacing: 15) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(.blue)
                .frame(width: 40)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.bold())
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.horizontal)
    }
}

// MARK: - Helper Views

struct LabeledRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
            Spacer()
            Text(value)
                .foregroundColor(.secondary)
        }
    }
}

#Preview {
    SettingsView()
        .environmentObject(VehicleManager())
        .environmentObject(ChargingManager())
        .environmentObject(NotificationManager())
}

import Foundation
import Combine

/// Manages charging state and smart charging features
/// INNOVATIVE FEATURES:
/// - Custom charge level alerts (user-defined %)
/// - Optimal charge notification (80% for battery longevity)
/// - Charging complete alerts
/// - Slow charging detection
/// - Ready-for-trip notifications
class ChargingManager: ObservableObject {
    // MARK: - Published Properties

    @Published var isCharging: Bool = false
    @Published var currentLevel: Int = 0
    @Published var targetLevel: Int = 100
    @Published var powerKW: Double = 0.0
    @Published var timeRemaining: String = "Calculating..."
    @Published var energyAdded: Double = 0.0
    @Published var chargeRate: Double = 0.0
    @Published var estimatedFullTime: Date?
    @Published var sessionStartTime: Date?
    @Published var sessionStartLevel: Int = 0

    // Alert Configuration - INNOVATIVE FEATURES
    @Published var alertAt80Percent: Bool = true      // Optimal charge alert
    @Published var alertAtFullCharge: Bool = true     // 100% complete alert
    @Published var customAlertLevel: Int = 90         // User-defined alert level
    @Published var customAlertEnabled: Bool = false
    @Published var lowBatteryAlert: Bool = true       // Alert when below 20%
    @Published var slowChargingAlert: Bool = true     // Alert if charging slower than expected
    @Published var readyForTripAlert: Bool = false    // Alert when ready for planned trip

    // Charging History
    @Published var chargingSessions: [ChargingSession] = []
    @Published var totalEnergyThisWeek: Double = 0.0
    @Published var totalCostThisWeek: Double = 0.0
    @Published var averageChargingTime: TimeInterval = 0

    // Alert State Tracking
    private var hasAlerted80Percent = false
    private var hasAlertedCustomLevel = false
    private var hasAlertedFullCharge = false

    // MARK: - Configuration

    private let apiBaseURL = "http://localhost:3000"
    private var cancellables = Set<AnyCancellable>()
    private var monitoringTimer: Timer?

    // MARK: - Types

    struct ChargingSession: Identifiable, Codable {
        let id: UUID
        let startTime: Date
        let endTime: Date
        let startLevel: Int
        let endLevel: Int
        let energyAdded: Double
        let averagePower: Double
        let cost: Double?

        var duration: TimeInterval {
            endTime.timeIntervalSince(startTime)
        }

        var durationFormatted: String {
            let hours = Int(duration) / 3600
            let minutes = Int(duration) % 3600 / 60
            return "\(hours)h \(minutes)min"
        }
    }

    // MARK: - Initialization

    init() {
        loadCachedData()
        loadChargingHistory()
    }

    // MARK: - Public Methods

    /// Start monitoring charging status
    func startMonitoring() {
        print("🔌 Starting charging monitor...")

        // Monitor every 30 seconds
        monitoringTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            Task {
                await self?.updateStatus()
            }
        }

        // Initial update
        Task {
            await updateStatus()
        }
    }

    /// Stop monitoring
    func stopMonitoring() {
        monitoringTimer?.invalidate()
        monitoringTimer = nil
        print("⏹️ Stopped charging monitor")
    }

    /// Update charging status from API
    @MainActor
    func updateStatus() async {
        do {
            // Simulate API call - replace with actual API
            try await Task.sleep(nanoseconds: 500_000_000) // 0.5 second

            // Mock data - in production, fetch from Spin server
            let wasCharging = isCharging
            let previousLevel = currentLevel

            // Simulate charging progression
            if isCharging && currentLevel < targetLevel {
                currentLevel = min(currentLevel + 1, targetLevel)
                energyAdded += 0.5
                powerKW = Double.random(in: 45...52)
                chargeRate = powerKW

                // Update time remaining
                if currentLevel < targetLevel {
                    let percentRemaining = targetLevel - currentLevel
                    let minutesRemaining = Int(Double(percentRemaining) * 1.5)
                    timeRemaining = formatTimeRemaining(minutes: minutesRemaining)
                    estimatedFullTime = Date().addingTimeInterval(TimeInterval(minutesRemaining * 60))
                } else {
                    timeRemaining = "Complete"
                    estimatedFullTime = Date()
                }
            } else {
                powerKW = 0.0
                chargeRate = 0.0
                timeRemaining = "Not charging"
            }

            // Check for alert conditions - INNOVATIVE FEATURE
            await checkChargingAlerts(previousLevel: previousLevel, currentLevel: currentLevel)

            // Detect slow charging - INNOVATIVE FEATURE
            if isCharging && powerKW < 20.0 && slowChargingAlert {
                await sendSlowChargingAlert()
            }

            saveCachedData()

        } catch {
            print("❌ Failed to update charging status: \(error)")
        }
    }

    /// Toggle charging on/off
    func toggleCharging() {
        if isCharging {
            stopCharging()
        } else {
            startCharging()
        }
    }

    /// Start charging
    func startCharging() {
        print("⚡ Starting charging...")
        isCharging = true
        sessionStartTime = Date()
        sessionStartLevel = currentLevel
        hasAlerted80Percent = false
        hasAlertedCustomLevel = false
        hasAlertedFullCharge = false

        Task {
            await updateStatus()
        }

        // TODO: Send command to vehicle via API
    }

    /// Stop charging
    func stopCharging() {
        print("⏹️ Stopping charging...")

        // Record session
        if let startTime = sessionStartTime {
            let session = ChargingSession(
                id: UUID(),
                startTime: startTime,
                endTime: Date(),
                startLevel: sessionStartLevel,
                endLevel: currentLevel,
                energyAdded: energyAdded,
                averagePower: powerKW,
                cost: calculateSessionCost()
            )
            chargingSessions.insert(session, at: 0)
            saveChargingHistory()
        }

        isCharging = false
        powerKW = 0.0
        timeRemaining = "Not charging"
        sessionStartTime = nil

        // TODO: Send command to vehicle via API
    }

    /// Pre-condition vehicle (heat/cool cabin)
    func precondition() {
        print("❄️ Starting pre-conditioning...")
        // TODO: Send pre-condition command to vehicle
    }

    /// Set custom charge target
    func setChargeTarget(_ target: Int) {
        targetLevel = max(50, min(100, target))
        print("🎯 Charge target set to \(targetLevel)%")
        saveCachedData()
    }

    // MARK: - Smart Alerts - INNOVATIVE FEATURES

    @MainActor
    private func checkChargingAlerts(previousLevel: Int, currentLevel: Int) async {
        // 80% Optimal Charge Alert
        if alertAt80Percent && !hasAlerted80Percent && currentLevel >= 80 && previousLevel < 80 {
            hasAlerted80Percent = true
            NotificationCenter.default.post(
                name: .chargingAlertTriggered,
                object: ChargingAlert(
                    type: .optimalCharge,
                    level: 80,
                    message: "Battery at 80% - optimal for battery longevity"
                )
            )
            print("🔔 80% optimal charge alert triggered")
        }

        // Custom Level Alert
        if customAlertEnabled && !hasAlertedCustomLevel && currentLevel >= customAlertLevel && previousLevel < customAlertLevel {
            hasAlertedCustomLevel = true
            NotificationCenter.default.post(
                name: .chargingAlertTriggered,
                object: ChargingAlert(
                    type: .customLevel,
                    level: customAlertLevel,
                    message: "Battery reached your custom level of \(customAlertLevel)%"
                )
            )
            print("🔔 Custom level \(customAlertLevel)% alert triggered")
        }

        // 100% Full Charge Alert
        if alertAtFullCharge && !hasAlertedFullCharge && currentLevel >= 100 && previousLevel < 100 {
            hasAlertedFullCharge = true
            NotificationCenter.default.post(
                name: .chargingAlertTriggered,
                object: ChargingAlert(
                    type: .chargeComplete,
                    level: 100,
                    message: "Your vehicle is fully charged and ready to go!"
                )
            )
            print("🔔 100% charge complete alert triggered")
        }

        // Low Battery Alert
        if lowBatteryAlert && currentLevel <= 20 && previousLevel > 20 {
            NotificationCenter.default.post(
                name: .chargingAlertTriggered,
                object: ChargingAlert(
                    type: .lowBattery,
                    level: currentLevel,
                    message: "Battery low at \(currentLevel)%. Consider charging soon."
                )
            )
            print("🔔 Low battery alert triggered")
        }
    }

    @MainActor
    private func sendSlowChargingAlert() async {
        NotificationCenter.default.post(
            name: .chargingAlertTriggered,
            object: ChargingAlert(
                type: .slowCharging,
                level: currentLevel,
                message: "Charging slower than expected (\(String(format: "%.1f", powerKW)) kW)"
            )
        )
    }

    // MARK: - Analytics

    private func calculateSessionCost() -> Double {
        // Assume €0.173 per kWh average
        return energyAdded * 0.173
    }

    func calculateWeeklyStats() {
        let oneWeekAgo = Date().addingTimeInterval(-7 * 24 * 3600)
        let recentSessions = chargingSessions.filter { $0.startTime > oneWeekAgo }

        totalEnergyThisWeek = recentSessions.reduce(0) { $0 + $1.energyAdded }
        totalCostThisWeek = recentSessions.reduce(0) { $0 + ($1.cost ?? 0) }

        if !recentSessions.isEmpty {
            averageChargingTime = recentSessions.reduce(0) { $0 + $1.duration } / Double(recentSessions.count)
        }
    }

    // MARK: - Helpers

    private func formatTimeRemaining(minutes: Int) -> String {
        if minutes < 60 {
            return "\(minutes) min"
        } else {
            let hours = minutes / 60
            let mins = minutes % 60
            return "\(hours)h \(mins)min"
        }
    }

    // MARK: - Data Persistence

    private func loadCachedData() {
        let defaults = UserDefaults.standard

        isCharging = defaults.bool(forKey: "charging_is_charging")
        currentLevel = defaults.integer(forKey: "charging_current_level")
        targetLevel = defaults.integer(forKey: "charging_target_level")
        powerKW = defaults.double(forKey: "charging_power_kw")
        energyAdded = defaults.double(forKey: "charging_energy_added")

        // Alert preferences
        alertAt80Percent = defaults.object(forKey: "alert_80_percent") as? Bool ?? true
        alertAtFullCharge = defaults.object(forKey: "alert_full_charge") as? Bool ?? true
        customAlertLevel = defaults.integer(forKey: "alert_custom_level")
        customAlertEnabled = defaults.bool(forKey: "alert_custom_enabled")
        lowBatteryAlert = defaults.object(forKey: "alert_low_battery") as? Bool ?? true
        slowChargingAlert = defaults.object(forKey: "alert_slow_charging") as? Bool ?? true

        if customAlertLevel == 0 {
            customAlertLevel = 90  // Default
        }

        if targetLevel == 0 {
            targetLevel = 100  // Default
            currentLevel = 45   // Demo data
        }

        print("📦 Loaded charging preferences")
    }

    private func saveCachedData() {
        let defaults = UserDefaults.standard
        defaults.set(isCharging, forKey: "charging_is_charging")
        defaults.set(currentLevel, forKey: "charging_current_level")
        defaults.set(targetLevel, forKey: "charging_target_level")
        defaults.set(powerKW, forKey: "charging_power_kw")
        defaults.set(energyAdded, forKey: "charging_energy_added")

        defaults.set(alertAt80Percent, forKey: "alert_80_percent")
        defaults.set(alertAtFullCharge, forKey: "alert_full_charge")
        defaults.set(customAlertLevel, forKey: "alert_custom_level")
        defaults.set(customAlertEnabled, forKey: "alert_custom_enabled")
        defaults.set(lowBatteryAlert, forKey: "alert_low_battery")
        defaults.set(slowChargingAlert, forKey: "alert_slow_charging")
    }

    private func loadChargingHistory() {
        if let data = UserDefaults.standard.data(forKey: "charging_sessions"),
           let sessions = try? JSONDecoder().decode([ChargingSession].self, from: data) {
            chargingSessions = sessions
            calculateWeeklyStats()
            print("📦 Loaded \(sessions.count) charging sessions")
        } else {
            // Add demo sessions
            addDemoSessions()
        }
    }

    private func saveChargingHistory() {
        if let data = try? JSONEncoder().encode(chargingSessions) {
            UserDefaults.standard.set(data, forKey: "charging_sessions")
        }
        calculateWeeklyStats()
    }

    private func addDemoSessions() {
        let now = Date()
        let demoSessions = [
            ChargingSession(
                id: UUID(),
                startTime: now.addingTimeInterval(-3 * 3600),
                endTime: now.addingTimeInterval(-1 * 3600),
                startLevel: 45,
                endLevel: 100,
                energyAdded: 35.5,
                averagePower: 50.0,
                cost: 6.14
            ),
            ChargingSession(
                id: UUID(),
                startTime: now.addingTimeInterval(-27 * 3600),
                endTime: now.addingTimeInterval(-25.5 * 3600),
                startLevel: 20,
                endLevel: 80,
                energyAdded: 42.0,
                averagePower: 48.5,
                cost: 7.27
            ),
        ]
        chargingSessions = demoSessions
        calculateWeeklyStats()
    }
}

// MARK: - Charging Alert Types

struct ChargingAlert {
    enum AlertType {
        case optimalCharge      // 80% reached
        case chargeComplete     // 100% reached
        case customLevel        // User-defined level reached
        case lowBattery        // Below 20%
        case slowCharging      // Charging slower than expected
        case readyForTrip      // Ready for planned trip
    }

    let type: AlertType
    let level: Int
    let message: String
}

// MARK: - Notification Names

extension Notification.Name {
    static let chargingAlertTriggered = Notification.Name("chargingAlertTriggered")
}

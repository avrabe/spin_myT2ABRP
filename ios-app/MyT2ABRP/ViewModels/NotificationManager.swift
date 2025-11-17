import Foundation
import UserNotifications
import Combine

/// Manages all user notifications including smart charging alerts
/// INNOVATIVE FEATURES:
/// - Smart charging notifications (80%, 100%, custom levels)
/// - Battery health alerts
/// - Ready-for-trip notifications
/// - Charging complete with actionable suggestions
/// - Time-based smart reminders
class NotificationManager: ObservableObject {
    // MARK: - Published Properties

    @Published var isAuthorized: Bool = false
    @Published var recentNotifications: [NotificationRecord] = []

    // Notification Preferences
    @Published var enableChargingAlerts: Bool = true
    @Published var enableBatteryHealthAlerts: Bool = true
    @Published var enableTripReminders: Bool = true
    @Published var quietHoursEnabled: Bool = false
    @Published var quietHoursStart: Date = Calendar.current.date(from: DateComponents(hour: 22, minute: 0))!
    @Published var quietHoursEnd: Date = Calendar.current.date(from: DateComponents(hour: 7, minute: 0))!

    // MARK: - Types

    struct NotificationRecord: Identifiable {
        let id: UUID
        let title: String
        let message: String
        let timestamp: Date
        let type: NotificationType
        let wasDelivered: Bool
    }

    enum NotificationType {
        case chargingComplete
        case optimalCharge
        case customLevel
        case lowBattery
        case slowCharging
        case batteryHealth
        case tripReminder
        case general
    }

    // MARK: - Private Properties

    private let notificationCenter = UNUserNotificationCenter.current()
    private var cancellables = Set<AnyCancellable>()

    // MARK: - Initialization

    init() {
        checkAuthorizationStatus()
        setupChargingAlertObserver()
        loadPreferences()
    }

    // MARK: - Authorization

    func requestAuthorization() async -> Bool {
        do {
            let granted = try await notificationCenter.requestAuthorization(options: [.alert, .badge, .sound, .criticalAlert])
            await MainActor.run {
                self.isAuthorized = granted
            }
            print(granted ? "✅ Notification permission granted" : "❌ Notification permission denied")
            return granted
        } catch {
            print("❌ Error requesting notification permission: \(error)")
            return false
        }
    }

    private func checkAuthorizationStatus() {
        notificationCenter.getNotificationSettings { [weak self] settings in
            DispatchQueue.main.async {
                self?.isAuthorized = settings.authorizationStatus == .authorized
            }
        }
    }

    // MARK: - Smart Charging Alerts Observer

    private func setupChargingAlertObserver() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleChargingAlert(_:)),
            name: .chargingAlertTriggered,
            object: nil
        )
    }

    @objc private func handleChargingAlert(_ notification: Notification) {
        guard enableChargingAlerts else { return }
        guard let alert = notification.object as? ChargingAlert else { return }

        Task {
            await sendChargingNotification(alert)
        }
    }

    // MARK: - Notification Sending

    /// Send charging-related notification
    @MainActor
    private func sendChargingNotification(_ alert: ChargingAlert) async {
        // Check quiet hours
        if isInQuietHours() {
            print("🔇 Suppressing notification during quiet hours")
            return
        }

        let content = UNMutableNotificationContent()

        switch alert.type {
        case .optimalCharge:
            content.title = "🟢 Optimal Charge Reached"
            content.body = "Battery at 80% - perfect for battery longevity. You can stop charging now."
            content.sound = .default
            content.categoryIdentifier = "CHARGING_OPTIMAL"

        case .chargeComplete:
            content.title = "⚡ Charging Complete"
            content.body = "Your vehicle is fully charged and ready to go!"
            content.sound = .default
            content.categoryIdentifier = "CHARGING_COMPLETE"

        case .customLevel:
            content.title = "🎯 Custom Level Reached"
            content.body = "Battery reached your target of \(alert.level)%"
            content.sound = .default
            content.categoryIdentifier = "CHARGING_CUSTOM"

        case .lowBattery:
            content.title = "🔋 Low Battery Warning"
            content.body = "Battery at \(alert.level)%. Consider charging soon."
            content.sound = .defaultCritical  // Critical alert
            content.categoryIdentifier = "BATTERY_LOW"

        case .slowCharging:
            content.title = "⚠️ Slow Charging Detected"
            content.body = alert.message
            content.sound = .default
            content.categoryIdentifier = "CHARGING_SLOW"

        case .readyForTrip:
            content.title = "✅ Ready for Your Trip"
            content.body = "Vehicle is charged and ready for your planned journey"
            content.sound = .default
            content.categoryIdentifier = "TRIP_READY"
        }

        // Add actions
        content = addNotificationActions(to: content, for: alert.type)

        // Create trigger (immediate delivery)
        let trigger = UNTimeIntervalNotificationTrigger(timeInterval: 1, repeats: false)

        // Create request
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: trigger
        )

        do {
            try await notificationCenter.add(request)
            print("📬 Sent notification: \(content.title)")

            // Record notification
            recordNotification(
                title: content.title,
                message: content.body,
                type: convertAlertType(alert.type)
            )

        } catch {
            print("❌ Failed to send notification: \(error)")
        }
    }

    /// Send battery health alert
    func sendBatteryHealthAlert(healthPercentage: Int, message: String) async {
        guard enableBatteryHealthAlerts else { return }
        guard isAuthorized else { return }

        let content = UNMutableNotificationContent()
        content.title = "❤️ Battery Health Update"
        content.body = message
        content.sound = .default

        let trigger = UNTimeIntervalNotificationTrigger(timeInterval: 1, repeats: false)
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: trigger
        )

        try? await notificationCenter.add(request)
        recordNotification(title: content.title, message: content.body, type: .batteryHealth)
    }

    /// Send trip reminder
    func sendTripReminder(tripName: String, departureTime: Date, batteryLevel: Int) async {
        guard enableTripReminders else { return }
        guard isAuthorized else { return }

        let content = UNMutableNotificationContent()
        content.title = "🗺️ Trip Reminder: \(tripName)"

        if batteryLevel >= 80 {
            content.body = "Your vehicle is ready! Battery at \(batteryLevel)%"
        } else {
            content.body = "Battery at \(batteryLevel)%. Consider charging before your trip."
        }

        content.sound = .default

        // Schedule for 1 hour before departure
        let triggerDate = departureTime.addingTimeInterval(-3600)
        let components = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: triggerDate)
        let trigger = UNCalendarNotificationTrigger(dateMatching: components, repeats: false)

        let request = UNNotificationRequest(
            identifier: "trip-\(tripName)",
            content: content,
            trigger: trigger
        )

        try? await notificationCenter.add(request)
        print("📅 Scheduled trip reminder for \(tripName)")
    }

    // MARK: - Notification Actions

    private func addNotificationActions(to content: UNMutableNotificationContent, for type: ChargingAlert.AlertType) -> UNMutableNotificationContent {
        switch type {
        case .optimalCharge:
            // Add "Stop Charging" action
            let stopAction = UNNotificationAction(
                identifier: "STOP_CHARGING",
                title: "Stop Charging",
                options: [.foreground]
            )
            let continueAction = UNNotificationAction(
                identifier: "CONTINUE_CHARGING",
                title: "Continue to 100%",
                options: []
            )
            registerCategory(identifier: "CHARGING_OPTIMAL", actions: [stopAction, continueAction])

        case .chargeComplete:
            let viewAction = UNNotificationAction(
                identifier: "VIEW_STATS",
                title: "View Stats",
                options: [.foreground]
            )
            registerCategory(identifier: "CHARGING_COMPLETE", actions: [viewAction])

        case .lowBattery:
            let navigateAction = UNNotificationAction(
                identifier: "FIND_CHARGER",
                title: "Find Charger",
                options: [.foreground]
            )
            registerCategory(identifier: "BATTERY_LOW", actions: [navigateAction])

        default:
            break
        }

        return content
    }

    private func registerCategory(identifier: String, actions: [UNNotificationAction]) {
        let category = UNNotificationCategory(
            identifier: identifier,
            actions: actions,
            intentIdentifiers: [],
            options: []
        )
        notificationCenter.setNotificationCategories([category])
    }

    // MARK: - Quiet Hours

    private func isInQuietHours() -> Bool {
        guard quietHoursEnabled else { return false }

        let now = Date()
        let calendar = Calendar.current

        let startComponents = calendar.dateComponents([.hour, .minute], from: quietHoursStart)
        let endComponents = calendar.dateComponents([.hour, .minute], from: quietHoursEnd)
        let nowComponents = calendar.dateComponents([.hour, .minute], from: now)

        let nowMinutes = (nowComponents.hour ?? 0) * 60 + (nowComponents.minute ?? 0)
        let startMinutes = (startComponents.hour ?? 0) * 60 + (startComponents.minute ?? 0)
        let endMinutes = (endComponents.hour ?? 0) * 60 + (endComponents.minute ?? 0)

        if startMinutes < endMinutes {
            // Quiet hours within same day (e.g., 22:00 - 23:00)
            return nowMinutes >= startMinutes && nowMinutes < endMinutes
        } else {
            // Quiet hours span midnight (e.g., 22:00 - 07:00)
            return nowMinutes >= startMinutes || nowMinutes < endMinutes
        }
    }

    // MARK: - Notification History

    private func recordNotification(title: String, message: String, type: NotificationType) {
        let record = NotificationRecord(
            id: UUID(),
            title: title,
            message: message,
            timestamp: Date(),
            type: type,
            wasDelivered: true
        )

        DispatchQueue.main.async {
            self.recentNotifications.insert(record, at: 0)

            // Keep only last 50 notifications
            if self.recentNotifications.count > 50 {
                self.recentNotifications = Array(self.recentNotifications.prefix(50))
            }

            self.saveNotificationHistory()
        }
    }

    private func convertAlertType(_ type: ChargingAlert.AlertType) -> NotificationType {
        switch type {
        case .chargeComplete: return .chargingComplete
        case .optimalCharge: return .optimalCharge
        case .customLevel: return .customLevel
        case .lowBattery: return .lowBattery
        case .slowCharging: return .slowCharging
        case .readyForTrip: return .tripReminder
        }
    }

    // MARK: - Data Persistence

    private func loadPreferences() {
        let defaults = UserDefaults.standard
        enableChargingAlerts = defaults.object(forKey: "notif_charging_alerts") as? Bool ?? true
        enableBatteryHealthAlerts = defaults.object(forKey: "notif_battery_health") as? Bool ?? true
        enableTripReminders = defaults.object(forKey: "notif_trip_reminders") as? Bool ?? true
        quietHoursEnabled = defaults.bool(forKey: "notif_quiet_hours_enabled")

        if let data = defaults.data(forKey: "notification_history"),
           let history = try? JSONDecoder().decode([NotificationRecord].self, from: data) {
            recentNotifications = history
        }
    }

    func savePreferences() {
        let defaults = UserDefaults.standard
        defaults.set(enableChargingAlerts, forKey: "notif_charging_alerts")
        defaults.set(enableBatteryHealthAlerts, forKey: "notif_battery_health")
        defaults.set(enableTripReminders, forKey: "notif_trip_reminders")
        defaults.set(quietHoursEnabled, forKey: "notif_quiet_hours_enabled")
    }

    private func saveNotificationHistory() {
        if let data = try? JSONEncoder().encode(recentNotifications) {
            UserDefaults.standard.set(data, forKey: "notification_history")
        }
    }

    // MARK: - Cleanup

    func clearAllPendingNotifications() {
        notificationCenter.removeAllPendingNotificationRequests()
        print("🗑️ Cleared all pending notifications")
    }

    func clearDeliveredNotifications() {
        notificationCenter.removeAllDeliveredNotifications()
        print("🗑️ Cleared all delivered notifications")
    }
}

// MARK: - NotificationRecord Codable

extension NotificationManager.NotificationRecord: Codable {
    enum CodingKeys: String, CodingKey {
        case id, title, message, timestamp, type, wasDelivered
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        title = try container.decode(String.self, forKey: .title)
        message = try container.decode(String.self, forKey: .message)
        timestamp = try container.decode(Date.self, forKey: .timestamp)
        type = try container.decode(NotificationManager.NotificationType.self, forKey: .type)
        wasDelivered = try container.decode(Bool.self, forKey: .wasDelivered)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(title, forKey: .title)
        try container.encode(message, forKey: .message)
        try container.encode(timestamp, forKey: .timestamp)
        try container.encode(type, forKey: .type)
        try container.encode(wasDelivered, forKey: .wasDelivered)
    }
}

extension NotificationManager.NotificationType: Codable {}

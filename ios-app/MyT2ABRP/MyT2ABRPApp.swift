import SwiftUI
import UserNotifications
import BackgroundTasks

@main
struct MyT2ABRPApp: App {
    @StateObject private var vehicleManager = VehicleManager()
    @StateObject private var chargingManager = ChargingManager()
    @StateObject private var notificationManager = NotificationManager()

    init() {
        // Request notification permissions
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .badge, .sound]) { granted, error in
            if granted {
                print("✅ Notification permission granted")
            }
        }

        // Register background tasks
        registerBackgroundTasks()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(vehicleManager)
                .environmentObject(chargingManager)
                .environmentObject(notificationManager)
                .onAppear {
                    // Start monitoring
                    chargingManager.startMonitoring()
                }
        }
    }

    private func registerBackgroundTasks() {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.toyota.myt2abrp.refresh",
            using: nil
        ) { task in
            handleBackgroundRefresh(task: task as! BGAppRefreshTask)
        }
    }

    private func handleBackgroundRefresh(task: BGAppRefreshTask) {
        // Fetch latest vehicle status
        Task {
            await chargingManager.updateStatus()
            task.setTaskCompleted(success: true)
        }

        // Schedule next refresh
        scheduleBackgroundRefresh()
    }

    private func scheduleBackgroundRefresh() {
        let request = BGAppRefreshTaskRequest(identifier: "com.toyota.myt2abrp.refresh")
        request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60) // 15 minutes

        try? BGTaskScheduler.shared.submit(request)
    }
}

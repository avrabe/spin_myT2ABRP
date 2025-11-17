import Foundation
import Combine

/// Manages vehicle state and data synchronization
class VehicleManager: ObservableObject {
    // MARK: - Published Properties

    @Published var vin: String = "LOADING..."
    @Published var batteryLevel: Int = 0
    @Published var rangeKm: Int = 0
    @Published var rangeAt80: Int = 0  // Optimal range for battery health
    @Published var batteryHealth: Int = 100
    @Published var batteryTemp: Double = 20.0
    @Published var lastUpdated: Date = Date()
    @Published var isConnected: Bool = false
    @Published var location: Location?

    // MARK: - Types

    struct Location {
        let latitude: Double
        let longitude: Double
    }

    // MARK: - Configuration

    private let apiBaseURL = "http://localhost:3000"  // Will be configurable
    private var cancellables = Set<AnyCancellable>()

    // MARK: - Initialization

    init() {
        loadCachedData()
        startAutoRefresh()
    }

    // MARK: - Public Methods

    /// Refresh vehicle data from API
    @MainActor
    func refresh() async {
        print("🔄 Refreshing vehicle data...")

        do {
            // Simulate API call for now
            // In production, this would call the Spin server API
            try await Task.sleep(nanoseconds: 1_000_000_000) // 1 second delay

            // Mock data - replace with actual API call
            await updateVehicleData(
                vin: "JN1AZ4EH5PM123456",
                batteryLevel: Int.random(in: 75...95),
                rangeKm: Int.random(in: 280...350),
                batteryHealth: Int.random(in: 96...100),
                batteryTemp: Double.random(in: 18...25),
                isConnected: true
            )

            print("✅ Vehicle data refreshed")
        } catch {
            print("❌ Failed to refresh vehicle data: \(error)")
        }
    }

    /// Update vehicle data
    @MainActor
    func updateVehicleData(
        vin: String,
        batteryLevel: Int,
        rangeKm: Int,
        batteryHealth: Int,
        batteryTemp: Double,
        isConnected: Bool
    ) async {
        self.vin = vin
        self.batteryLevel = batteryLevel
        self.rangeKm = rangeKm
        self.batteryHealth = batteryHealth
        self.batteryTemp = batteryTemp
        self.isConnected = isConnected
        self.lastUpdated = Date()

        // Calculate optimal range at 80% charge
        self.rangeAt80 = calculateRangeAt80Percent()

        // Cache the data
        saveCachedData()
    }

    /// Calculate estimated range at 80% battery (optimal for longevity)
    private func calculateRangeAt80Percent() -> Int {
        guard batteryLevel > 0 else { return 0 }
        let rangePerPercent = Double(rangeKm) / Double(batteryLevel)
        return Int(rangePerPercent * 80.0)
    }

    // MARK: - Auto Refresh

    private func startAutoRefresh() {
        // Auto-refresh every 5 minutes when app is active
        Timer.publish(every: 300, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                Task {
                    await self?.refresh()
                }
            }
            .store(in: &cancellables)
    }

    // MARK: - Data Persistence

    private func loadCachedData() {
        let defaults = UserDefaults.standard

        if let cachedVin = defaults.string(forKey: "vehicle_vin") {
            vin = cachedVin
            batteryLevel = defaults.integer(forKey: "vehicle_battery_level")
            rangeKm = defaults.integer(forKey: "vehicle_range_km")
            rangeAt80 = defaults.integer(forKey: "vehicle_range_at_80")
            batteryHealth = defaults.integer(forKey: "vehicle_battery_health")
            batteryTemp = defaults.double(forKey: "vehicle_battery_temp")
            isConnected = defaults.bool(forKey: "vehicle_is_connected")

            if let timestamp = defaults.object(forKey: "vehicle_last_updated") as? Date {
                lastUpdated = timestamp
            }

            print("📦 Loaded cached vehicle data")
        } else {
            // Set initial demo data if no cache
            vin = "Demo Vehicle"
            batteryLevel = 85
            rangeKm = 320
            rangeAt80 = 301
            batteryHealth = 98
            batteryTemp = 22.5
            isConnected = false
            print("🎭 Using demo vehicle data")
        }
    }

    private func saveCachedData() {
        let defaults = UserDefaults.standard
        defaults.set(vin, forKey: "vehicle_vin")
        defaults.set(batteryLevel, forKey: "vehicle_battery_level")
        defaults.set(rangeKm, forKey: "vehicle_range_km")
        defaults.set(rangeAt80, forKey: "vehicle_range_at_80")
        defaults.set(batteryHealth, forKey: "vehicle_battery_health")
        defaults.set(batteryTemp, forKey: "vehicle_battery_temp")
        defaults.set(isConnected, forKey: "vehicle_is_connected")
        defaults.set(lastUpdated, forKey: "vehicle_last_updated")
    }

    // MARK: - API Communication (Placeholder)

    private func fetchVehicleStatus() async throws -> VehicleStatusResponse {
        // TODO: Implement actual API call to Spin server
        // Example:
        // let url = URL(string: "\(apiBaseURL)/api/vehicle/status")!
        // let (data, _) = try await URLSession.shared.data(from: url)
        // return try JSONDecoder().decode(VehicleStatusResponse.self, from: data)

        throw NSError(domain: "VehicleManager", code: -1, userInfo: [
            NSLocalizedDescriptionKey: "API not yet implemented"
        ])
    }
}

// MARK: - API Response Models

struct VehicleStatusResponse: Codable {
    let vin: String
    let batteryLevel: Int
    let rangeKm: Int
    let isCharging: Bool
    let isConnected: Bool
    let location: LocationResponse?
    let batteryHealth: Int
    let batteryTemp: Double

    enum CodingKeys: String, CodingKey {
        case vin
        case batteryLevel = "battery_level"
        case rangeKm = "range_km"
        case isCharging = "is_charging"
        case isConnected = "is_connected"
        case location
        case batteryHealth = "battery_health"
        case batteryTemp = "battery_temp"
    }
}

struct LocationResponse: Codable {
    let latitude: Double
    let longitude: Double

    enum CodingKeys: String, CodingKey {
        case latitude = "lat"
        case longitude = "lon"
    }
}

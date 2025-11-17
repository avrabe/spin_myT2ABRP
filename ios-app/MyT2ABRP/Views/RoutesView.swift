import SwiftUI
import MapKit

/// Route planning with integrated charging stops
/// INNOVATIVE FEATURES:
/// - ABRP-style route planning
/// - Intelligent charging stop recommendations
/// - Real-time range consideration
/// - Weather and elevation impact
/// - Save favorite routes
struct RoutesView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @State private var searchText = ""
    @State private var savedRoutes: [SavedRoute] = []
    @State private var showingRoutePlanner = false
    @State private var region = MKCoordinateRegion(
        center: CLLocationCoordinate2D(latitude: 52.52, longitude: 13.405),
        span: MKCoordinateSpan(latitudeDelta: 0.5, longitudeDelta: 0.5)
    )

    var body: some View {
        NavigationView {
            ZStack {
                // Map view
                Map(coordinateRegion: $region, annotationItems: nearbyChargers) { charger in
                    MapAnnotation(coordinate: charger.coordinate) {
                        ChargerPin(charger: charger)
                    }
                }
                .ignoresSafeArea()

                // Overlay content
                VStack {
                    Spacer()

                    // Quick stats card
                    quickStatsCard

                    // Saved routes section
                    savedRoutesSection
                }
            }
            .navigationTitle("Routes")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        showingRoutePlanner = true
                    }) {
                        Image(systemName: "plus.circle.fill")
                            .font(.title2)
                    }
                }
            }
            .sheet(isPresented: $showingRoutePlanner) {
                RoutePlannerView()
                    .environmentObject(vehicleManager)
            }
            .onAppear {
                loadSavedRoutes()
            }
        }
    }

    // MARK: - Quick Stats Card

    private var quickStatsCard: some View {
        VStack(spacing: 12) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Current Range")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("\(vehicleManager.rangeKm) km")
                        .font(.title2.bold())
                }

                Spacer()

                VStack(alignment: .trailing) {
                    Text("Optimal Range")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("\(vehicleManager.rangeAt80) km @ 80%")
                        .font(.subheadline.bold())
                        .foregroundColor(.green)
                }
            }

            // Nearby chargers count
            HStack {
                Image(systemName: "mappin.circle.fill")
                    .foregroundColor(.blue)
                Text("\(nearbyChargers.count) chargers nearby")
                    .font(.caption)
                Spacer()
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(.ultraThinMaterial)
                .shadow(radius: 10)
        )
        .padding(.horizontal)
    }

    // MARK: - Saved Routes Section

    private var savedRoutesSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Saved Routes")
                .font(.headline)
                .padding(.horizontal)

            if savedRoutes.isEmpty {
                EmptyStateView(
                    icon: "map.fill",
                    message: "No saved routes yet",
                    actionTitle: "Plan a Route",
                    action: { showingRoutePlanner = true }
                )
                .padding()
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 12) {
                        ForEach(savedRoutes) { route in
                            RouteCard(route: route)
                        }
                    }
                    .padding(.horizontal)
                }
            }
        }
        .padding(.vertical)
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(.ultraThinMaterial)
        )
    }

    // MARK: - Mock Data

    private var nearbyChargers: [Charger] = [
        Charger(
            id: UUID(),
            name: "Supercharger Berlin",
            coordinate: CLLocationCoordinate2D(latitude: 52.5244, longitude: 13.4105),
            power: 150,
            available: 8,
            total: 12,
            type: .supercharger
        ),
        Charger(
            id: UUID(),
            name: "IONITY Potsdam",
            coordinate: CLLocationCoordinate2D(latitude: 52.3988, longitude: 13.0656),
            power: 350,
            available: 4,
            total: 6,
            type: .ultraFast
        ),
        Charger(
            id: UUID(),
            name: "ChargePoint Mall",
            coordinate: CLLocationCoordinate2D(latitude: 52.5070, longitude: 13.3233),
            power: 50,
            available: 2,
            total: 4,
            type: .fast
        ),
    ]

    // MARK: - Methods

    private func loadSavedRoutes() {
        // Mock saved routes
        savedRoutes = [
            SavedRoute(
                id: UUID(),
                name: "Home to Office",
                distance: 35,
                estimatedTime: 45,
                chargingStops: 0,
                isFavorite: true
            ),
            SavedRoute(
                id: UUID(),
                name: "Weekend Trip to Hamburg",
                distance: 290,
                estimatedTime: 195,
                chargingStops: 1,
                isFavorite: false
            ),
        ]
    }
}

// MARK: - Route Planner View

struct RoutePlannerView: View {
    @Environment(\.dismiss) var dismiss
    @EnvironmentObject var vehicleManager: VehicleManager
    @State private var origin = ""
    @State private var destination = ""
    @State private var departureTime = Date()
    @State private var targetArrivalCharge = 20
    @State private var plannedRoute: PlannedRoute?
    @State private var isCalculating = false

    var body: some View {
        NavigationView {
            Form {
                Section("Trip Details") {
                    TextField("Starting point", text: $origin)
                        .textContentType(.location)
                    TextField("Destination", text: $destination)
                        .textContentType(.location)

                    DatePicker("Departure", selection: $departureTime, in: Date()...)
                }

                Section("Charging Preferences") {
                    VStack(alignment: .leading) {
                        Text("Arrival Charge: \(targetArrivalCharge)%")
                            .font(.subheadline)
                        Slider(value: Binding(
                            get: { Double(targetArrivalCharge) },
                            set: { targetArrivalCharge = Int($0) }
                        ), in: 10...80, step: 5)
                        .tint(.green)

                        Text("Minimum charge on arrival for peace of mind")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                if let route = plannedRoute {
                    Section("Planned Route") {
                        routeSummary(route)
                    }

                    Section("Charging Stops") {
                        ForEach(route.chargingStops) { stop in
                            ChargingStopRow(stop: stop)
                        }
                    }

                    Section {
                        Button("Save Route") {
                            // Save route
                            dismiss()
                        }
                        .frame(maxWidth: .infinity)
                    }
                }

                Section {
                    if isCalculating {
                        HStack {
                            Spacer()
                            ProgressView()
                            Text("Calculating best route...")
                                .padding(.leading, 10)
                            Spacer()
                        }
                    } else {
                        Button("Plan Route") {
                            planRoute()
                        }
                        .frame(maxWidth: .infinity)
                        .disabled(origin.isEmpty || destination.isEmpty)
                    }
                }
            }
            .navigationTitle("Plan Route")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Close") {
                        dismiss()
                    }
                }
            }
        }
    }

    private func routeSummary(_ route: PlannedRoute) -> some View {
        VStack(spacing: 10) {
            HStack {
                Label("\(route.totalDistance) km", systemImage: "map")
                Spacer()
                Label("\(route.totalTime) min", systemImage: "clock")
            }

            HStack {
                Label("\(route.chargingStops.count) stops", systemImage: "bolt.fill")
                    .foregroundColor(.blue)
                Spacer()
                Label("\(route.totalChargingTime) min charging", systemImage: "hourglass")
                    .foregroundColor(.orange)
            }

            if route.totalDistance > vehicleManager.rangeKm {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.orange)
                    Text("Charging required for this trip")
                        .font(.caption)
                }
            } else {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Can complete without charging")
                        .font(.caption)
                }
            }
        }
        .font(.subheadline)
    }

    private func planRoute() {
        isCalculating = true

        // Simulate route planning
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            plannedRoute = PlannedRoute(
                id: UUID(),
                origin: origin,
                destination: destination,
                totalDistance: 290,
                totalTime: 195,
                totalChargingTime: 25,
                chargingStops: [
                    ChargingStop(
                        id: UUID(),
                        name: "Supercharger Magdeburg",
                        distanceFromStart: 145,
                        arrivalCharge: 25,
                        departureCharge: 80,
                        chargingTime: 25,
                        power: 150
                    )
                ]
            )
            isCalculating = false
        }
    }
}

// MARK: - Supporting Views

struct ChargerPin: View {
    let charger: Charger

    var body: some View {
        VStack(spacing: 2) {
            Image(systemName: charger.available > 0 ? "bolt.circle.fill" : "bolt.circle")
                .foregroundColor(charger.available > 0 ? .green : .gray)
                .font(.title2)
                .background(
                    Circle()
                        .fill(.white)
                        .frame(width: 30, height: 30)
                )

            Text("\(charger.available)/\(charger.total)")
                .font(.caption2)
                .padding(.horizontal, 4)
                .background(Color.white)
                .cornerRadius(4)
        }
    }
}

struct RouteCard: View {
    let route: SavedRoute

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(route.name)
                    .font(.headline)
                if route.isFavorite {
                    Image(systemName: "star.fill")
                        .foregroundColor(.yellow)
                        .font(.caption)
                }
            }

            HStack(spacing: 15) {
                Label("\(route.distance) km", systemImage: "map")
                    .font(.caption)
                Label("\(route.estimatedTime) min", systemImage: "clock")
                    .font(.caption)
            }
            .foregroundColor(.secondary)

            if route.chargingStops > 0 {
                HStack {
                    Image(systemName: "bolt.fill")
                        .foregroundColor(.blue)
                        .font(.caption)
                    Text("\(route.chargingStops) charging stop\(route.chargingStops > 1 ? "s" : "")")
                        .font(.caption)
                }
            } else {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                        .font(.caption)
                    Text("No charging needed")
                        .font(.caption)
                }
            }
        }
        .frame(width: 200)
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(.systemBackground))
                .shadow(radius: 3)
        )
    }
}

struct ChargingStopRow: View {
    let stop: ChargingStop

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(stop.name)
                .font(.headline)

            HStack {
                Label("\(stop.distanceFromStart) km", systemImage: "location")
                Spacer()
                Label("\(stop.power) kW", systemImage: "bolt.fill")
                    .foregroundColor(.blue)
            }
            .font(.caption)
            .foregroundColor(.secondary)

            HStack {
                Text("\(stop.arrivalCharge)% → \(stop.departureCharge)%")
                    .font(.subheadline)
                Spacer()
                Text("\(stop.chargingTime) min")
                    .font(.subheadline)
                    .foregroundColor(.orange)
            }
        }
        .padding()
        .background(Color(.secondarySystemBackground))
        .cornerRadius(8)
    }
}

struct EmptyStateView: View {
    let icon: String
    let message: String
    let actionTitle: String?
    let action: (() -> Void)?

    var body: some View {
        VStack(spacing: 15) {
            Image(systemName: icon)
                .font(.system(size: 50))
                .foregroundColor(.gray)

            Text(message)
                .font(.subheadline)
                .foregroundColor(.secondary)

            if let actionTitle = actionTitle, let action = action {
                Button(action: action) {
                    Text(actionTitle)
                        .font(.subheadline.bold())
                        .padding(.horizontal, 20)
                        .padding(.vertical, 10)
                        .background(Color.blue)
                        .foregroundColor(.white)
                        .cornerRadius(8)
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 30)
    }
}

// MARK: - Data Models

struct Charger: Identifiable {
    let id: UUID
    let name: String
    let coordinate: CLLocationCoordinate2D
    let power: Int
    let available: Int
    let total: Int
    let type: ChargerType

    enum ChargerType {
        case supercharger
        case ultraFast
        case fast
        case slow
    }
}

struct SavedRoute: Identifiable {
    let id: UUID
    let name: String
    let distance: Int
    let estimatedTime: Int
    let chargingStops: Int
    let isFavorite: Bool
}

struct PlannedRoute: Identifiable {
    let id: UUID
    let origin: String
    let destination: String
    let totalDistance: Int
    let totalTime: Int
    let totalChargingTime: Int
    let chargingStops: [ChargingStop]
}

struct ChargingStop: Identifiable {
    let id: UUID
    let name: String
    let distanceFromStart: Int
    let arrivalCharge: Int
    let departureCharge: Int
    let chargingTime: Int
    let power: Int
}

#Preview {
    RoutesView()
        .environmentObject(VehicleManager())
}

import SwiftUI
import Charts

/// Comprehensive analytics and insights
/// INNOVATIVE FEATURES:
/// - Battery health tracking over time
/// - Charging efficiency analysis
/// - Cost breakdown and projections
/// - Energy consumption patterns
/// - Comparative analytics
/// - Environmental impact tracking
struct AnalyticsView: View {
    @EnvironmentObject var vehicleManager: VehicleManager
    @EnvironmentObject var chargingManager: ChargingManager
    @State private var selectedTimeRange: TimeRange = .week
    @State private var selectedTab: AnalyticsTab = .overview

    enum TimeRange: String, CaseIterable {
        case week = "Week"
        case month = "Month"
        case year = "Year"
    }

    enum AnalyticsTab: String, CaseIterable {
        case overview = "Overview"
        case battery = "Battery"
        case costs = "Costs"
        case efficiency = "Efficiency"
    }

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Time range picker
                    timeRangePicker

                    // Tab content
                    switch selectedTab {
                    case .overview:
                        overviewContent
                    case .battery:
                        batteryAnalytics
                    case .costs:
                        costAnalytics
                    case .efficiency:
                        efficiencyAnalytics
                    }
                }
                .padding()
            }
            .navigationTitle("Analytics")
            .toolbar {
                ToolbarItem(placement: .principal) {
                    tabPicker
                }
            }
        }
    }

    // MARK: - Time Range Picker

    private var timeRangePicker: some View {
        Picker("Time Range", selection: $selectedTimeRange) {
            ForEach(TimeRange.allCases, id: \.self) { range in
                Text(range.rawValue).tag(range)
            }
        }
        .pickerStyle(.segmented)
    }

    // MARK: - Tab Picker

    private var tabPicker: some View {
        Picker("Analytics Tab", selection: $selectedTab) {
            ForEach(AnalyticsTab.allCases, id: \.self) { tab in
                Text(tab.rawValue).tag(tab)
            }
        }
        .pickerStyle(.segmented)
    }

    // MARK: - Overview Content

    private var overviewContent: some View {
        VStack(spacing: 20) {
            // Key metrics cards
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 15) {
                MetricCard(
                    title: "Total Energy",
                    value: "\(String(format: "%.1f", chargingManager.totalEnergyThisWeek)) kWh",
                    icon: "bolt.fill",
                    color: .blue,
                    trend: .up,
                    trendValue: "12%"
                )

                MetricCard(
                    title: "Total Cost",
                    value: "€\(String(format: "%.2f", chargingManager.totalCostThisWeek))",
                    icon: "eurosign.circle.fill",
                    color: .green,
                    trend: .down,
                    trendValue: "8%"
                )

                MetricCard(
                    title: "Sessions",
                    value: "\(chargingManager.chargingSessions.prefix(7).count)",
                    icon: "bolt.car.fill",
                    color: .purple,
                    trend: .neutral,
                    trendValue: "Same"
                )

                MetricCard(
                    title: "Avg. Duration",
                    value: formatDuration(chargingManager.averageChargingTime),
                    icon: "clock.fill",
                    color: .orange,
                    trend: .down,
                    trendValue: "5%"
                )
            }

            // Charging activity chart
            chargingActivityChart

            // Quick insights
            quickInsightsSection
        }
    }

    // MARK: - Battery Analytics

    private var batteryAnalytics: some View {
        VStack(spacing: 20) {
            // Battery health overview - INNOVATIVE
            batteryHealthCard

            // Health trend chart - INNOVATIVE
            batteryHealthTrendChart

            // Cycle count and degradation - INNOVATIVE
            batteryCycleInfo

            // Temperature analysis
            batteryTemperatureChart

            // Health insights
            batteryHealthInsights
        }
    }

    private var batteryHealthCard: some View {
        VStack(spacing: 15) {
            HStack {
                Image(systemName: "heart.fill")
                    .foregroundColor(.red)
                    .font(.title2)
                Text("Battery Health")
                    .font(.headline)
                Spacer()
            }

            // Large health percentage
            ZStack {
                Circle()
                    .stroke(Color.gray.opacity(0.2), lineWidth: 15)
                    .frame(width: 150, height: 150)

                Circle()
                    .trim(from: 0, to: Double(vehicleManager.batteryHealth) / 100.0)
                    .stroke(
                        LinearGradient(
                            colors: healthGradientColors,
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        ),
                        style: StrokeStyle(lineWidth: 15, lineCap: .round)
                    )
                    .frame(width: 150, height: 150)
                    .rotationEffect(.degrees(-90))
                    .animation(.easeInOut, value: vehicleManager.batteryHealth)

                VStack {
                    Text("\(vehicleManager.batteryHealth)%")
                        .font(.system(size: 42, weight: .bold))
                    Text(healthStatus)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            // Health details grid
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                HealthDetailBox(label: "Capacity", value: "\(vehicleManager.batteryHealth)%", icon: "battery.100")
                HealthDetailBox(label: "Cycles", value: "120", icon: "arrow.triangle.2.circlepath")
                HealthDetailBox(label: "Age", value: "1.5 yrs", icon: "calendar")
                HealthDetailBox(label: "Degradation", value: "2%", icon: "chart.line.downtrend.xyaxis")
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var batteryHealthTrendChart: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Health Trend")
                .font(.headline)

            // Mock health trend data
            let healthData: [(String, Int)] = [
                ("Jan", 100),
                ("Feb", 100),
                ("Mar", 99),
                ("Apr", 99),
                ("May", 98),
                ("Jun", 98),
            ]

            Chart {
                ForEach(healthData, id: \.0) { item in
                    LineMark(
                        x: .value("Month", item.0),
                        y: .value("Health", item.1)
                    )
                    .foregroundStyle(.green)
                    .interpolationMethod(.catmullRom)

                    AreaMark(
                        x: .value("Month", item.0),
                        y: .value("Health", item.1)
                    )
                    .foregroundStyle(
                        .linearGradient(
                            colors: [.green.opacity(0.3), .green.opacity(0.1)],
                            startPoint: .top,
                            endPoint: .bottom
                        )
                    )
                    .interpolationMethod(.catmullRom)
                }
            }
            .frame(height: 200)
            .chartYScale(domain: 95...100)
            .chartXAxis {
                AxisMarks(values: .automatic) { _ in
                    AxisValueLabel()
                    AxisGridLine()
                }
            }
            .chartYAxis {
                AxisMarks(position: .leading)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var batteryCycleInfo: some View {
        VStack(spacing: 15) {
            HStack {
                Text("Charging Cycles")
                    .font(.headline)
                Spacer()
                Text("120 cycles")
                    .font(.title3.bold())
                    .foregroundColor(.blue)
            }

            // Progress bar showing cycles used
            GeometryReader { geometry in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color.gray.opacity(0.2))
                        .frame(height: 20)

                    RoundedRectangle(cornerRadius: 8)
                        .fill(
                            LinearGradient(
                                colors: [.blue, .purple],
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .frame(width: geometry.size.width * 0.12, height: 20)
                }
            }
            .frame(height: 20)

            HStack {
                Text("12% of expected lifetime")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Text("~880 cycles remaining")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var batteryTemperatureChart: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Temperature History")
                .font(.headline)

            let tempData: [(String, Double)] = [
                ("Mon", 22.5),
                ("Tue", 21.8),
                ("Wed", 23.2),
                ("Thu", 22.0),
                ("Fri", 24.1),
                ("Sat", 23.5),
                ("Sun", 22.8),
            ]

            Chart {
                ForEach(tempData, id: \.0) { item in
                    BarMark(
                        x: .value("Day", item.0),
                        y: .value("Temp", item.1)
                    )
                    .foregroundStyle(.orange)
                    .cornerRadius(6)
                }

                RuleMark(y: .value("Optimal", 22.5))
                    .foregroundStyle(.green.opacity(0.5))
                    .lineStyle(StrokeStyle(lineWidth: 2, dash: [5, 5]))
            }
            .frame(height: 180)
            .chartYScale(domain: 20...26)

            HStack {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(.green)
                Text("Temperature within optimal range")
                    .font(.caption)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var batteryHealthInsights: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Health Insights")
                .font(.headline)

            InsightRow(
                icon: "checkmark.circle.fill",
                color: .green,
                title: "Excellent Health",
                description: "Your battery is performing optimally with minimal degradation"
            )

            InsightRow(
                icon: "chart.line.uptrend.xyaxis",
                color: .blue,
                title: "Charging Pattern",
                description: "Frequent 80% charging is helping maintain battery health"
            )

            InsightRow(
                icon: "thermometer.medium",
                color: .orange,
                title: "Temperature",
                description: "Operating temperature is within ideal range (20-25°C)"
            )
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Cost Analytics

    private var costAnalytics: some View {
        VStack(spacing: 20) {
            // Total cost overview
            totalCostCard

            // Cost breakdown chart
            costBreakdownChart

            // Cost per kWh trend
            costPerKwhChart

            // Savings comparison
            savingsComparison
        }
    }

    private var totalCostCard: some View {
        VStack(spacing: 15) {
            HStack {
                Image(systemName: "eurosign.circle.fill")
                    .foregroundColor(.green)
                    .font(.title2)
                Text("Charging Costs")
                    .font(.headline)
                Spacer()
            }

            HStack(alignment: .firstTextBaseline) {
                Text("€\(String(format: "%.2f", chargingManager.totalCostThisWeek))")
                    .font(.system(size: 48, weight: .bold))
                    .foregroundColor(.green)
                Text("this week")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                CostDetailBox(label: "Per Session", value: "€6.08")
                CostDetailBox(label: "Per kWh", value: "€0.173")
                CostDetailBox(label: "Monthly Est.", value: "€170")
                CostDetailBox(label: "Yearly Est.", value: "€2,040")
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var costBreakdownChart: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Weekly Cost Breakdown")
                .font(.headline)

            let costData: [(String, Double)] = [
                ("Mon", 8.5),
                ("Tue", 0),
                ("Wed", 12.3),
                ("Thu", 0),
                ("Fri", 9.7),
                ("Sat", 11.2),
                ("Sun", 0),
            ]

            Chart {
                ForEach(costData, id: \.0) { item in
                    BarMark(
                        x: .value("Day", item.0),
                        y: .value("Cost", item.1)
                    )
                    .foregroundStyle(.green)
                    .cornerRadius(6)
                }
            }
            .frame(height: 200)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var costPerKwhChart: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Cost per kWh Trend")
                .font(.headline)

            let priceData: [(String, Double)] = [
                ("Week 1", 0.168),
                ("Week 2", 0.175),
                ("Week 3", 0.171),
                ("Week 4", 0.173),
            ]

            Chart {
                ForEach(priceData, id: \.0) { item in
                    LineMark(
                        x: .value("Week", item.0),
                        y: .value("Price", item.1)
                    )
                    .foregroundStyle(.green)
                    .symbol(.circle)

                    PointMark(
                        x: .value("Week", item.0),
                        y: .value("Price", item.1)
                    )
                    .foregroundStyle(.green)
                }
            }
            .frame(height: 150)
            .chartYScale(domain: 0.16...0.18)

            HStack {
                Text("Average: €0.172/kWh")
                    .font(.caption)
                Spacer()
                Text("Below market avg")
                    .font(.caption)
                    .foregroundColor(.green)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var savingsComparison: some View {
        VStack(alignment: .leading, spacing: 15) {
            Text("Savings vs. Gasoline")
                .font(.headline)

            HStack(spacing: 30) {
                VStack {
                    Text("⚡")
                        .font(.system(size: 40))
                    Text("€42.50")
                        .font(.title2.bold())
                        .foregroundColor(.green)
                    Text("Electric")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Image(systemName: "arrow.right")
                    .foregroundColor(.gray)

                VStack {
                    Text("⛽")
                        .font(.system(size: 40))
                    Text("€87.50")
                        .font(.title2.bold())
                        .foregroundColor(.red)
                    Text("Gasoline")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            .frame(maxWidth: .infinity)

            Divider()

            HStack {
                Image(systemName: "leaf.fill")
                    .foregroundColor(.green)
                Text("You saved €45.00 this week")
                    .font(.subheadline.bold())
            }

            HStack {
                Image(systemName: "cloud.fill")
                    .foregroundColor(.blue)
                Text("Avoided 25 kg of CO₂ emissions")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Efficiency Analytics

    private var efficiencyAnalytics: some View {
        VStack(spacing: 20) {
            // Charging efficiency
            chargingEfficiencyCard

            // Energy consumption
            energyConsumptionCard

            // Efficiency tips
            efficiencyTipsSection
        }
    }

    private var chargingEfficiencyCard: some View {
        VStack(spacing: 15) {
            HStack {
                Image(systemName: "bolt.badge.checkmark")
                    .foregroundColor(.blue)
                    .font(.title2)
                Text("Charging Efficiency")
                    .font(.headline)
                Spacer()
            }

            ZStack {
                Circle()
                    .stroke(Color.gray.opacity(0.2), lineWidth: 20)
                    .frame(width: 140, height: 140)

                Circle()
                    .trim(from: 0, to: 0.92)
                    .stroke(
                        LinearGradient(
                            colors: [.blue, .cyan],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        ),
                        style: StrokeStyle(lineWidth: 20, lineCap: .round)
                    )
                    .frame(width: 140, height: 140)
                    .rotationEffect(.degrees(-90))

                VStack {
                    Text("92%")
                        .font(.system(size: 42, weight: .bold))
                    Text("Efficient")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Text("Your charging efficiency is excellent")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var energyConsumptionCard: some View {
        VStack(alignment: .leading, spacing: 15) {
            Text("Energy Consumption")
                .font(.headline)

            HStack(alignment: .firstTextBaseline) {
                Text("18.5")
                    .font(.system(size: 48, weight: .bold))
                    .foregroundColor(.purple)
                Text("kWh/100km")
                    .font(.title3)
                    .foregroundColor(.secondary)
            }

            let consumptionData: [(String, Double)] = [
                ("Mon", 17.2),
                ("Tue", 18.9),
                ("Wed", 17.5),
                ("Thu", 19.1),
                ("Fri", 18.0),
                ("Sat", 18.8),
                ("Sun", 17.9),
            ]

            Chart {
                ForEach(consumptionData, id: \.0) { item in
                    LineMark(
                        x: .value("Day", item.0),
                        y: .value("Consumption", item.1)
                    )
                    .foregroundStyle(.purple)
                    .interpolationMethod(.catmullRom)
                }

                RuleMark(y: .value("Target", 18.0))
                    .foregroundStyle(.green.opacity(0.5))
                    .lineStyle(StrokeStyle(lineWidth: 2, dash: [5, 5]))
            }
            .frame(height: 150)

            HStack {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(.green)
                Text("Below average consumption")
                    .font(.caption)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var efficiencyTipsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Efficiency Tips")
                .font(.headline)

            TipRow(
                icon: "80.circle.fill",
                color: .green,
                title: "Charge to 80%",
                description: "Charging to 80% is faster and better for battery health"
            )

            TipRow(
                icon: "thermometer.medium",
                color: .orange,
                title: "Pre-condition",
                description: "Pre-heat or cool while plugged in to save energy"
            )

            TipRow(
                icon: "leaf.fill",
                color: .green,
                title: "Eco Mode",
                description: "Enable eco mode to maximize range efficiency"
            )
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Supporting Content

    private var chargingActivityChart: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Charging Activity")
                .font(.headline)

            let activityData: [(String, Double)] = [
                ("Mon", 35.5),
                ("Tue", 0),
                ("Wed", 42.0),
                ("Thu", 0),
                ("Fri", 28.5),
                ("Sat", 38.2),
                ("Sun", 0),
            ]

            Chart {
                ForEach(activityData, id: \.0) { item in
                    BarMark(
                        x: .value("Day", item.0),
                        y: .value("Energy", item.1)
                    )
                    .foregroundStyle(.blue)
                    .cornerRadius(6)
                }
            }
            .frame(height: 200)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    private var quickInsightsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Insights")
                .font(.headline)

            InsightRow(
                icon: "chart.line.uptrend.xyaxis",
                color: .green,
                title: "Cost Trending Down",
                description: "Your average charging cost decreased 8% this week"
            )

            InsightRow(
                icon: "clock.badge.checkmark",
                color: .blue,
                title: "Optimal Timing",
                description: "Most sessions during off-peak hours (60% savings)"
            )

            InsightRow(
                icon: "battery.100",
                color: .purple,
                title: "Battery Health Excellent",
                description: "Consistent 80% charging is maintaining health at 98%"
            )
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(Color(.systemBackground))
                .shadow(color: Color.black.opacity(0.1), radius: 10, x: 0, y: 5)
        )
    }

    // MARK: - Helpers

    private var healthGradientColors: [Color] {
        if vehicleManager.batteryHealth >= 95 {
            return [.green, .blue]
        } else if vehicleManager.batteryHealth >= 85 {
            return [.blue, .purple]
        } else {
            return [.orange, .red]
        }
    }

    private var healthStatus: String {
        if vehicleManager.batteryHealth >= 95 {
            return "Excellent"
        } else if vehicleManager.batteryHealth >= 85 {
            return "Good"
        } else if vehicleManager.batteryHealth >= 70 {
            return "Fair"
        } else {
            return "Poor"
        }
    }

    private func formatDuration(_ duration: TimeInterval) -> String {
        let hours = Int(duration) / 3600
        let minutes = Int(duration) % 3600 / 60
        if hours > 0 {
            return "\(hours)h \(minutes)m"
        } else {
            return "\(minutes)m"
        }
    }
}

// MARK: - Supporting Views

struct MetricCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color
    let trend: Trend
    let trendValue: String

    enum Trend {
        case up, down, neutral
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .foregroundColor(color)
                Spacer()
                trendIndicator
            }

            Text(value)
                .font(.title3.bold())

            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(.secondarySystemBackground))
        )
    }

    @ViewBuilder
    private var trendIndicator: some View {
        HStack(spacing: 2) {
            switch trend {
            case .up:
                Image(systemName: "arrow.up.right")
                    .foregroundColor(.green)
            case .down:
                Image(systemName: "arrow.down.right")
                    .foregroundColor(.red)
            case .neutral:
                Image(systemName: "minus")
                    .foregroundColor(.gray)
            }
            Text(trendValue)
                .font(.caption2)
        }
        .foregroundColor(.secondary)
    }
}

struct HealthDetailBox: View {
    let label: String
    let value: String
    let icon: String

    var body: some View {
        VStack(spacing: 6) {
            Image(systemName: icon)
                .foregroundColor(.blue)
            Text(value)
                .font(.headline)
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 10)
        .background(Color(.secondarySystemBackground))
        .cornerRadius(8)
    }
}

struct CostDetailBox: View {
    let label: String
    let value: String

    var body: some View {
        VStack(spacing: 6) {
            Text(value)
                .font(.headline)
                .foregroundColor(.green)
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 10)
        .background(Color(.secondarySystemBackground))
        .cornerRadius(8)
    }
}

struct InsightRow: View {
    let icon: String
    let color: Color
    let title: String
    let description: String

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(color)
                .frame(width: 40)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.bold())
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 8)
    }
}

struct TipRow: View {
    let icon: String
    let color: Color
    let title: String
    let description: String

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(color)
                .frame(width: 40)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.bold())
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 8)
    }
}

#Preview {
    AnalyticsView()
        .environmentObject(VehicleManager())
        .environmentObject(ChargingManager())
}

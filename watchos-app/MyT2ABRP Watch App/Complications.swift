import SwiftUI
import WidgetKit

/// Watch complications for at-a-glance vehicle status
/// INNOVATIVE FEATURES:
/// - Battery level on watch face
/// - Charging indicator
/// - Range display
/// - Quick status colors
/// - Real-time updates

// MARK: - Complication Provider

struct MyT2ABRPComplicationProvider: TimelineProvider {
    typealias Entry = MyT2ABRPEntry

    func placeholder(in context: Context) -> MyT2ABRPEntry {
        MyT2ABRPEntry(date: Date(), batteryLevel: 85, isCharging: false, range: 320)
    }

    func getSnapshot(in context: Context, completion: @escaping (MyT2ABRPEntry) -> Void) {
        let entry = MyT2ABRPEntry(date: Date(), batteryLevel: 85, isCharging: false, range: 320)
        completion(entry)
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<MyT2ABRPEntry>) -> Void) {
        // Fetch current vehicle status
        let currentDate = Date()
        let batteryLevel = UserDefaults.standard.integer(forKey: "vehicle_battery_level")
        let range = UserDefaults.standard.integer(forKey: "vehicle_range_km")
        let isCharging = UserDefaults.standard.bool(forKey: "charging_is_charging")

        let entry = MyT2ABRPEntry(
            date: currentDate,
            batteryLevel: batteryLevel > 0 ? batteryLevel : 85,
            isCharging: isCharging,
            range: range > 0 ? range : 320
        )

        // Refresh every 15 minutes
        let nextUpdate = Calendar.current.date(byAdding: .minute, value: 15, to: currentDate)!
        let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))

        completion(timeline)
    }
}

// MARK: - Complication Entry

struct MyT2ABRPEntry: TimelineEntry {
    let date: Date
    let batteryLevel: Int
    let isCharging: Bool
    let range: Int

    var batteryColor: Color {
        switch batteryLevel {
        case 80...100: return .green
        case 50..<80: return .blue
        case 20..<50: return .orange
        default: return .red
        }
    }
}

// MARK: - Complication Views

struct MyT2ABRPComplicationView: View {
    var entry: MyT2ABRPEntry
    @Environment(\.widgetFamily) var family

    var body: some View {
        switch family {
        case .accessoryCircular:
            circularComplication

        case .accessoryRectangular:
            rectangularComplication

        case .accessoryInline:
            inlineComplication

        case .accessoryCorner:
            cornerComplication

        default:
            EmptyView()
        }
    }

    // MARK: - Circular Complication (Large Round)

    private var circularComplication: some View {
        ZStack {
            // Background ring
            Circle()
                .stroke(Color.gray.opacity(0.3), lineWidth: 5)

            // Battery level ring
            Circle()
                .trim(from: 0, to: Double(entry.batteryLevel) / 100.0)
                .stroke(entry.batteryColor, style: StrokeStyle(lineWidth: 5, lineCap: .round))
                .rotationEffect(.degrees(-90))

            // Center content
            VStack(spacing: 2) {
                Text("\(entry.batteryLevel)")
                    .font(.system(size: 26, weight: .bold, design: .rounded))
                    .foregroundColor(entry.batteryColor)

                if entry.isCharging {
                    Image(systemName: "bolt.fill")
                        .font(.system(size: 10))
                        .foregroundColor(.yellow)
                }
            }
        }
        .padding(4)
    }

    // MARK: - Rectangular Complication

    private var rectangularComplication: some View {
        HStack(spacing: 8) {
            // Battery icon with level
            ZStack {
                Image(systemName: "battery.100")
                    .font(.title2)
                    .foregroundColor(entry.batteryColor)

                if entry.isCharging {
                    Image(systemName: "bolt.fill")
                        .font(.caption2)
                        .foregroundColor(.yellow)
                }
            }

            VStack(alignment: .leading, spacing: 2) {
                // Battery percentage
                HStack(spacing: 4) {
                    Text("\(entry.batteryLevel)%")
                        .font(.headline)
                        .foregroundColor(entry.batteryColor)

                    if entry.isCharging {
                        Image(systemName: "arrow.up")
                            .font(.caption2)
                            .foregroundColor(.green)
                    }
                }

                // Range
                HStack(spacing: 3) {
                    Image(systemName: "location.fill")
                        .font(.caption2)
                    Text("\(entry.range) km")
                        .font(.caption2)
                }
                .foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 4)
    }

    // MARK: - Inline Complication (Small Text)

    private var inlineComplication: some View {
        HStack(spacing: 4) {
            if entry.isCharging {
                Image(systemName: "bolt.fill")
                    .foregroundColor(.yellow)
            }

            Text("\(entry.batteryLevel)%")
                .foregroundColor(entry.batteryColor)

            Text("·")
                .foregroundColor(.secondary)

            Text("\(entry.range)km")
                .foregroundColor(.secondary)
        }
        .font(.system(size: 14, weight: .medium, design: .rounded))
    }

    // MARK: - Corner Complication

    private var cornerComplication: some View {
        ZStack {
            // Curved gauge
            Circle()
                .trim(from: 0, to: Double(entry.batteryLevel) / 100.0)
                .stroke(entry.batteryColor, style: StrokeStyle(lineWidth: 6, lineCap: .round))
                .rotationEffect(.degrees(-90))

            VStack(spacing: 0) {
                Text("\(entry.batteryLevel)")
                    .font(.system(size: 20, weight: .bold, design: .rounded))
                    .foregroundColor(entry.batteryColor)

                if entry.isCharging {
                    Image(systemName: "bolt.fill")
                        .font(.system(size: 8))
                        .foregroundColor(.yellow)
                }
            }
        }
    }
}

// MARK: - Complication Widget Configuration

@main
struct MyT2ABRPComplication: Widget {
    let kind: String = "MyT2ABRPComplication"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: MyT2ABRPComplicationProvider()) { entry in
            MyT2ABRPComplicationView(entry: entry)
        }
        .configurationDisplayName("Vehicle Status")
        .description("Battery level and charging status at a glance")
        .supportedFamilies([
            .accessoryCircular,
            .accessoryRectangular,
            .accessoryInline,
            .accessoryCorner
        ])
    }
}

// MARK: - Preview

#Preview("Circular", as: .accessoryCircular) {
    MyT2ABRPComplication()
} timeline: {
    MyT2ABRPEntry(date: .now, batteryLevel: 85, isCharging: false, range: 320)
    MyT2ABRPEntry(date: .now, batteryLevel: 87, isCharging: true, range: 330)
    MyT2ABRPEntry(date: .now, batteryLevel: 25, isCharging: false, range: 95)
}

#Preview("Rectangular", as: .accessoryRectangular) {
    MyT2ABRPComplication()
} timeline: {
    MyT2ABRPEntry(date: .now, batteryLevel: 85, isCharging: false, range: 320)
    MyT2ABRPEntry(date: .now, batteryLevel: 87, isCharging: true, range: 330)
}

#Preview("Inline", as: .accessoryInline) {
    MyT2ABRPComplication()
} timeline: {
    MyT2ABRPEntry(date: .now, batteryLevel: 85, isCharging: false, range: 320)
    MyT2ABRPEntry(date: .now, batteryLevel: 87, isCharging: true, range: 330)
}

#Preview("Corner", as: .accessoryCorner) {
    MyT2ABRPComplication()
} timeline: {
    MyT2ABRPEntry(date: .now, batteryLevel: 85, isCharging: false, range: 320)
    MyT2ABRPEntry(date: .now, batteryLevel: 87, isCharging: true, range: 330)
}

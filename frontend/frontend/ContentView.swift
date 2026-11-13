import SwiftUI
import Playgrounds

struct ContentView: View {
    @State private var villages: [Village] = []

    func remainingSeconds(exportTimestamp: Int, timer: Int) -> TimeInterval {
        return TimeInterval(exportTimestamp) + TimeInterval(timer) - Date().timeIntervalSince1970
    }

    func formatRemainingTime(time: TimeInterval) -> String {
        if(time <= 0) { return "Expected to be complete" } else {
            let totalSeconds = Int(time)

            let days    = totalSeconds / 86_400
            let hours   = (totalSeconds % 86_400) / 3_600
            let minutes = (totalSeconds % 3_600) / 60
            let seconds = totalSeconds % 60

            if (days > 0) {
                return "\(days)d \(hours)h \(minutes)m \(seconds)s left"
            } else if (hours > 0) {
                return "\(hours)h \(minutes)m \(seconds)s left"
            } else if (minutes > 0){
                return "\(minutes)m \(seconds)s left"
            } else if (minutes == 0 && totalSeconds > 0) {
                return "\(totalSeconds)s left"
            } else {
                return "Less than a second left"
            }
        }

    }

    var body: some View {
        ScrollView {
            VStack {
                Text("My Villages")
                    .padding()
                    .font(.largeTitle)
                    .bold()
                ForEach(villages, id: \.id) {
                    village in Text(village.label ?? village.tag)

                    ForEach(village.timedEntries.indices, id: \.self) {
                        index in VStack(alignment: .leading) {
                            Text(village.timedEntries[index].name ?? "Unknown upgrade")

                            TimelineView(.periodic(from: .now, by: 1)) {
                                _ in
                                let remaining = remainingSeconds(
                                    exportTimestamp: village.exportTimestamp,
                                    timer: village.timedEntries[index].entry.timer
                                )

                                Text(formatRemainingTime(time: remaining))
                                    .foregroundStyle(.secondary)
                            }


                        }
                    }
                }
                .padding()
                .background(.gray.opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 12))

            }
            .task {
                do {
                    villages = try await VillageAPI().fetchVillages()
                } catch {
                    print("Failed to fetch villages:", error)
                }
            }
        }
    }
}

#Preview {
    ContentView()
}

#Playground {
    _ = 1 + 2
}

import SwiftUI

struct ContentView: View {
    @State private var villages: [Village]
    @State private var selectedID: String?
    @State private var showImport = false
    @State private var isLoading = false
    @State private var loadError: String?
    @State private var reminderMessage: String?
    @State private var requestingReminderPermission = false
    private let previewMode: Bool

    init(previewVillages: [Village]? = nil) {
        _villages = State(initialValue: previewVillages ?? [])
        previewMode = previewVillages != nil
    }

    private var selectedVillage: Village? {
        villages.first { $0.id == selectedID } ?? villages.first
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                header

                if let loadError {
                    Label(loadError, systemImage: "exclamationmark.circle")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                if let reminderMessage {
                    Label(reminderMessage, systemImage: "bell")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                if let village = selectedVillage {
                    villagePicker(village)
                    TimelineView(.periodic(from: .now, by: 1)) { context in
                        upgrades(for: village, now: context.date)
                    }
                } else if isLoading {
                    ProgressView("Loading villages…")
                        .frame(maxWidth: .infinity, minHeight: 220)
                } else {
                    ContentUnavailableView(
                        loadError == nil ? "No villages yet" : "Couldn’t load villages",
                        systemImage: "house",
                        description: Text(loadError == nil
                            ? "Import a village export to start tracking upgrades."
                            : "Make sure the backend is running, then refresh.")
                    )
                }
            }
            .frame(maxWidth: 680)
            .padding(20)
            .frame(maxWidth: .infinity)
        }
        .background(TrackerStyle.background.ignoresSafeArea())
        .preferredColorScheme(.dark)
        .tint(TrackerStyle.accent)
        .refreshable { await loadVillages() }
        .task { if !previewMode { await loadVillages() } }
        .sheet(isPresented: $showImport) {
            ImportVillageView {
                await loadVillages()
            }
            .preferredColorScheme(.dark)
            .tint(TrackerStyle.accent)
        }
    }

    private var header: some View {
        ViewThatFits(in: .horizontal) {
            HStack {
                title
                Spacer(minLength: 16)
                actions
            }
            VStack(alignment: .leading, spacing: 12) {
                title
                actions
            }
        }
    }

    private var title: some View {
        Text("My Villages")
            .font(.largeTitle.weight(.bold))
            .accessibilityAddTraits(.isHeader)
    }

    private var actions: some View {
        HStack(spacing: 8) {
            Button {
                requestingReminderPermission = true
                Task { await enableReminders() }
            } label: {
                Image(systemName: "bell")
                    .frame(minWidth: 44, minHeight: 44)
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Enable one-minute upgrade reminders")
            .help("Enable one-minute upgrade reminders")
            .disabled(requestingReminderPermission || isLoading || previewMode)

            Button {
                Task { await loadVillages() }
            } label: {
                Image(systemName: "arrow.clockwise")
                    .frame(minWidth: 44, minHeight: 44)
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Refresh villages")
            .disabled(isLoading || requestingReminderPermission)

            Button {
                showImport = true
            } label: {
                Label("Import", systemImage: "square.and.arrow.down")
                    .fontWeight(.semibold)
                    .padding(.horizontal, 14)
                    .frame(minHeight: 44)
                    .background(TrackerStyle.accent, in: RoundedRectangle(cornerRadius: 10))
                    .foregroundStyle(.black)
            }
            .buttonStyle(.plain)
        }
    }

    private func villagePicker(_ village: Village) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            if villages.count > 1 {
                Menu {
                    ForEach(villages, id: \.id) { option in
                        Button {
                            selectedID = option.id
                        } label: {
                            if option.id == village.id {
                                Label(option.displayName, systemImage: "checkmark")
                            } else {
                                Text(option.displayName)
                            }
                        }
                    }
                } label: {
                    Label(village.displayName, systemImage: "chevron.up.chevron.down")
                        .font(.title3.weight(.semibold))
                        .frame(minHeight: 44)
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
                .accessibilityHint("Switch village")
            } else {
                Text(village.displayName)
                    .font(.title3.weight(.semibold))
            }

            if village.displayName != village.tag {
                Text(village.tag)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text("Exported \(Date(timeIntervalSince1970: Double(village.exportTimestamp)).formatted(date: .abbreviated, time: .shortened))")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(16)
        .background(TrackerStyle.surface, in: RoundedRectangle(cornerRadius: 14))
    }

    private func upgrades(for village: Village, now: Date) -> some View {
        let rows = village.timedEntries.enumerated().map { index, entry in
            UpgradeRow(
                id: index,
                upgrade: entry,
                finish: Date(timeIntervalSince1970: Double(village.exportTimestamp) + Double(entry.entry.timer))
            )
        }.sorted {
            if $0.finish == $1.finish { return $0.id < $1.id }
            return $0.finish > $1.finish
        }
        let home = rows.filter { $0.upgrade.baseName == "Home Village" }
        let builder = rows.filter { $0.upgrade.baseName == "Builder Base" }
        let unclassified = rows.filter { $0.upgrade.baseName == nil }

        return VStack(alignment: .leading, spacing: 24) {
            upgradeSection("Home Village", rows: home, now: now)
            upgradeSection("Builder Base", rows: builder, now: now)
            if !unclassified.isEmpty {
                upgradeSection("Unclassified upgrades", rows: unclassified, now: now)
            }
        }
        .id(village.id)
    }

    private func upgradeSection(_ title: String, rows: [UpgradeRow], now: Date) -> some View {
        let active = rows.filter { $0.finish > now }
        let elapsed = rows.filter { $0.finish <= now }

        return VStack(alignment: .leading, spacing: 10) {
            Text(title)
                .font(.headline)
                .accessibilityAddTraits(.isHeader)

            if active.isEmpty {
                Text("No active timers in this export.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            } else {
                upgradeList(active, now: now)
            }

            if !elapsed.isEmpty {
                DisclosureGroup("Estimated complete (\(elapsed.count))") {
                    upgradeList(elapsed, now: now)
                        .padding(.top, 8)
                }
                .font(.subheadline.weight(.semibold))
            }
        }
    }

    private func upgradeList(_ rows: [UpgradeRow], now: Date) -> some View {
        VStack(spacing: 0) {
            ForEach(rows) { row in
                if row.id != rows.first?.id {
                    Divider().padding(.leading, 62)
                }
                row.content(now: now)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 9)
            }
        }
        .background(TrackerStyle.surface, in: RoundedRectangle(cornerRadius: 14))
    }

    private func enableReminders() async {
        defer { requestingReminderPermission = false }
        do {
            guard try await UpgradeReminders.requestPermission() else {
                reminderMessage = "Allow notifications in Settings to get upgrade reminders."
                return
            }
            reminderMessage = "Reminders enabled for one minute before upgrades finish."
            await loadVillages()
        } catch {
            reminderMessage = "Couldn’t enable upgrade reminders."
        }
    }

    private func loadVillages() async {
        guard !isLoading else { return }
        isLoading = true
        defer { isLoading = false }

        do {
            villages = try await VillageAPI().fetchVillages()
            loadError = nil
            do {
                try await UpgradeReminders.synchronize(villages: villages)
            } catch {
                reminderMessage = "Couldn’t schedule all upgrade reminders."
            }
        } catch is CancellationError {
            return
        } catch {
            loadError = "Couldn’t refresh village data."
        }
    }
}

private struct UpgradeRow: Identifiable {
    let id: Int
    let upgrade: TimedEntry
    let finish: Date

    func content(now: Date) -> some View {
        let remaining = finish.timeIntervalSince(now)
        return HStack(spacing: 12) {
            Group {
                if let itemID = upgrade.entry.data, let artwork = ClashArtwork.byID[itemID] {
                    Image(artwork).resizable().scaledToFit()
                } else {
                    Image(systemName: "hammer").foregroundStyle(.secondary)
                }
            }
            .frame(width: 44, height: 44)
            .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 4) {
                Text(upgrade.name ?? "Unknown upgrade")
                    .font(.subheadline.weight(.semibold))

                if let level = upgrade.entry.level {
                    Text("Level \(level)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if remaining > 0 {
                    Text(timerText(remaining))
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(TrackerStyle.accent)
                } else {
                    Text("Expected to be complete")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer(minLength: 0)
        }
        .accessibilityElement(children: .combine)
    }

    private func timerText(_ interval: TimeInterval) -> String {
        let seconds = Int(interval)
        let days = seconds / 86_400
        let hours = seconds % 86_400 / 3_600
        let minutes = seconds % 3_600 / 60
        let secondsLeft = seconds % 60
        if days > 0 { return "\(days)d \(hours)h \(minutes)m \(secondsLeft)s left" }
        if hours > 0 { return "\(hours)h \(minutes)m \(secondsLeft)s left" }
        if minutes > 0 { return "\(minutes)m \(secondsLeft)s left" }
        return "\(seconds)s left"
    }
}

enum TrackerStyle {
    static let background = Color(red: 0.07, green: 0.08, blue: 0.10)
    static let surface = Color(red: 0.12, green: 0.14, blue: 0.17)
    static let accent = Color(red: 0.98, green: 0.77, blue: 0.36)
}

#Preview {
    ContentView(previewVillages: [
        Village(
            id: "main",
            tag: "#MAIN",
            label: "Main",
            exportTimestamp: Int(Date.now.timeIntervalSince1970),
            timedEntries: [
                TimedEntry(name: "Scattershot", entry: UpgradeEntry(data: 1000067, level: 3, timer: 84_500), base: "homeVillage"),
                TimedEntry(name: "Bowler", entry: UpgradeEntry(data: 4000022, level: 5, timer: 7_200), base: "homeVillage"),
                TimedEntry(name: "Hidden Tesla", entry: UpgradeEntry(data: 1000043, level: 4, timer: 3_600), base: "builderBase")
            ]
        ),
        Village(id: "alt", tag: "#ALT", label: "Alt", exportTimestamp: 0, timedEntries: [])
    ])
}

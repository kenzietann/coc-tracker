import Foundation
import UserNotifications

enum UpgradeReminders {
    private static let prefix = "coc-upgrade-"
    private static let leadTime: TimeInterval = 60

    static func requestPermission() async throws -> Bool {
        try await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])
    }

    static func synchronize(villages: [Village]) async throws {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()
        guard settings.authorizationStatus == .authorized ||
              settings.authorizationStatus == .provisional else { return }

        let now = Date.now
        let reminders = villages.flatMap { village in
            village.timedEntries.enumerated().compactMap { index, upgrade -> Reminder? in
                let reminderTime = Double(village.exportTimestamp) + Double(upgrade.entry.timer) - leadTime
                let date = Date(timeIntervalSince1970: reminderTime)
                guard date > now else { return nil }

                return Reminder(
                    id: "\(prefix)\(village.id)-\(village.exportTimestamp)-\(index)-\(upgrade.entry.timer)",
                    date: date,
                    village: village.displayName,
                    upgrade: upgrade.name ?? "An upgrade"
                )
            }
        }.sorted { $0.date < $1.date }

        let pending = await center.pendingNotificationRequests()
        let trackerRequests = pending.filter { $0.identifier.hasPrefix(prefix) }
        let pendingIDs = Set(trackerRequests.map(\.identifier))
        let desiredIDs = Set(reminders.map(\.id))
        center.removePendingNotificationRequests(withIdentifiers: Array(pendingIDs.subtracting(desiredIDs)))

        for reminder in reminders where !pendingIDs.contains(reminder.id) {
            let delay = reminder.date.timeIntervalSinceNow
            guard delay > 0 else { continue }

            let content = UNMutableNotificationContent()
            content.title = "\(reminder.village) · 1 minute left"
            content.body = "\(reminder.upgrade) is expected to finish in one minute."
            content.sound = .default

            let trigger = UNTimeIntervalNotificationTrigger(timeInterval: delay, repeats: false)
            try await center.add(UNNotificationRequest(identifier: reminder.id, content: content, trigger: trigger))
        }
    }

    private struct Reminder {
        let id: String
        let date: Date
        let village: String
        let upgrade: String
    }
}

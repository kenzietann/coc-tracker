//
//  Village.swift
//  frontend
//
//  Created by Kenzie Tandera on 2026/11/13.
//

struct UpgradeEntry: Decodable {
    let data: Int?
    let level: Int?
    let timer: Int
}

struct TimedEntry: Decodable {
    let name: String?
    let entry: UpgradeEntry
    var base: String? = nil

    var baseName: String? {
        switch base {
        case "homeVillage": return "Home Village"
        case "builderBase": return "Builder Base"
        default:
            guard let id = entry.data else { return nil }

            // Builder Base troops and heroes share artwork prefixes with Home Village.
            switch id {
            case 4000031...4000038, 4000041, 4000042, 4000070, 4000106,
                 28000003, 28000005:
                return "Builder Base"
            default:
                break
            }

            guard let artwork = ClashArtwork.byID[id] else { return nil }
            if artwork.hasPrefix("buildings-builder-base-") || artwork.hasPrefix("traps-builder-base-") {
                return "Builder Base"
            }

            let homePrefixes = [
                "buildings-home-village-", "buildings-seasonal-defense-", "traps-home-village-",
                "troops-", "heroes-", "spells-", "pets-", "equipment-", "guardians-", "helpers-"
            ]
            return homePrefixes.contains { artwork.hasPrefix($0) } ? "Home Village" : nil
        }
    }
}

struct Village: Decodable {
    let id: String
    let tag: String
    let label: String?
    let exportTimestamp: Int
    let timedEntries: [TimedEntry]

    var displayName: String {
        if let label, !label.isEmpty { return label }
        return tag
    }
}

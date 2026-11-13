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
}

struct Village: Decodable {
    let id: String
    let tag: String
    let label: String?
    let exportTimestamp: Int
    let timedEntries: [TimedEntry]
}

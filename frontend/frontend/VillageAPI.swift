//
//  VillageAPI.swift
//  frontend
//
//  Created by Kenzie Tandera on 2026/11/13.
//

import Foundation

struct VillageAPI {
    func fetchVillages() async throws -> [Village] {
        guard let url = URL(string: "http://127.0.0.1:4444/clash/villages") else {
            throw URLError(.badURL)
        }

        let (data, response) = try await URLSession.shared.data(from: url)

        guard let httpResponse = response as? HTTPURLResponse,
              (200...299).contains(httpResponse.statusCode) else {
            throw URLError(.badServerResponse)
        }

        return try JSONDecoder().decode([Village].self, from: data)
    }
}

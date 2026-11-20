//
//  VillageAPI.swift
//  frontend
//
//  Created by Kenzie Tandera on 2026/11/13.
//

import Foundation

struct VillageAPI {
    enum ImportError: LocalizedError {
        case invalidExport
        case newerExportExists
        case serverRejected

        var errorDescription: String? {
            switch self {
            case .invalidExport: "This export couldn’t be read. Copy fresh village data and try again."
            case .newerExportExists: "A newer export is already saved for this village."
            case .serverRejected: "The server couldn’t save this export. Try again."
            }
        }
    }

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
    
    func importVillage(json: String) async throws {
        guard let url = URL(string: "http://127.0.0.1:4444/clash/import") else {
            throw URLError(.badURL)
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = Data(json.utf8)
        
        let (_, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw ImportError.serverRejected
        }
        switch httpResponse.statusCode {
        case 200...299: return
        case 400: throw ImportError.invalidExport
        case 409: throw ImportError.newerExportExists
        default: throw ImportError.serverRejected
        }
    }
}

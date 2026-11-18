//
//  ImportVillageView.swift
//  frontend
//
//  Created by Kenzie Tandera on 2026/11/13.
//

import SwiftUI

struct ImportVillageView: View {
    
    @State private var exportJSON: String = ""
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        VStack {
            Text("Paste your village export")
            
            TextEditor(text: $exportJSON)
                .frame(height: 250)
                .border(.gray)
            
            Button("Import") {
                Task {
                    do {
                        try await VillageAPI().importVillage(json: exportJSON)
                        dismiss()
                        print("Village imported!")
                    } catch {
                        print("Import failed:", error)
                    }
                }
            }
            
        }
        .padding()
        
    }
}

#Preview {
    ImportVillageView()
}

import SwiftUI

struct ImportVillageView: View {
    @State private var exportJSON = ""
    @State private var isImporting = false
    @State private var errorMessage: String?
    @Environment(\.dismiss) private var dismiss

    var onImported: () async -> Void = {}

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Import village")
                .font(.title2.bold())
                .accessibilityAddTraits(.isHeader)

            Text("Paste the JSON export copied from Clash of Clans.")
                .foregroundStyle(.secondary)

            TextEditor(text: $exportJSON)
                .font(.system(.body, design: .monospaced))
                .scrollContentBackground(.hidden)
                .padding(8)
                .frame(minHeight: 140, idealHeight: 220)
                .background(TrackerStyle.surface, in: RoundedRectangle(cornerRadius: 10))
                .accessibilityLabel("Village export JSON")
                .disabled(isImporting)

            if let errorMessage {
                Label(errorMessage, systemImage: "exclamationmark.circle")
                    .font(.subheadline)
                    .foregroundStyle(.orange)
            }

            HStack {
                Button("Cancel") { dismiss() }
                    .disabled(isImporting)
                Spacer()
                Button(isImporting ? "Importing…" : "Import village") {
                    isImporting = true
                    Task { await submit() }
                }
                .buttonStyle(.borderedProminent)
                .disabled(isImporting || exportJSON.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
            .controlSize(.large)
        }
        .padding(24)
        #if os(macOS)
        .frame(minWidth: 420, idealWidth: 520)
        #endif
        .background(TrackerStyle.background.ignoresSafeArea())
        .interactiveDismissDisabled(isImporting)
    }

    private func submit() async {
        errorMessage = nil
        defer { isImporting = false }
        do {
            try await VillageAPI().importVillage(json: exportJSON)
            await onImported()
            dismiss()
        } catch {
            errorMessage = (error as? VillageAPI.ImportError)?.errorDescription
                ?? "Couldn’t reach the backend. Check that it’s running and try again."
        }
    }
}

#Preview {
    ImportVillageView()
        .preferredColorScheme(.dark)
        .tint(TrackerStyle.accent)
}

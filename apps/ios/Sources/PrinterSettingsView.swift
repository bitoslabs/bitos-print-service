import SwiftUI

public struct PrinterSettingsView: View {
    @ObservedObject private var store: PrinterStore
    private let transport = TcpPrinterTransport()

    @State private var showAddSheet = false
    @State private var statusMessage: String?
    @State private var newName = ""
    @State private var newIp = ""
    @State private var newPort = "9100"
    @State private var newWidth = 80

    public init(store: PrinterStore) {
        self.store = store
    }

    public var body: some View {
        NavigationView {
            List {
                if let statusMessage {
                    Section {
                        Text(statusMessage)
                            .font(.footnote)
                            .foregroundColor(.secondary)
                    }
                }

                Section(header: Text("Configured IP Printers")) {
                    if store.printers.isEmpty {
                        Text("No printers added. Tap + to add an Xprinter IP.")
                            .foregroundColor(.secondary)
                    } else {
                        ForEach(store.printers) { printer in
                            HStack {
                                VStack(alignment: .leading, spacing: 4) {
                                    HStack {
                                        Text(printer.name).font(.headline)
                                        if printer.isDefault {
                                            Text("Default")
                                                .font(.caption2)
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(Color.green.opacity(0.2))
                                                .cornerRadius(4)
                                        }
                                    }
                                    Text("\(printer.ip):\(printer.port) · \(printer.paperWidthMm)mm")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                                Spacer()
                                Button("Test") {
                                    runTestPrint(printer)
                                }
                                .buttonStyle(.bordered)
                            }
                            .contentShape(Rectangle())
                            .onTapGesture {
                                store.setDefault(id: printer.id)
                            }
                        }
                        .onDelete { offsets in
                            for idx in offsets {
                                store.remove(id: store.printers[idx].id)
                            }
                        }
                    }
                }
            }
            .navigationTitle("BNOS Print")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button(action: { showAddSheet = true }) {
                        Image(systemName: "plus")
                    }
                }
            }
            .sheet(isPresented: $showAddSheet) {
                NavigationView {
                    Form {
                        TextField("Printer Name (e.g. Cashier)", text: $newName)
                        TextField("IP Address (e.g. 192.168.2.100)", text: $newIp)
                        TextField("Port", text: $newPort)
                        Picker("Paper Width", selection: $newWidth) {
                            Text("80mm").tag(80)
                            Text("58mm").tag(58)
                        }
                    }
                    .navigationTitle("Add IP Printer")
                    .toolbar {
                        ToolbarItem(placement: .cancellationAction) {
                            Button("Cancel") { showAddSheet = false }
                        }
                        ToolbarItem(placement: .confirmationAction) {
                            Button("Save") {
                                store.add(
                                    name: newName,
                                    ip: newIp,
                                    port: UInt16(newPort) ?? 9100,
                                    paperWidthMm: newWidth
                                )
                                newName = ""
                                newIp = ""
                                newPort = "9100"
                                showAddSheet = false
                            }
                            .disabled(newIp.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        }
                    }
                }
            }
        }
    }

    private func runTestPrint(_ printer: PrinterConfig) {
        statusMessage = "Sending test print to \(printer.ip):\(printer.port)..."
        let payload = Data([0x1B, 0x40, 0x1C, 0x2E, 0x1B, 0x74, 0x00])
            + Data("BNOS Print Service Test OK\n\(printer.name)\n\n\n\n".utf8)
            + Data([0x1D, 0x56, 0x00])

        transport.send(host: printer.ip, port: printer.port, payload: payload) { result in
            switch result {
            case .success(let bytes):
                statusMessage = "Test print OK (\(bytes) bytes) -> \(printer.name)"
            case .failure(let error):
                statusMessage = "Test print failed: \(error.localizedDescription)"
            }
        }
    }
}

import Foundation
import Combine

public struct PrinterConfig: Identifiable, Codable, Equatable {
    public let id: String
    public var name: String
    public var ip: String
    public var port: UInt16
    public var paperWidthMm: Int
    public var isDefault: Bool

    public init(
        id: String = UUID().uuidString,
        name: String,
        ip: String,
        port: UInt16 = 9100,
        paperWidthMm: Int = 80,
        isDefault: Bool = false
    ) {
        self.id = id
        self.name = name
        self.ip = ip
        self.port = port
        self.paperWidthMm = paperWidthMm
        self.isDefault = isDefault
    }
}

public final class PrinterStore: ObservableObject {
    @Published public private(set) var printers: [PrinterConfig] = []

    private let storageKey = "space.bnos.print.printers"
    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        load()
    }

    public func add(name: String, ip: String, port: UInt16 = 9100, paperWidthMm: Int = 80) {
        let trimmedIp = ip.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedIp.isEmpty else { return }
        let trimmedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
        let isFirst = printers.isEmpty
        printers.append(
            PrinterConfig(
                name: trimmedName.isEmpty ? "Xprinter LAN" : trimmedName,
                ip: trimmedIp,
                port: port,
                paperWidthMm: paperWidthMm,
                isDefault: isFirst
            )
        )
        save()
    }

    public func remove(id: String) {
        printers.removeAll { $0.id == id }
        if !printers.contains(where: { $0.isDefault }), !printers.isEmpty {
            printers[0].isDefault = true
        }
        save()
    }

    public func setDefault(id: String) {
        for idx in printers.indices {
            printers[idx].isDefault = (printers[idx].id == id)
        }
        save()
    }

    public func resolveTarget(printerId: String? = nil, ip: String? = nil, port: UInt16? = nil) -> (String, UInt16)? {
        if let ip, !ip.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return (ip.trimmingCharacters(in: .whitespacesAndNewlines), port ?? 9100)
        }
        if let printerId, let found = printers.first(where: { $0.id == printerId }) {
            return (found.ip, port ?? found.port)
        }
        if let def = printers.first(where: { $0.isDefault }) ?? printers.first {
            return (def.ip, port ?? def.port)
        }
        return nil
    }

    private func load() {
        guard let data = defaults.data(forKey: storageKey),
              let decoded = try? JSONDecoder().decode([PrinterConfig].self, from: data) else {
            printers = []
            return
        }
        printers = decoded
    }

    private func save() {
        if let data = try? JSONEncoder().encode(printers) {
            defaults.set(data, forKey: storageKey)
        }
    }
}

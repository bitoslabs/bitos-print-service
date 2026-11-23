import Foundation

struct DesktopPrinter: Codable {
    let id: String
    let name: String
    let ip: String
    let port: Int
    let paperWidthMm: Int
    let isDefault: Bool
}

private struct PrintersResponse: Codable {
    let ok: Bool
    let printers: [DesktopPrinter]
}

private struct PrintResultResponse: Codable {
    let ok: Bool
    let bytes: Int?
    let error: String?
}

final class DaemonClient {
    let baseURL = URL(string: "http://127.0.0.1:3100")!

    func checkHealth(completion: @escaping (Bool) -> Void) {
        var req = URLRequest(url: baseURL.appendingPathComponent("health"))
        req.timeoutInterval = 2
        URLSession.shared.dataTask(with: req) { _, resp, _ in
            let ok = (resp as? HTTPURLResponse)?.statusCode == 200
            DispatchQueue.main.async { completion(ok) }
        }.resume()
    }

    func listPrinters(completion: @escaping ([DesktopPrinter]) -> Void) {
        URLSession.shared.dataTask(with: baseURL.appendingPathComponent("printers")) { data, _, _ in
            let list = data
                .flatMap { try? JSONDecoder().decode(PrintersResponse.self, from: $0) }?
                .printers ?? []
            DispatchQueue.main.async { completion(list) }
        }.resume()
    }

    func addPrinter(name: String, ip: String, port: Int, paperWidthMm: Int, completion: @escaping (Bool) -> Void) {
        var req = URLRequest(url: baseURL.appendingPathComponent("printers"))
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        let body: [String: Any] = ["name": name, "ip": ip, "port": port, "paperWidthMm": paperWidthMm]
        req.httpBody = try? JSONSerialization.data(withJSONObject: body)
        URLSession.shared.dataTask(with: req) { _, resp, _ in
            let ok = (resp as? HTTPURLResponse)?.statusCode == 201
            DispatchQueue.main.async { completion(ok) }
        }.resume()
    }

    func setDefault(id: String, completion: @escaping () -> Void) {
        var req = URLRequest(url: baseURL.appendingPathComponent("printers/\(id)/default"))
        req.httpMethod = "POST"
        URLSession.shared.dataTask(with: req) { _, _, _ in
            DispatchQueue.main.async { completion() }
        }.resume()
    }

    func removePrinter(id: String, completion: @escaping () -> Void) {
        var req = URLRequest(url: baseURL.appendingPathComponent("printers/\(id)"))
        req.httpMethod = "DELETE"
        URLSession.shared.dataTask(with: req) { _, _, _ in
            DispatchQueue.main.async { completion() }
        }.resume()
    }

    func testPrint(printerId: String, completion: @escaping (String) -> Void) {
        var req = URLRequest(url: baseURL.appendingPathComponent("print"))
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try? JSONSerialization.data(withJSONObject: ["type": "test", "printerId": printerId])
        URLSession.shared.dataTask(with: req) { data, _, err in
            let message: String
            if let err {
                message = "Error: \(err.localizedDescription)"
            } else if let data, let parsed = try? JSONDecoder().decode(PrintResultResponse.self, from: data) {
                message = parsed.ok ? "Test print OK (\(parsed.bytes ?? 0) bytes)" : "Failed: \(parsed.error ?? "Unknown")"
            } else {
                message = "Daemon unreachable on 127.0.0.1:3100"
            }
            DispatchQueue.main.async { completion(message) }
        }.resume()
    }
}

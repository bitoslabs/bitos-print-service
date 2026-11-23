import Foundation

public struct ParsedDeeplinkJob {
    public let action: String
    public let jobType: String
    public let ip: String?
    public let port: UInt16
    public let payload: Data
    public let callback: URL?
    public let silent: Bool
}

public enum DeeplinkError: Error {
    case invalidUrl
    case disallowedCallback
    case invalidBase64
}

public enum DeeplinkRouter {
    public static func isAllowedCallback(_ url: URL) -> Bool {
        guard let host = url.host?.lowercased(), let scheme = url.scheme?.lowercased() else {
            return false
        }
        if (host == "localhost" || host == "127.0.0.1") && (scheme == "http" || scheme == "https") {
            return true
        }
        let isBnosOrBdgo = host == "bnos.space"
            || host.hasSuffix(".bnos.space")
            || host == "bdgo.vn"
            || host.hasSuffix(".bdgo.vn")
        return isBnosOrBdgo && scheme == "https"
    }

    public static func parse(url: URL) throws -> ParsedDeeplinkJob {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            throw DeeplinkError.invalidUrl
        }
        let query = Dictionary(
            uniqueKeysWithValues: (components.queryItems ?? []).compactMap { item in
                item.value.map { (item.name, $0) }
            }
        )

        var callbackUrl: URL?
        if let rawCallback = query["callback"], let parsed = URL(string: rawCallback) {
            guard isAllowedCallback(parsed) else {
                throw DeeplinkError.disallowedCallback
            }
            callbackUrl = parsed
        }

        let jobType = query["type"] ?? "raw"
        let port = UInt16(query["port"] ?? "9100") ?? 9100
        let payload: Data
        if jobType == "test" {
            payload = Data([0x1B, 0x40, 0x1C, 0x2E, 0x1B, 0x74, 0x00])
                + Data("BNOS Print Service Test OK\n\n\n\n".utf8)
                + Data([0x1D, 0x56, 0x00])
        } else if let b64Url = query["data"] {
            payload = try decodeBase64Url(b64Url)
        } else {
            payload = Data()
        }

        return ParsedDeeplinkJob(
            action: url.host ?? "print",
            jobType: jobType,
            ip: query["ip"],
            port: port,
            payload: payload,
            callback: callbackUrl,
            silent: query["silent"] == "1" || query["silent"]?.lowercased() == "true"
        )
    }

    private static func decodeBase64Url(_ input: String) throws -> Data {
        var s = input.replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")
        let remainder = s.count % 4
        if remainder > 0 {
            s.append(String(repeating: "=", count: 4 - remainder))
        }
        guard let data = Data(base64Encoded: s) else {
            throw DeeplinkError.invalidBase64
        }
        return data
    }
}

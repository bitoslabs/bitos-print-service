import Foundation
import Network

public enum TcpPrinterError: Error {
    case invalidEndpoint
    case timeout
    case connectionFailed(String)
}

/// Sends raw ESC/POS bytes over TCP port 9100 using Apple's Network.framework.
public final class TcpPrinterTransport {
    private let queue = DispatchQueue(label: "space.bnos.print.tcp")

    public init() {}

    public func send(
        host: String,
        port: UInt16 = 9100,
        payload: Data,
        timeoutSeconds: TimeInterval = 12,
        completion: @escaping (Result<Int, Error>) -> Void
    ) {
        guard let nwPort = NWEndpoint.Port(rawValue: port) else {
            completion(.failure(TcpPrinterError.invalidEndpoint))
            return
        }

        let tcpOptions = NWProtocolTCP.Options()
        tcpOptions.noDelay = true
        let params = NWParameters(tls: nil, tcp: tcpOptions)
        let connection = NWConnection(
            host: NWEndpoint.Host(host),
            port: nwPort,
            using: params
        )

        var finished = false
        let finish: (Result<Int, Error>) -> Void = { result in
            guard !finished else { return }
            finished = true
            connection.cancel()
            DispatchQueue.main.async {
                completion(result)
            }
        }

        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.schedule(deadline: .now() + timeoutSeconds)
        timer.setEventHandler {
            finish(.failure(TcpPrinterError.timeout))
        }
        timer.resume()

        connection.stateUpdateHandler = { state in
            switch state {
            case .ready:
                connection.send(
                    content: payload,
                    completion: .contentProcessed { error in
                        timer.cancel()
                        if let error {
                            finish(.failure(error))
                        } else {
                            finish(.success(payload.count))
                        }
                    }
                )
            case .failed(let error):
                timer.cancel()
                finish(.failure(TcpPrinterError.connectionFailed(error.localizedDescription)))
            default:
                break
            }
        }

        connection.start(queue: queue)
    }
}

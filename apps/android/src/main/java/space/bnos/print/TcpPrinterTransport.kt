package space.bnos.print

import java.net.InetSocketAddress
import java.net.Socket

class TcpPrinterTransport {
    fun send(ip: String, port: Int = 9100, payload: ByteArray, timeoutMs: Int = 12_000): Int {
        Socket().use { socket ->
            socket.tcpNoDelay = true
            socket.soTimeout = timeoutMs
            socket.connect(InetSocketAddress(ip, port), timeoutMs)
            val out = socket.getOutputStream()
            out.write(payload)
            out.flush()
        }
        return payload.size
    }
}

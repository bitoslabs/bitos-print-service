package space.bnos.print

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.util.Base64
import java.io.ByteArrayInputStream
import java.util.concurrent.Executors
import java.util.zip.GZIPInputStream

class DeeplinkActivity : Activity() {
    private val executor = Executors.newSingleThreadExecutor()
    private val tcpTransport = TcpPrinterTransport()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        handleIntent(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent?) {
        val uri = intent?.data ?: return finish()
        val store = PrinterStore(this)
        val jobType = uri.getQueryParameter("type") ?: "raw"
        val printerId = uri.getQueryParameter("printerId")
        val rawIp = uri.getQueryParameter("ip")
        val rawPort = uri.getQueryParameter("port")?.toIntOrNull()
        val (ip, port) = store.resolveTarget(printerId, rawIp, rawPort) ?: ("192.168.2.100" to 9100)
        val callbackRaw = uri.getQueryParameter("callback")
        val callbackUri = callbackRaw?.let { Uri.parse(it) }?.takeIf { isAllowedCallback(it) }

        val payload = when {
            jobType == "test" -> byteArrayOf(0x1B, 0x40, 0x1C, 0x2E, 0x1B, 0x74, 0x00) +
                "BNOS Print Service Test OK\n\n\n\n".toByteArray(Charsets.UTF_8) +
                byteArrayOf(0x1D, 0x56, 0x00)
            else -> {
                val dataParam = uri.getQueryParameter("data") ?: ""
                val decoded = Base64.decode(dataParam, Base64.URL_SAFE or Base64.NO_WRAP or Base64.NO_PADDING)
                if (uri.getQueryParameter("enc") == "gzip") {
                    GZIPInputStream(ByteArrayInputStream(decoded)).use { it.readBytes() }
                } else {
                    decoded
                }
            }
        }

        executor.execute {
            val resultUri = try {
                tcpTransport.send(ip, port, payload)
                callbackUri?.buildUpon()?.appendQueryParameter("ok", "1")?.build()
            } catch (e: Exception) {
                callbackUri?.buildUpon()
                    ?.appendQueryParameter("ok", "0")
                    ?.appendQueryParameter("error", e.message ?: "Print failed")
                    ?.build()
            }

            runOnUiThread {
                if (resultUri != null) {
                    startActivity(Intent(Intent.ACTION_VIEW, resultUri))
                }
                finish()
            }
        }
    }

    private fun isAllowedCallback(uri: Uri): Boolean {
        val host = uri.host?.lowercase() ?: return false
        val scheme = uri.scheme?.lowercase() ?: return false
        if ((host == "localhost" || host == "127.0.0.1") && (scheme == "http" || scheme == "https")) {
            return true
        }
        val allowed = host == "bnos.space" || host.endsWith(".bnos.space") ||
            host == "bdgo.vn" || host.endsWith(".bdgo.vn")
        return allowed && scheme == "https"
    }
}

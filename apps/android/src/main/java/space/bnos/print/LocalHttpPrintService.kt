package space.bnos.print

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Base64
import org.json.JSONArray
import org.json.JSONObject
import java.io.BufferedInputStream
import java.io.ByteArrayOutputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.net.URLDecoder
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

class LocalHttpPrintService : Service() {
    private val running = AtomicBoolean(false)
    private var serverSocket: ServerSocket? = null
    private val acceptExecutor = Executors.newSingleThreadExecutor()
    private val workerExecutor = Executors.newCachedThreadPool()
    private val transport = TcpPrinterTransport()

    override fun onCreate() {
        super.onCreate()
        startInForeground()
        startServer()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (!running.get()) {
            startServer()
        }
        return START_STICKY
    }

    override fun onDestroy() {
        running.set(false)
        try {
            serverSocket?.close()
        } catch (_: Exception) {
        }
        acceptExecutor.shutdownNow()
        workerExecutor.shutdownNow()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun startInForeground() {
        val channelId = "bnos_print_http"
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            val channel = NotificationChannel(
                channelId,
                "BNOS Print Local Service",
                NotificationManager.IMPORTANCE_LOW
            )
            manager.createNotificationChannel(channel)
        }

        val launchIntent = Intent(this, PrinterSettingsActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            launchIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, channelId)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
        }

        val notification = builder
            .setContentTitle("BNOS Print Service")
            .setContentText("Listening on http://127.0.0.1:3100")
            .setSmallIcon(android.R.drawable.ic_menu_send)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(1001, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(1001, notification)
        }
    }

    private fun startServer() {
        if (!running.compareAndSet(false, true)) return
        acceptExecutor.execute {
            try {
                val socket = ServerSocket(3100, 50, InetAddress.getByName("127.0.0.1")).apply {
                    reuseAddress = true
                }
                serverSocket = socket
                while (running.get()) {
                    val client = socket.accept()
                    workerExecutor.execute { handleClient(client) }
                }
            } catch (_: Exception) {
                running.set(false)
            }
        }
    }

    private fun handleClient(client: Socket) {
        client.use { socket ->
            socket.soTimeout = 10_000
            val input = BufferedInputStream(socket.getInputStream())
            val requestLine = readLine(input) ?: return
            val parts = requestLine.split(" ")
            if (parts.size < 2) return

            val method = parts[0].uppercase()
            val rawTarget = parts[1]
            val pathParts = rawTarget.split("?", limit = 2)
            val path = pathParts[0]
            val query = parseQuery(pathParts.getOrNull(1).orEmpty())

            val headers = mutableMapOf<String, String>()
            while (true) {
                val line = readLine(input) ?: break
                if (line.isEmpty()) break
                val idx = line.indexOf(':')
                if (idx > 0) {
                    val key = line.substring(0, idx).trim().lowercase()
                    val value = line.substring(idx + 1).trim()
                    headers[key] = value
                }
            }

            val contentLength = headers["content-length"]?.toIntOrNull() ?: 0
            val body = if (contentLength > 0) readExact(input, contentLength) else ByteArray(0)

            val response = when {
                method == "OPTIONS" -> HttpResponse(204, ByteArray(0))
                method == "GET" && path == "/health" -> {
                    val json = JSONObject().apply {
                        put("ok", true)
                        put("service", "bnos-print-android")
                        put("version", "0.1.0")
                        put("platform", "android")
                    }
                    jsonResponse(200, json.toString())
                }
                method == "GET" && path == "/printers" -> {
                    val store = PrinterStore(this)
                    val arr = JSONArray()
                    for (p in store.list()) {
                        arr.put(
                            JSONObject().apply {
                                put("id", p.id)
                                put("name", p.name)
                                put("transport", "tcp")
                                put("ip", p.ip)
                                put("port", p.port)
                                put("address", "${p.ip}:${p.port}")
                                put("protocol", "escpos")
                                put("paperWidthMm", p.paperWidthMm)
                                put("isDefault", p.isDefault)
                            }
                        )
                    }
                    jsonResponse(200, arr.toString())
                }
                method == "POST" && path == "/print" -> handlePrint(headers, query, body)
                else -> jsonResponse(404, JSONObject().put("ok", false).put("error", "Not found").toString())
            }

            writeResponse(socket, response)
        }
    }

    private fun handlePrint(
        headers: Map<String, String>,
        query: Map<String, String>,
        body: ByteArray
    ): HttpResponse {
        val store = PrinterStore(this)
        val contentType = headers["content-type"].orEmpty().lowercase()

        var printerId = query["printerId"]
        var ip = query["ip"]
        var port = query["port"]?.toIntOrNull()
        val payload: ByteArray

        if (contentType.contains("application/octet-stream")) {
            payload = body
        } else {
            val rawText = body.toString(Charsets.UTF_8).trim()
            if (rawText.startsWith("{")) {
                val obj = try {
                    JSONObject(rawText)
                } catch (e: Exception) {
                    return jsonResponse(400, JSONObject().put("ok", false).put("error", "Invalid JSON").toString())
                }
                if (printerId.isNullOrBlank() && obj.has("printerId")) {
                    printerId = obj.optString("printerId")
                }
                if (ip.isNullOrBlank() && obj.has("ip")) {
                    ip = obj.optString("ip")
                }
                if (port == null && obj.has("port")) {
                    port = obj.optInt("port", 9100)
                }
                val type = obj.optString("type", "")
                val dataBase64 = obj.optString("dataBase64", "")
                payload = when {
                    type == "test" -> buildTestPayload()
                    dataBase64.isNotEmpty() -> Base64.decode(dataBase64, Base64.DEFAULT)
                    else -> return jsonResponse(
                        400,
                        JSONObject().put("ok", false).put("error", "Missing dataBase64 or type=test").toString()
                    )
                }
            } else if (body.isNotEmpty()) {
                payload = body
            } else {
                return jsonResponse(400, JSONObject().put("ok", false).put("error", "Empty print body").toString())
            }
        }

        val (targetIp, targetPort) = store.resolveTarget(printerId, ip, port) ?: ("192.168.2.100" to 9100)
        return try {
            val bytes = transport.send(targetIp, targetPort, payload)
            jsonResponse(200, JSONObject().put("ok", true).put("bytes", bytes).toString())
        } catch (e: Exception) {
            jsonResponse(
                502,
                JSONObject().put("ok", false).put("error", e.message ?: "Print failed").toString()
            )
        }
    }

    private fun buildTestPayload(): ByteArray {
        return byteArrayOf(0x1B, 0x40, 0x1C, 0x2E, 0x1B, 0x74, 0x00) +
            "BNOS Print Service Test OK\nAndroid 127.0.0.1:3100\n\n\n\n".toByteArray(Charsets.UTF_8) +
            byteArrayOf(0x1D, 0x56, 0x00)
    }

    private fun jsonResponse(status: Int, json: String): HttpResponse {
        return HttpResponse(status, json.toByteArray(Charsets.UTF_8), "application/json; charset=utf-8")
    }

    private fun writeResponse(socket: Socket, resp: HttpResponse) {
        val statusText = when (resp.status) {
            200 -> "OK"
            204 -> "No Content"
            400 -> "Bad Request"
            404 -> "Not Found"
            502 -> "Bad Gateway"
            else -> "Error"
        }
        val header = buildString {
            append("HTTP/1.1 ${resp.status} $statusText\r\n")
            append("Access-Control-Allow-Origin: *\r\n")
            append("Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS\r\n")
            append("Access-Control-Allow-Headers: Content-Type, Authorization\r\n")
            append("Access-Control-Allow-Private-Network: true\r\n")
            append("Connection: close\r\n")
            if (resp.contentType != null) {
                append("Content-Type: ${resp.contentType}\r\n")
            }
            append("Content-Length: ${resp.body.size}\r\n\r\n")
        }
        val out = socket.getOutputStream()
        out.write(header.toByteArray(Charsets.US_ASCII))
        if (resp.body.isNotEmpty()) {
            out.write(resp.body)
        }
        out.flush()
    }

    private fun readLine(input: BufferedInputStream): String? {
        val out = ByteArrayOutputStream()
        var prev = -1
        while (true) {
            val b = input.read()
            if (b == -1) {
                return if (out.size() == 0) null else out.toString("US-ASCII")
            }
            if (b == '\n'.code) {
                val bytes = out.toByteArray()
                val len = if (prev == '\r'.code && bytes.isNotEmpty()) bytes.size - 1 else bytes.size
                return String(bytes, 0, len, Charsets.US_ASCII)
            }
            out.write(b)
            prev = b
        }
    }

    private fun readExact(input: BufferedInputStream, length: Int): ByteArray {
        val buf = ByteArray(length)
        var offset = 0
        while (offset < length) {
            val read = input.read(buf, offset, length - offset)
            if (read <= 0) break
            offset += read
        }
        return if (offset == length) buf else buf.copyOf(offset)
    }

    private fun parseQuery(raw: String): Map<String, String> {
        if (raw.isEmpty()) return emptyMap()
        val result = mutableMapOf<String, String>()
        for (pair in raw.split("&")) {
            val idx = pair.indexOf('=')
            if (idx > 0) {
                val k = URLDecoder.decode(pair.substring(0, idx), "UTF-8")
                val v = URLDecoder.decode(pair.substring(idx + 1), "UTF-8")
                result[k] = v
            }
        }
        return result
    }

    private data class HttpResponse(
        val status: Int,
        val body: ByteArray,
        val contentType: String? = null
    )

    companion object {
        fun start(context: Context) {
            val intent = Intent(context, LocalHttpPrintService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }
    }
}

package space.bnos.print

import android.app.Activity
import android.app.AlertDialog
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ListView
import android.widget.TextView
import java.util.concurrent.Executors

class PrinterSettingsActivity : Activity() {
    private lateinit var store: PrinterStore
    private lateinit var statusText: TextView
    private lateinit var listView: ListView
    private val transport = TcpPrinterTransport()
    private val executor = Executors.newSingleThreadExecutor()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        store = PrinterStore(this)
        LocalHttpPrintService.start(this)

        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 32, 32, 32)
        }

        statusText = TextView(this).apply {
            text = "BNOS Print · Local HTTP: http://127.0.0.1:3100"
            textSize = 14f
        }

        val addButton = Button(this).apply {
            text = "+ Add IP Printer"
            setOnClickListener { showAddDialog() }
        }

        listView = ListView(this)
        root.addView(statusText)
        root.addView(addButton)
        root.addView(listView)
        setContentView(root)

        refreshList()
    }

    private fun refreshList() {
        val printers = store.list()
        val labels = printers.map { p ->
            val defTag = if (p.isDefault) " [Default]" else ""
            "${p.name}$defTag\n${p.ip}:${p.port} · ${p.paperWidthMm}mm"
        }
        listView.adapter = ArrayAdapter(this, android.R.layout.simple_list_item_1, labels)
        listView.setOnItemClickListener { _, _, position, _ ->
            showPrinterActions(printers[position])
        }
    }

    private fun showAddDialog() {
        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 16, 32, 0)
        }
        val nameInput = EditText(this).apply { hint = "Printer Name (e.g. Cashier)" }
        val ipInput = EditText(this).apply { hint = "IP Address (e.g. 192.168.2.100)" }
        val portInput = EditText(this).apply { setText("9100") }
        layout.addView(nameInput)
        layout.addView(ipInput)
        layout.addView(portInput)

        AlertDialog.Builder(this)
            .setTitle("Add IP Printer")
            .setView(layout)
            .setPositiveButton("Save") { _, _ ->
                val ip = ipInput.text.toString().trim()
                if (ip.isNotEmpty()) {
                    store.add(
                        name = nameInput.text.toString(),
                        ip = ip,
                        port = portInput.text.toString().toIntOrNull() ?: 9100
                    )
                    refreshList()
                }
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun showPrinterActions(printer: PrinterConfig) {
        val options = arrayOf(
            "Test Print",
            "Copy Local Webhook (127.0.0.1:3100)",
            "Copy Deeplink (bnos-print://)",
            "Copy Web BNOS Config (JSON)",
            "Set as Default",
            "Delete"
        )
        AlertDialog.Builder(this)
            .setTitle("${printer.name} (${printer.ip}:${printer.port})")
            .setItems(options) { _, which ->
                when (which) {
                    0 -> runTestPrint(printer)
                    1 -> copyToClipboard(
                        "BNOS Local Webhook",
                        "http://127.0.0.1:3100/print?ip=${printer.ip}&port=${printer.port}",
                        "Copied local webhook for ${printer.name}"
                    )
                    2 -> copyToClipboard(
                        "BNOS Print Deeplink",
                        "bnos-print://print?ip=${printer.ip}&port=${printer.port}",
                        "Copied deeplink for ${printer.name}"
                    )
                    3 -> copyToClipboard(
                        "BNOS Web Config",
                        """{"name":"${printer.name}","ip":"${printer.ip}","port":${printer.port},"paperWidthMm":${printer.paperWidthMm},"webhookUrl":"http://127.0.0.1:3100/print?ip=${printer.ip}&port=${printer.port}","deeplink":"bnos-print://print?ip=${printer.ip}&port=${printer.port}"}""",
                        "Copied Web BNOS config for ${printer.name}"
                    )
                    4 -> {
                        store.setDefault(printer.id)
                        refreshList()
                    }
                    5 -> {
                        store.remove(printer.id)
                        refreshList()
                    }
                }
            }
            .show()
    }

    private fun copyToClipboard(label: String, text: String, status: String) {
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText(label, text))
        statusText.text = "$status:\n$text"
    }

    private fun runTestPrint(printer: PrinterConfig) {
        statusText.text = "Testing ${printer.ip}:${printer.port}..."
        val payload = byteArrayOf(0x1B, 0x40, 0x1C, 0x2E, 0x1B, 0x74, 0x00) +
            "BNOS Print Service Test OK\n${printer.name}\n\n\n\n".toByteArray(Charsets.UTF_8) +
            byteArrayOf(0x1D, 0x56, 0x00)

        executor.execute {
            val msg = try {
                val bytes = transport.send(printer.ip, printer.port, payload)
                "Test print OK ($bytes bytes) -> ${printer.name}"
            } catch (e: Exception) {
                "Test print failed: ${e.message}"
            }
            runOnUiThread { statusText.text = msg }
        }
    }
}

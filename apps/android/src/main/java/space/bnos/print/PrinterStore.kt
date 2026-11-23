package space.bnos.print

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID

data class PrinterConfig(
    val id: String,
    val name: String,
    val ip: String,
    val port: Int = 9100,
    val paperWidthMm: Int = 80,
    val isDefault: Boolean = false
)

class PrinterStore(context: Context) {
    private val prefs = context.getSharedPreferences("bnos_print_printers", Context.MODE_PRIVATE)
    private val key = "printers_json"

    fun list(): List<PrinterConfig> {
        val raw = prefs.getString(key, null) ?: return emptyList()
        val arr = JSONArray(raw)
        val result = mutableListOf<PrinterConfig>()
        for (i in 0 until arr.length()) {
            val obj = arr.getJSONObject(i)
            result.add(
                PrinterConfig(
                    id = obj.getString("id"),
                    name = obj.getString("name"),
                    ip = obj.getString("ip"),
                    port = obj.optInt("port", 9100),
                    paperWidthMm = obj.optInt("paperWidthMm", 80),
                    isDefault = obj.optBoolean("isDefault", false)
                )
            )
        }
        return result
    }

    fun add(name: String, ip: String, port: Int = 9100, paperWidthMm: Int = 80): PrinterConfig {
        val current = list().toMutableList()
        val item = PrinterConfig(
            id = UUID.randomUUID().toString(),
            name = name.trim().ifEmpty { "Xprinter LAN" },
            ip = ip.trim(),
            port = port,
            paperWidthMm = paperWidthMm,
            isDefault = current.isEmpty()
        )
        current.add(item)
        save(current)
        return item
    }

    fun setDefault(id: String) {
        val updated = list().map { it.copy(isDefault = it.id == id) }
        save(updated)
    }

    fun remove(id: String) {
        val updated = list().filterNot { it.id == id }.toMutableList()
        if (updated.isNotEmpty() && updated.none { it.isDefault }) {
            updated[0] = updated[0].copy(isDefault = true)
        }
        save(updated)
    }

    fun resolveTarget(printerId: String?, ip: String?, port: Int?): Pair<String, Int>? {
        val trimmedIp = ip?.trim().orEmpty()
        if (trimmedIp.isNotEmpty()) {
            return trimmedIp to (port ?: 9100)
        }
        val all = list()
        val matched = printerId?.let { id -> all.firstOrNull { it.id == id } }
            ?: all.firstOrNull { it.isDefault }
            ?: all.firstOrNull()
        return matched?.let { it.ip to (port ?: it.port) }
    }

    private fun save(items: List<PrinterConfig>) {
        val arr = JSONArray()
        for (item in items) {
            arr.put(
                JSONObject().apply {
                    put("id", item.id)
                    put("name", item.name)
                    put("ip", item.ip)
                    put("port", item.port)
                    put("paperWidthMm", item.paperWidthMm)
                    put("isDefault", item.isDefault)
                }
            )
        }
        prefs.edit().putString(key, arr.toString()).apply()
    }
}

import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate, NSTableViewDataSource, NSTableViewDelegate {
    private let client = DaemonClient()
    private var statusItem: NSStatusItem!
    private var window: NSWindow?
    private var printers: [DesktopPrinter] = []
    private var daemonProcess: Process?
    private var allowLan: Bool {
        get { UserDefaults.standard.bool(forKey: "space.bnos.print.allowLan") }
        set { UserDefaults.standard.set(newValue, forKey: "space.bnos.print.allowLan") }
    }

    private let statusLabel = NSTextField(labelWithString: "Checking daemon...")
    private let nameField = NSTextField(string: "")
    private let ipField = NSTextField(string: "")
    private let portField = NSTextField(string: "9100")
    private let widthPopup = NSPopUpButton()
    private let tableView = NSTableView()

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            button.image = NSImage(systemSymbolName: "printer.fill", accessibilityDescription: "BNOS Print")
            if button.image == nil {
                button.title = "BNOS"
            }
        }
        rebuildMenu()
        ensureBundledDaemonRunning()
        refresh()
    }

    func applicationWillTerminate(_ notification: Notification) {
        daemonProcess?.terminate()
    }

    private func ensureBundledDaemonRunning(forceRestart: Bool = false) {
        if forceRestart {
            daemonProcess?.terminate()
            daemonProcess = nil
        }
        client.checkHealth { [weak self] online in
            guard let self else { return }
            if online && !forceRestart { return }
            let bundled = Bundle.main.bundleURL
                .appendingPathComponent("Contents/MacOS/bnos-print-daemon")
            guard FileManager.default.isExecutableFile(atPath: bundled.path) else { return }
            let proc = Process()
            proc.executableURL = bundled
            var env = ProcessInfo.processInfo.environment
            if self.allowLan {
                env["ALLOW_LAN"] = "1"
            }
            proc.environment = env
            try? proc.run()
            self.daemonProcess = proc
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                self.refresh()
            }
        }
    }

    private func rebuildMenu(online: Bool = false) {
        let menu = NSMenu()
        let healthItem = NSMenuItem(
            title: online ? "Daemon: Online (127.0.0.1:3100)" : "Daemon: Offline",
            action: nil,
            keyEquivalent: ""
        )
        healthItem.isEnabled = false
        menu.addItem(healthItem)
        menu.addItem(.separator())

        for printer in printers {
            let suffix = printer.isDefault ? " ★" : ""
            let item = NSMenuItem(
                title: "Test: \(printer.name) (\(printer.ip))\(suffix)",
                action: #selector(menuTestPrint(_:)),
                keyEquivalent: ""
            )
            item.representedObject = printer.id
            menu.addItem(item)
        }

        if !printers.isEmpty { menu.addItem(.separator()) }
        let lanItem = NSMenuItem(
            title: allowLan ? "✓ Allow LAN Access (0.0.0.0:3100)" : "Allow LAN Access (0.0.0.0:3100)",
            action: #selector(toggleAllowLan),
            keyEquivalent: ""
        )
        menu.addItem(lanItem)
        menu.addItem(NSMenuItem(title: "Printer Settings...", action: #selector(openSettingsWindow), keyEquivalent: ","))
        menu.addItem(NSMenuItem(title: "Open Web UI", action: #selector(openWebUI), keyEquivalent: ""))
        menu.addItem(.separator())
        menu.addItem(NSMenuItem(title: "Quit BNOS Print", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))
        statusItem.menu = menu
    }

    @objc private func menuTestPrint(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String else { return }
        client.testPrint(printerId: id) { [weak self] msg in
            self?.statusLabel.stringValue = msg
        }
    }

    @objc private func openWebUI() {
        NSWorkspace.shared.open(client.baseURL)
    }

    @objc private func toggleAllowLan() {
        allowLan = !allowLan
        ensureBundledDaemonRunning(forceRestart: true)
    }

    @objc private func openSettingsWindow() {
        if window == nil {
            buildSettingsWindow()
        }
        window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        refresh()
    }

    private func buildSettingsWindow() {
        let win = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 540, height: 420),
            styleMask: [.titled, .closable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        win.title = "BNOS & BDGO OS Print Settings"
        win.center()
        win.isReleasedWhenClosed = false

        let content = NSView(frame: win.contentView!.bounds)
        content.autoresizingMask = [.width, .height]

        statusLabel.frame = NSRect(x: 20, y: 382, width: 500, height: 20)
        content.addSubview(statusLabel)

        nameField.placeholderString = "Printer Name (e.g. Cashier)"
        nameField.frame = NSRect(x: 20, y: 344, width: 170, height: 24)
        ipField.placeholderString = "192.168.2.100"
        ipField.frame = NSRect(x: 198, y: 344, width: 130, height: 24)
        portField.frame = NSRect(x: 336, y: 344, width: 56, height: 24)
        widthPopup.addItems(withTitles: ["80mm", "58mm"])
        widthPopup.frame = NSRect(x: 398, y: 342, width: 70, height: 26)
        let addBtn = NSButton(title: "Add", target: self, action: #selector(onAddPrinter))
        addBtn.frame = NSRect(x: 472, y: 341, width: 52, height: 28)

        content.addSubview(nameField)
        content.addSubview(ipField)
        content.addSubview(portField)
        content.addSubview(widthPopup)
        content.addSubview(addBtn)

        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("printer"))
        col.title = "Configured IP Printers"
        col.width = 480
        tableView.addTableColumn(col)
        tableView.dataSource = self
        tableView.delegate = self

        let scroll = NSScrollView(frame: NSRect(x: 20, y: 64, width: 500, height: 264))
        scroll.documentView = tableView
        scroll.hasVerticalScroller = true
        content.addSubview(scroll)

        let testBtn = NSButton(title: "Test Print", target: self, action: #selector(onTestSelected))
        testBtn.frame = NSRect(x: 20, y: 20, width: 92, height: 30)
        let copyBtn = NSButton(title: "Copy Webhook", target: self, action: #selector(onCopyWebhookSelected))
        copyBtn.frame = NSRect(x: 118, y: 20, width: 116, height: 30)
        let defBtn = NSButton(title: "Set Default", target: self, action: #selector(onSetDefaultSelected))
        defBtn.frame = NSRect(x: 240, y: 20, width: 96, height: 30)
        let delBtn = NSButton(title: "Delete", target: self, action: #selector(onDeleteSelected))
        delBtn.frame = NSRect(x: 342, y: 20, width: 76, height: 30)
        let refreshBtn = NSButton(title: "Refresh", target: self, action: #selector(refresh))
        refreshBtn.frame = NSRect(x: 440, y: 20, width: 80, height: 30)

        content.addSubview(testBtn)
        content.addSubview(copyBtn)
        content.addSubview(defBtn)
        content.addSubview(delBtn)
        content.addSubview(refreshBtn)

        win.contentView = content
        window = win
    }

    @objc private func refresh() {
        client.checkHealth { [weak self] online in
            guard let self else { return }
            self.statusLabel.stringValue = online
                ? "Daemon online at 127.0.0.1:3100"
                : "Daemon offline — start print-daemon first"
            if online {
                self.client.listPrinters { list in
                    self.printers = list
                    self.tableView.reloadData()
                    self.rebuildMenu(online: true)
                }
            } else {
                self.rebuildMenu(online: false)
            }
        }
    }

    @objc private func onAddPrinter() {
        let ip = ipField.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !ip.isEmpty else {
            statusLabel.stringValue = "Please enter an IP address"
            return
        }
        let name = nameField.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)
        let port = Int(portField.stringValue) ?? 9100
        let width = widthPopup.titleOfSelectedItem == "58mm" ? 58 : 80
        client.addPrinter(name: name.isEmpty ? "Xprinter LAN" : name, ip: ip, port: port, paperWidthMm: width) { [weak self] ok in
            guard let self else { return }
            self.statusLabel.stringValue = ok ? "Added printer \(ip):\(port)" : "Failed to add printer"
            self.nameField.stringValue = ""
            self.ipField.stringValue = ""
            self.refresh()
        }
    }

    @objc private func onTestSelected() {
        let row = tableView.selectedRow
        guard row >= 0, row < printers.count else { return }
        statusLabel.stringValue = "Sending test print..."
        client.testPrint(printerId: printers[row].id) { [weak self] msg in
            self?.statusLabel.stringValue = msg
        }
    }

    @objc private func onCopyWebhookSelected() {
        let row = tableView.selectedRow
        let url: String
        if row >= 0, row < printers.count {
            let p = printers[row]
            url = "http://127.0.0.1:3100/print?ip=\(p.ip)&port=\(p.port)"
        } else {
            url = "http://127.0.0.1:3100/print"
        }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(url, forType: .string)
        statusLabel.stringValue = "Copied Webhook URL: \(url)"
    }

    @objc private func onSetDefaultSelected() {
        let row = tableView.selectedRow
        guard row >= 0, row < printers.count else { return }
        client.setDefault(id: printers[row].id) { [weak self] in
            self?.refresh()
        }
    }

    @objc private func onDeleteSelected() {
        let row = tableView.selectedRow
        guard row >= 0, row < printers.count else { return }
        client.removePrinter(id: printers[row].id) { [weak self] in
            self?.refresh()
        }
    }

    func numberOfRows(in tableView: NSTableView) -> Int {
        printers.count
    }

    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        let p = printers[row]
        let badge = p.isDefault ? " [Default]" : ""
        return NSTextField(labelWithString: "\(p.name)\(badge) — \(p.ip):\(p.port) (\(p.paperWidthMm)mm)")
    }
}

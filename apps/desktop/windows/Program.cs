using System.Diagnostics;
using System.Drawing;
using System.Windows.Forms;

namespace BNOSPrintTray;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        ApplicationConfiguration.Initialize();
        Application.Run(new TrayApplicationContext());
    }
}

internal sealed class TrayApplicationContext : ApplicationContext
{
    private readonly DaemonClient _client = new();
    private readonly NotifyIcon _trayIcon;
    private readonly ContextMenuStrip _menu = new();
    private SettingsForm? _settingsForm;
    private List<DesktopPrinter> _printers = new();

    public TrayApplicationContext()
    {
        _trayIcon = new NotifyIcon
        {
            Icon = Icon.ExtractAssociatedIcon(Application.ExecutablePath) ?? SystemIcons.Application,
            Text = "BNOS & BDGO OS Print Service",
            Visible = true,
            ContextMenuStrip = _menu
        };
        _trayIcon.DoubleClick += (_, _) => OpenSettings();
        _ = RefreshMenuAsync();
    }

    private async Task RefreshMenuAsync()
    {
        var online = await _client.CheckHealthAsync();
        _printers = online ? await _client.ListPrintersAsync() : new List<DesktopPrinter>();

        _menu.Items.Clear();
        _menu.Items.Add(new ToolStripMenuItem(online ? "Daemon: Online (127.0.0.1:3100)" : "Daemon: Offline") { Enabled = false });
        _menu.Items.Add(new ToolStripSeparator());

        foreach (var p in _printers)
        {
            var star = p.IsDefault ? " *" : "";
            var item = new ToolStripMenuItem($"Test: {p.Name} ({p.Ip}){star}");
            item.Click += async (_, _) =>
            {
                var msg = await _client.TestPrintAsync(p.Id);
                _trayIcon.ShowBalloonTip(2500, "BNOS Print", msg, ToolTipIcon.Info);
            };
            _menu.Items.Add(item);
        }

        if (_printers.Count > 0) _menu.Items.Add(new ToolStripSeparator());
        _menu.Items.Add("Printer Settings...", null, (_, _) => OpenSettings());
        _menu.Items.Add("Open Web UI", null, (_, _) => Process.Start(new ProcessStartInfo(DaemonClient.BaseUrl) { UseShellExecute = true }));
        _menu.Items.Add(new ToolStripSeparator());
        _menu.Items.Add("Quit BNOS Print", null, (_, _) =>
        {
            _trayIcon.Visible = false;
            ExitThread();
        });
    }

    private void OpenSettings()
    {
        if (_settingsForm is null || _settingsForm.IsDisposed)
        {
            _settingsForm = new SettingsForm(_client, () => _ = RefreshMenuAsync());
        }
        _settingsForm.Show();
        _settingsForm.BringToFront();
        _ = _settingsForm.ReloadAsync();
    }
}

internal sealed class SettingsForm : Form
{
    private readonly DaemonClient _client;
    private readonly Action _onUpdated;
    private readonly Label _statusLabel = new() { Left = 16, Top = 12, Width = 500, Text = "Checking daemon..." };
    private readonly TextBox _nameBox = new() { Left = 16, Top = 42, Width = 160, PlaceholderText = "Printer Name" };
    private readonly TextBox _ipBox = new() { Left = 184, Top = 42, Width = 130, PlaceholderText = "192.168.2.100" };
    private readonly TextBox _portBox = new() { Left = 322, Top = 42, Width = 56, Text = "9100" };
    private readonly ComboBox _widthBox = new() { Left = 386, Top = 42, Width = 68, DropDownStyle = ComboBoxStyle.DropDownList };
    private readonly ListBox _listBox = new() { Left = 16, Top = 78, Width = 500, Height = 230 };
    private List<DesktopPrinter> _printers = new();

    public SettingsForm(DaemonClient client, Action onUpdated)
    {
        _client = client;
        _onUpdated = onUpdated;

        Text = "BNOS & BDGO OS Print Settings";
        Width = 548;
        Height = 398;
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;

        _widthBox.Items.AddRange(new object[] { "80mm", "58mm" });
        _widthBox.SelectedIndex = 0;

        var addBtn = new Button { Left = 462, Top = 40, Width = 54, Height = 26, Text = "Add" };
        addBtn.Click += async (_, _) => await AddPrinterAsync();

        var testBtn = new Button { Left = 16, Top = 318, Width = 88, Height = 28, Text = "Test Print" };
        testBtn.Click += async (_, _) => await TestSelectedAsync();

        var copyBtn = new Button { Left = 110, Top = 318, Width = 108, Height = 28, Text = "Copy Webhook" };
        copyBtn.Click += (_, _) => CopyWebhookSelected();

        var defBtn = new Button { Left = 224, Top = 318, Width = 90, Height = 28, Text = "Set Default" };
        defBtn.Click += async (_, _) => await SetDefaultSelectedAsync();

        var delBtn = new Button { Left = 320, Top = 318, Width = 70, Height = 28, Text = "Delete" };
        delBtn.Click += async (_, _) => await DeleteSelectedAsync();

        var refreshBtn = new Button { Left = 436, Top = 318, Width = 80, Height = 28, Text = "Refresh" };
        refreshBtn.Click += async (_, _) => await ReloadAsync();

        Controls.AddRange(new Control[]
        {
            _statusLabel, _nameBox, _ipBox, _portBox, _widthBox, addBtn,
            _listBox, testBtn, copyBtn, defBtn, delBtn, refreshBtn
        });
    }

    public async Task ReloadAsync()
    {
        var online = await _client.CheckHealthAsync();
        _statusLabel.Text = online ? "Daemon online at 127.0.0.1:3100" : "Daemon offline — start print-daemon.exe first";
        _printers = online ? await _client.ListPrintersAsync() : new List<DesktopPrinter>();
        _listBox.Items.Clear();
        foreach (var p in _printers)
        {
            var badge = p.IsDefault ? " [Default]" : "";
            _listBox.Items.Add($"{p.Name}{badge} — {p.Ip}:{p.Port} ({p.PaperWidthMm}mm)");
        }
        _onUpdated();
    }

    private async Task AddPrinterAsync()
    {
        var ip = _ipBox.Text.Trim();
        if (string.IsNullOrEmpty(ip)) { _statusLabel.Text = "Please enter an IP address"; return; }
        var name = string.IsNullOrWhiteSpace(_nameBox.Text) ? "Xprinter LAN" : _nameBox.Text.Trim();
        var port = int.TryParse(_portBox.Text, out var parsed) ? parsed : 9100;
        var width = _widthBox.SelectedItem?.ToString() == "58mm" ? 58 : 80;
        var ok = await _client.AddPrinterAsync(name, ip, port, width);
        _statusLabel.Text = ok ? $"Added printer {ip}:{port}" : "Failed to add printer";
        _nameBox.Text = "";
        _ipBox.Text = "";
        await ReloadAsync();
    }

    private async Task TestSelectedAsync()
    {
        if (_listBox.SelectedIndex < 0 || _listBox.SelectedIndex >= _printers.Count) return;
        _statusLabel.Text = "Sending test print...";
        _statusLabel.Text = await _client.TestPrintAsync(_printers[_listBox.SelectedIndex].Id);
    }

    private void CopyWebhookSelected()
    {
        string url;
        if (_listBox.SelectedIndex >= 0 && _listBox.SelectedIndex < _printers.Count)
        {
            var p = _printers[_listBox.SelectedIndex];
            url = $"{DaemonClient.BaseUrl}/print?ip={Uri.EscapeDataString(p.Ip)}&port={p.Port}";
        }
        else
        {
            url = $"{DaemonClient.BaseUrl}/print";
        }
        Clipboard.SetText(url);
        _statusLabel.Text = $"Copied Webhook URL: {url}";
    }

    private async Task SetDefaultSelectedAsync()
    {
        if (_listBox.SelectedIndex < 0 || _listBox.SelectedIndex >= _printers.Count) return;
        await _client.SetDefaultAsync(_printers[_listBox.SelectedIndex].Id);
        await ReloadAsync();
    }

    private async Task DeleteSelectedAsync()
    {
        if (_listBox.SelectedIndex < 0 || _listBox.SelectedIndex >= _printers.Count) return;
        await _client.RemovePrinterAsync(_printers[_listBox.SelectedIndex].Id);
        await ReloadAsync();
    }
}

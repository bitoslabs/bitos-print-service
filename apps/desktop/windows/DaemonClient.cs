using System.Net.Http.Json;
using System.Text.Json.Serialization;

namespace BNOSPrintTray;

public sealed record DesktopPrinter(
    [property: JsonPropertyName("id")] string Id,
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("ip")] string Ip,
    [property: JsonPropertyName("port")] int Port,
    [property: JsonPropertyName("paperWidthMm")] int PaperWidthMm,
    [property: JsonPropertyName("isDefault")] bool IsDefault
);

file sealed record PrintersResponse(
    [property: JsonPropertyName("ok")] bool Ok,
    [property: JsonPropertyName("printers")] List<DesktopPrinter> Printers
);

file sealed record PrintResultResponse(
    [property: JsonPropertyName("ok")] bool Ok,
    [property: JsonPropertyName("bytes")] int? Bytes,
    [property: JsonPropertyName("error")] string? Error
);

public sealed class DaemonClient
{
    public const string BaseUrl = "http://127.0.0.1:3100";
    private readonly HttpClient _http = new() { Timeout = TimeSpan.FromSeconds(5) };

    public async Task<bool> CheckHealthAsync()
    {
        try
        {
            using var resp = await _http.GetAsync($"{BaseUrl}/health");
            return resp.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    public async Task<List<DesktopPrinter>> ListPrintersAsync()
    {
        try
        {
            var resp = await _http.GetFromJsonAsync<PrintersResponse>($"{BaseUrl}/printers");
            return resp?.Printers ?? new List<DesktopPrinter>();
        }
        catch
        {
            return new List<DesktopPrinter>();
        }
    }

    public async Task<bool> AddPrinterAsync(string name, string ip, int port, int paperWidthMm)
    {
        var resp = await _http.PostAsJsonAsync($"{BaseUrl}/printers", new { name, ip, port, paperWidthMm });
        return resp.IsSuccessStatusCode;
    }

    public async Task SetDefaultAsync(string id)
    {
        await _http.PostAsync($"{BaseUrl}/printers/{Uri.EscapeDataString(id)}/default", null);
    }

    public async Task RemovePrinterAsync(string id)
    {
        await _http.DeleteAsync($"{BaseUrl}/printers/{Uri.EscapeDataString(id)}");
    }

    public async Task<string> TestPrintAsync(string printerId)
    {
        try
        {
            var resp = await _http.PostAsJsonAsync($"{BaseUrl}/print", new { type = "test", printerId });
            var parsed = await resp.Content.ReadFromJsonAsync<PrintResultResponse>();
            if (parsed is null) return "Daemon returned empty response";
            return parsed.Ok
                ? $"Test print OK ({parsed.Bytes ?? 0} bytes)"
                : $"Failed: {parsed.Error ?? "Unknown error"}";
        }
        catch (Exception ex)
        {
            return $"Error: {ex.Message}";
        }
    }
}

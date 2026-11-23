param(
  [string]$BinaryPath = ".\target\release\print-daemon.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $BinaryPath)) {
  throw "Binary not found at $BinaryPath. Run 'cargo build --release -p print-daemon' first."
}

$InstallDir = Join-Path $env:LOCALAPPDATA "BNOSPrint"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$TargetExe = Join-Path $InstallDir "bnos-print-daemon.exe"
Copy-Item -Force $BinaryPath $TargetExe

# Register per-user auto-start on Windows login (no admin required)
$RunKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Set-ItemProperty -Path $RunKey -Name "BNOSPrintDaemon" -Value "`"$TargetExe`""

# Start immediately in background
Start-Process -FilePath $TargetExe -WindowStyle Hidden

Write-Host "Installed and started: $TargetExe"
Write-Host "Settings UI: http://127.0.0.1:3100/"

$ErrorActionPreference = "SilentlyContinue"

Get-Process -Name "bnos-print-daemon" | Stop-Process -Force

$RunKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Remove-ItemProperty -Path $RunKey -Name "BNOSPrintDaemon"

$InstallDir = Join-Path $env:LOCALAPPDATA "BNOSPrint"
if (Test-Path $InstallDir) {
  Remove-Item -Recurse -Force $InstallDir
}

Write-Host "Uninstalled BNOS Print Daemon from Windows Startup."

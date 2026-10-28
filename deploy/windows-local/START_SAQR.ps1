# Alternative launcher if .bat fails (PowerShell)
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

if (-not (Test-Path ".env")) {
    if (Test-Path "env.template") { Copy-Item env.template .env }
    Write-Host "Edit .env then run again."
    notepad .env
    exit 1
}
if (-not (Test-Path "bin\saqr.exe")) {
    Write-Host "Missing bin\saqr.exe"
    exit 1
}
New-Item -ItemType Directory -Force -Path logs | Out-Null
$env:RUST_LOG = "info"
Write-Host "Dashboard: http://127.0.0.1:8787"
& ".\bin\saqr.exe"

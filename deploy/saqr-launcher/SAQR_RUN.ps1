# SAQR Engine v6.0 — Launcher (A to Z)
# Run: double-click SAQR_RUN.bat or: powershell -ExecutionPolicy Bypass -File SAQR_RUN.ps1
$ErrorActionPreference = "Continue"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root

function Write-Step($msg) { Write-Host "[SAQR] $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "[OK]   $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "[FAIL] $msg" -ForegroundColor Red }

Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "   SAQR v6.0 - Home Launcher" -ForegroundColor Yellow
Write-Host "   Root: $Root" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host ""

$ExeCandidates = @(
    (Join-Path $Root "bin\saqr.exe"),
    (Join-Path $Root "saqr.exe"),
    (Join-Path $Root "target\release\saqr.exe")
)
$Exe = $ExeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

$WebUi = Join-Path $Root "web\control_terminal\index.html"
$EnvFile = Join-Path $Root ".env"
$Template = Join-Path $Root "env.template"
if (-not (Test-Path $Template)) { $Template = Join-Path $Root ".env.example" }

foreach ($dir in @("logs", "data")) {
    $p = Join-Path $Root $dir
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p | Out-Null }
}

if (-not $Exe) {
    Write-Fail "saqr.exe not found. Expected bin\saqr.exe in release ZIP."
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Ok "Executable: $Exe"

if (-not (Test-Path $WebUi)) {
    Write-Fail "Missing web\control_terminal\index.html"
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Ok "Web UI: web\control_terminal"

if (-not (Test-Path $EnvFile)) {
    if (Test-Path $Template) {
        Copy-Item $Template $EnvFile
        Write-Step "Created .env from template. Edit keys then run again."
        notepad $EnvFile
        Read-Host "Press Enter after saving .env"
    } else {
        Write-Fail ".env missing and no env.template"
        Read-Host "Press Enter to exit"
        exit 1
    }
}
Write-Ok ".env present"

$env:RUST_LOG = "info"
Set-Location $Root

Write-Step "Version check..."
& $Exe --version
if ($LASTEXITCODE -ne 0) {
    Write-Fail "saqr.exe failed --version (install VC++ Redist x64?)"
    Read-Host "Press Enter to exit"
    exit 2
}

Write-Step "Config validation (--check)..."
& $Exe --check
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Fix .env errors above (Telegram both empty OR both set; no half-config)."
    Write-Host "Tip: SAQR_REQUIRE_INTEGRATED_DEMO=false for first run without Telegram."
    Read-Host "Press Enter to exit"
    exit 3
}
Write-Ok "Configuration valid"

$port = 8787
if (Test-Path $EnvFile) {
    Get-Content $EnvFile | ForEach-Object {
        if ($_ -match '^\s*SAQR_WEB_BIND_PORT\s*=\s*(\d+)') { $port = [int]$Matches[1] }
    }
}
$dashUrl = "http://127.0.0.1:$port"

Write-Host ""
Write-Host "Dashboard: $dashUrl" -ForegroundColor Green
Write-Host "Stop bot: Ctrl+C (do not press Enter while running)" -ForegroundColor Yellow
Write-Host ""

Start-Job -ScriptBlock { param($u) Start-Sleep -Seconds 4; Start-Process $u } -ArgumentList $dashUrl | Out-Null

$logFile = Join-Path $Root "logs\saqr-run.log"
Write-Step "Starting engine (log: logs\saqr-run.log)..."

try {
    & $Exe 2>&1 | Tee-Object -FilePath $logFile
} finally {
    Write-Host ""
    Write-Host "Engine stopped. Exit: $LASTEXITCODE" -ForegroundColor Cyan
    Read-Host "Press Enter to close"
}

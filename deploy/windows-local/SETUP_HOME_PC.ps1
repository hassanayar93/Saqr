# SAQR — إعداد حاسوب Windows (استنساخ + بناء + .env)
param(
    [string]$InstallDir = (Join-Path $env:USERPROFILE "Saqr")
)

$ErrorActionPreference = "Stop"
$RepoUrl = "https://github.com/hassanayar93/Saqr.git"

Write-Host "=== SAQR Home PC Setup ===" -ForegroundColor Cyan
Write-Host "Target: $InstallDir"

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Git via winget..."
    winget install -e --id Git.Git --accept-package-agreements --accept-source-agreements
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rust via winget..."
    winget install -e --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

if (-not (Test-Path (Join-Path $InstallDir ".git"))) {
    git clone $RepoUrl $InstallDir
} else {
    Set-Location $InstallDir
    git pull
}

Set-Location $InstallDir

if (-not (Test-Path ".env")) {
    if (Test-Path ".env.example") {
        Copy-Item ".env.example" ".env"
        Write-Host "Created .env from .env.example — edit before live/testnet." -ForegroundColor Yellow
    }
}

Write-Host "Building release binary..."
cargo build --release -p saqr-cli

Copy-Item -Force "deploy\windows-local\START_SAQR.bat" (Join-Path $InstallDir "START_SAQR.bat")
Copy-Item -Force "deploy\windows-local\OPEN_DASHBOARD.bat" (Join-Path $InstallDir "OPEN_DASHBOARD.bat")

Write-Host ""
Write-Host "Done. Next:" -ForegroundColor Green
Write-Host "  1. Edit $InstallDir\.env"
Write-Host "  2. Double-click START_SAQR.bat"
Write-Host "  3. Browser: http://127.0.0.1:8787"

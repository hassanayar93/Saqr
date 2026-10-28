# Open SAQR Control Terminal in default browser (bot must be running)
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$port = 8787
$envFile = Join-Path $Root ".env"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        if ($_ -match '^\s*SAQR_WEB_BIND_PORT\s*=\s*(\d+)') { $port = [int]$Matches[1] }
    }
}
$url = "http://127.0.0.1:$port"
Write-Host "Opening dashboard: $url" -ForegroundColor Cyan
Write-Host "If page fails: run SAQR_RUN.bat first." -ForegroundColor Yellow
Start-Process $url

# Fix .env beside SAQR_RUN.bat (ZWSP, + before Telegram token, missing keys)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$EnvFile = Join-Path $Root ".env"
if (-not (Test-Path $EnvFile)) {
    Write-Host "[FAIL] .env not found in $Root" -ForegroundColor Red
    Read-Host "Press Enter"
    exit 1
}

$invisible = [char[]]@(0x200B, 0x200C, 0x200D, 0xFEFF, 0x2060, 0x00AD)
$defaults = @{
    "SAQR_REQUIRE_INTEGRATED_DEMO" = "false"
    "SAQR_ENABLE_TRADING"          = "false"
    "SAQR_ENABLE_LIVE_ACK"         = "false"
    "SAQR_WAL_PATH"                = "data/saqr.wal.jsonl"
    "SAQR_WS_RECONNECT_SECS"       = "3"
    "SAQR_INTEGRATION_INTERVAL_SECS" = "300"
}

function Clean-Val($key, $val) {
    $v = $val.Trim().Trim('"').Trim("'")
    foreach ($c in $invisible) { $v = $v.Replace([string]$c, "") }
    if ($key -eq "TELEGRAM_BOT_TOKEN" -and $v.StartsWith("+")) { $v = $v.TrimStart("+") }
    return $v
}

$seen = @{}
$out = New-Object System.Collections.Generic.List[string]
Get-Content -LiteralPath $EnvFile -Encoding UTF8 | ForEach-Object {
    $line = $_
    if ($line -match '^\s*#' -or [string]::IsNullOrWhiteSpace($line)) {
        [void]$out.Add($line)
        return
    }
    if ($line -notmatch '=') {
        [void]$out.Add($line)
        return
    }
    $key, $rest = $line.Split('=', 2)
    $key = $key.Trim()
    $val = Clean-Val $key $rest
    $seen[$key.ToUpperInvariant()] = $true
    [void]$out.Add("$key=$val")
}

$insert = 0
for ($i = 0; $i -lt $out.Count; $i++) {
    if ($out[$i] -match '^SAQR_MODE=') { $insert = $i + 1; break }
}
foreach ($k in ($defaults.Keys | Sort-Object)) {
    if (-not $seen.ContainsKey($k.ToUpperInvariant())) {
        $out.Insert($insert, "$k=$($defaults[$k])")
        $insert++
    }
}

$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllLines($EnvFile, $out.ToArray(), $utf8NoBom)
Write-Host "[OK] Fixed .env in $Root" -ForegroundColor Green
Write-Host "Run SAQR_RUN.bat again." -ForegroundColor Cyan
Read-Host "Press Enter"

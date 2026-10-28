$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root
Write-Host "SAQR Diagnose - $Root"
@("bin\saqr.exe", "web\control_terminal\index.html", ".env", "env.template") | ForEach-Object {
    if (Test-Path $_) { Write-Host "[OK] $_" -ForegroundColor Green }
    else { Write-Host "[MISSING] $_" -ForegroundColor Red }
}
if (Test-Path "bin\saqr.exe") {
    & ".\bin\saqr.exe" --version
    if (Test-Path ".env") { & ".\bin\saqr.exe" --check }
}
Read-Host "Press Enter"

@echo off
chcp 65001 >nul
cd /d "%~dp0"
if exist "env.template" (
  copy /Y "env.template" ".env"
  echo OK: .env created. Edit keys then run START_SAQR.bat
  notepad ".env"
) else if exist ".env.example" (
  copy /Y ".env.example" ".env"
  notepad ".env"
) else (
  echo env.template not found
)
pause

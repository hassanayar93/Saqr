@echo off
cd /d "%~dp0"
if exist "env.template" (
  copy /Y env.template .env
  echo Created .env - edit Telegram and save.
  notepad .env
) else (
  echo env.template not found.
)
pause

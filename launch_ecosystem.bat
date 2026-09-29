@echo off
title Proteus Ecosystem Launcher
cd /d "%~dp0project"

echo ========================================================
echo   PROTEUS ECOSYSTEM - LAUNCHER
echo ========================================================
echo.
echo Cleaning up any stale instances...
taskkill /F /IM proteus-design-studio.exe /IM proteus.exe /IM proteus-client.exe /IM proteus-web.exe /IM proteus-mobile.exe >nul 2>&1

echo [1/3] Starting Proteus Web Hub (Port 8080)...
start "" /D "%~dp0project" "%~dp0project\target\debug\proteus-web.exe"

ping 127.0.0.1 -n 2 >nul

echo [2/3] Starting Proteus Client (Counter / BOS)...
start "" /D "%~dp0project" "%~dp0project\target\debug\proteus-client.exe"

ping 127.0.0.1 -n 2 >nul

echo [3/3] Starting Proteus Design Studio...
start "" /D "%~dp0project" "%~dp0project\target\debug\proteus-design-studio.exe"

ping 127.0.0.1 -n 2 >nul
start http://localhost:8080/hub

echo.
echo ========================================================
echo   All 3 applications launched successfully!
echo   - Web Hub: http://localhost:8080/hub
echo   - Proteus Client (Desktop)
echo   - Proteus Design Studio (Desktop)
echo.
echo   To stop all applications, run stop_ecosystem.bat
echo ========================================================


@echo off
title Proteus Ecosystem - Stop
cd /d "%~dp0project"

echo ========================================================
echo   CLOSING ALL PROTEUS APPLICATIONS
echo ========================================================
echo.
taskkill /F /IM proteus-design-studio.exe /IM proteus.exe /IM proteus-client.exe /IM proteus-web.exe /IM proteus-mobile.exe >nul 2>&1
echo [OK] Proteus Design Studio stopped.
echo [OK] Proteus Client stopped.
echo [OK] Proteus Web Hub stopped.
echo [OK] Proteus Mobile stopped.
echo.
echo All background and desktop processes have been completely closed.
ping 127.0.0.1 -n 2 >nul

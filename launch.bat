@echo off
setlocal
cd /d "%~dp0"
chcp 65001 >nul
title Proteus Business OS — Interactive Launcher

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\launcher.ps1"
if errorlevel 1 (
    echo.
    echo [Launcher exited with an error]
    pause
)

endlocal

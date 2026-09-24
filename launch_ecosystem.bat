@echo off
title Proteus Ecosystem Launcher
cd /d "%~dp0project"

echo [1/3] Starting Proteus Web Hub (Port 8080)...
start "Proteus Web" /D "%~dp0project" "%~dp0project\target\debug\proteus-web.exe"

timeout /t 1 /nobreak >nul

echo [2/3] Starting Proteus Client (Counter / BOS)...
start "Proteus Client" /D "%~dp0project" "%~dp0project\target\debug\proteus-client.exe"

timeout /t 1 /nobreak >nul

echo [3/3] Starting Proteus Designer Studio...
start "Proteus Designer" /D "%~dp0project" "%~dp0project\target\debug\proteus.exe"

timeout /t 1 /nobreak >nul
start http://localhost:8080

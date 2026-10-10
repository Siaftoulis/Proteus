@echo off
setlocal
cd /d %~dp0
chcp 65001 >nul

if "%~1"=="1" goto fast_studio
if /i "%~1"=="studio" goto fast_studio
if "%~1"=="2" goto fast_client
if /i "%~1"=="client" goto fast_client
if "%~1"=="8" goto fast_data
if /i "%~1"=="data" goto fast_data

title Proteus Business OS ? Interactive Launcher
powershell.exe -NoProfile -ExecutionPolicy Bypass -File %~dp0scripts\launcher.ps1
if errorlevel 1 (
    echo.
    echo [Launcher exited with an error]
    pause
)
endlocal
goto :eof

:fast_studio
endlocal
taskkill /F /IM proteus-design-studio.exe >nul 2>&1
cd /d %~dp0project
if not exist "%~dp0project\target\debug\proteus-design-studio.exe" cargo build -p proteus-design-studio
start "" "%~dp0project\target\debug\proteus-design-studio.exe"
goto :eof

:fast_client
endlocal
taskkill /F /IM proteus-client.exe >nul 2>&1
cd /d %~dp0project
if not exist "%~dp0project\target\debug\proteus-client.exe" cargo build -p proteus-client
start "" "%~dp0project\target\debug\proteus-client.exe"
goto :eof

:fast_data
endlocal
python "%~dp0scripts\generate_synthetic_db.py"
goto :eof

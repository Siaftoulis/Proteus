@echo off
setlocal
cd /d "%~dp0\..\project"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0verify_pipeline.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Pipeline verification failed with exit code %ERRORLEVEL%!
    pause
    exit /b %ERRORLEVEL%
)

echo [SUCCESS] Pipeline verification passed with zero errors!
pause
endlocal

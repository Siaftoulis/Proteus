@echo off
title Proteus Mobile Simulator
cd /d "%~dp0project"

echo ========================================================
echo   PROTEUS MOBILE - HANDHELD TOUCH CLIENT SIMULATOR
echo ========================================================
echo.
cargo run -p proteus-mobile

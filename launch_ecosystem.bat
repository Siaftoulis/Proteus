@echo off
rem Proteus Ecosystem Unified Launcher
rem Launches Proteus Web Hub, Proteus Client, and Proteus Designer without CMD console windows.
start "" "%~dp0project\target\debug\proteus-web.exe"
start "" "%~dp0project\target\debug\proteus-client.exe"
start "" "%~dp0project\target\debug\proteus.exe"

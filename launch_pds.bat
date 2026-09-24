@echo off
title Proteus Designer Studio
cd /d "%~dp0project"
start "Proteus Designer" /D "%~dp0project" "%~dp0project\target\debug\proteus.exe"

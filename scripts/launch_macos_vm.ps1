<#
.SYNOPSIS
    Launches a near-native macOS virtual machine environment on Windows using Docker-OSX & QEMU/KVM.
    Enables building, testing, and viewing Proteus on macOS and running the Xcode iOS/iPhone Simulator.

.DESCRIPTION
    Docker-OSX runs macOS (Sonoma / Ventura) inside a hardware-accelerated container via WSL2.
    It exposes:
      - Web noVNC GUI: http://localhost:6080
      - Native VNC:    vnc://localhost:5900
      - SSH:           ssh -p 50922 arch@localhost
#>

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   Proteus macOS & iPhone Virtualization Launcher       " -ForegroundColor Yellow
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Check Docker Desktop availability
$dockerInstalled = Get-Command docker -ErrorAction SilentlyContinue
if (-not $dockerInstalled) {
    Write-Host "[ERROR] Docker Desktop is not installed or not in PATH." -ForegroundColor Red
    Write-Host "Please install Docker Desktop for Windows with WSL2 backend enabled:" -ForegroundColor White
    Write-Host "https://docs.docker.com/desktop/setup/install/windows-install/" -ForegroundColor Gray
    Exit 1
}

# 2. Check Docker daemon status
docker info >$null 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Docker service is not running. Please start Docker Desktop first." -ForegroundColor Red
    Exit 1
}

Write-Host "[OK] Docker is running." -ForegroundColor Green
$composeFile = Join-Path $PSScriptRoot "..\project\deploy\docker-compose.macos.yml"

Write-Host "[INFO] Starting macOS Virtual Container via Docker-OSX..." -ForegroundColor Cyan
Write-Host "[INFO] Using configuration: $composeFile" -ForegroundColor Gray

docker compose -f $composeFile up -d

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n==========================================================" -ForegroundColor Green
    Write-Host " macOS Virtual Machine is now starting up! " -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host "1. Direct Browser GUI:  http://localhost:6080" -ForegroundColor Yellow
    Write-Host "2. Native VNC Viewer:   localhost:5900" -ForegroundColor Yellow
    Write-Host "3. SSH Console:         ssh -p 50922 arch@localhost (pass: alpine)" -ForegroundColor Yellow
    Write-Host "`nYour workspace is mapped to: /home/arch/crm-builder" -ForegroundColor Cyan
    Write-Host "Inside macOS, you can run Xcode, the iOS Simulator, and native cargo builds!" -ForegroundColor Cyan
} else {
    Write-Host "[ERROR] Failed to start macOS container." -ForegroundColor Red
}

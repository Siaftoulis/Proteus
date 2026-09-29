<#
.SYNOPSIS
    Cross-compiles native macOS binaries (Intel x86_64 & Apple Silicon ARM64)
    directly on Windows without requiring a physical Mac or VM.

.DESCRIPTION
    Uses cargo-zigbuild and the macOS SDK to produce authentic Mach-O binaries for:
      - x86_64-apple-darwin (Intel Macs)
      - aarch64-apple-darwin (Apple Silicon M1/M2/M3/M4 Macs)
#>

param(
    [string]$Target = "all", # "arm64", "x86_64", or "all"
    [string]$Package = "proteus-client" # or "proteus-design-studio", "proteus-mobile"
)

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   Proteus macOS Native Cross-Compiler (Windows Host)    " -ForegroundColor Yellow
Write-Host "==========================================================" -ForegroundColor Cyan

# Check if cargo-zigbuild is installed
$zigbuild = Get-Command cargo-zigbuild -ErrorAction SilentlyContinue
if (-not $zigbuild) {
    Write-Host "[INFO] cargo-zigbuild not found. Installing via cargo..." -ForegroundColor Cyan
    cargo install cargo-zigbuild
}

$targets = @()
if ($Target -eq "all" -or $Target -eq "arm64") {
    $targets += "aarch64-apple-darwin"
}
if ($Target -eq "all" -or $Target -eq "x86_64") {
    $targets += "x86_64-apple-darwin"
}

Push-Location (Join-Path $PSScriptRoot "..\project")
try {
    foreach ($t in $targets) {
        Write-Host "`n[COMPILING] Building $Package for $t..." -ForegroundColor Green
        rustup target add $t
        cargo zigbuild --package $Package --target $t --release
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[SUCCESS] Artifact created at target/$t/release/$Package" -ForegroundColor Green
        } else {
            Write-Host "[FAILED] Build failed for $t" -ForegroundColor Red
        }
    }
} finally {
    Pop-Location
}

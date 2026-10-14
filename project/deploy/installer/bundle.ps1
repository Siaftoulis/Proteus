# ==============================================================================
# Proteus Sovereign BOS — Portable Distribution Assembler
# Creates a standalone, zero-installation portable bundle in dist/proteus-desktop/
# ==============================================================================

[CmdletBinding()]
param (
    [string]$ProjectRoot = "",
    [string]$OutputDir = "",
    [ValidateSet("release", "debug")]
    [string]$Profile = "release",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$scriptDir = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($scriptDir)) {
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
}
if (-not [System.IO.Path]::IsPathRooted($scriptDir)) {
    $scriptDir = (Resolve-Path $scriptDir).Path
}

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path (Join-Path $scriptDir "..\..")).Path
} else {
    $ProjectRoot = (Resolve-Path $ProjectRoot).Path
}

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $ProjectRoot "dist\proteus-desktop"
}

Write-Host ">>> [Proteus] Initializing Portable Bundle Assembler..." -ForegroundColor Cyan
Write-Host "    Project Root: $ProjectRoot"
Write-Host "    Target Bundle: $OutputDir"
Write-Host "    Build Profile: $Profile"

# 1. Build release binaries if not skipped
if (-not $SkipBuild) {
    Write-Host "`n>>> [Proteus] Compiling native binaries (profile: $Profile)..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    try {
        if ($Profile -eq "release") {
            cargo build --release -p proteus-client -p proteus-design-studio
        } else {
            cargo build -p proteus-client -p proteus-design-studio
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo compilation failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
}

# 2. Prepare directory structure
Write-Host "`n>>> [Proteus] Preparing directory layout..." -ForegroundColor Yellow
$binDir = Join-Path $OutputDir "bin"
$configDir = Join-Path $OutputDir "config"
$dataDir = Join-Path $OutputDir "data"
$assetsDir = Join-Path $OutputDir "assets"

foreach ($dir in @($binDir, $configDir, $dataDir, $assetsDir)) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}

# 3. Copy compiled binaries
$targetSub = if ($Profile -eq "release") { "release" } else { "debug" }
$targetBinDir = Join-Path $ProjectRoot "target\$targetSub"

$binaries = @("proteus-client.exe", "proteus-design-studio.exe")
foreach ($bin in $binaries) {
    $srcPath = Join-Path $targetBinDir $bin
    if (Test-Path $srcPath) {
        Copy-Item -Path $srcPath -Destination $binDir -Force
        Write-Host "    [OK] Copied binary: $bin" -ForegroundColor Green
    } else {
        Write-Warning "Binary not found at $srcPath (skip if running dry-run test)"
    }
}

# 4. Copy default configuration
$defaultConfig = Join-Path $PSScriptRoot "proteus.default.toml"
$targetConfig = Join-Path $configDir "proteus.toml"
if (Test-Path $defaultConfig) {
    Copy-Item -Path $defaultConfig -Destination $targetConfig -Force
    Write-Host "    [OK] Placed default config: config\proteus.toml" -ForegroundColor Green
}

# 5. Create 1-click launcher scripts for Average Joe
$posBat = Join-Path $OutputDir "Run-Proteus-POS.bat"
$posBatContent = @"
@echo off
title Proteus Sovereign POS - Frontline Cashier Station
cd /d "%~dp0"
set PROTEUS_PORTABLE=1
set PROTEUS_DATA_DIR=%~dp0data
start "" "%~dp0bin\proteus-client.exe"
"@
Set-Content -Path $posBat -Value $posBatContent -Encoding ASCII
Write-Host "    [OK] Created launcher: Run-Proteus-POS.bat" -ForegroundColor Green

$studioBat = Join-Path $OutputDir "Run-Proteus-Studio.bat"
$studioBatContent = @"
@echo off
title Proteus Design Studio - Management & SSG Storefront
cd /d "%~dp0"
set PROTEUS_PORTABLE=1
set PROTEUS_DATA_DIR=%~dp0data
start "" "%~dp0bin\proteus-design-studio.exe"
"@
Set-Content -Path $studioBat -Value $studioBatContent -Encoding ASCII
Write-Host "    [OK] Created launcher: Run-Proteus-Studio.bat" -ForegroundColor Green

# 6. Create Greek merchant user guide
$readme = Join-Path $OutputDir "README-PORTABLE.txt"
$readmeContent = @"
==============================================================================
               PROTEUS SOVEREIGN BUSINESS OPERATING SYSTEM
              ΦΟΡΗΤΗ ΕΚΔΟΣΗ (PORTABLE EDITION) — 100% ΑΥΤΟΝΟΜΟ
==============================================================================

Συγχαρητήρια! Έχετε στην κατοχή σας το αυτόνομο επιχειρησιακό λογισμικό Proteus,
σχεδιασμένο για 100% τοπική εκτέλεση χωρίς μηνιαίες συνδρομές και χωρίς κλείδωμα
σε παρόχους cloud (Zero SaaS lock-in).

ΟΔΗΓΙΕΣ ΕΚΚΙΝΗΣΗΣ:
------------------
1. Ταμείο & Λιανική Πώληση (POS):
   Κάντε διπλό κλικ στο αρχείο: Run-Proteus-POS.bat

2. Studio Διαχείρισης, Αποθήκη & WebStore:
   Κάντε διπλό κλικ στο αρχείο: Run-Proteus-Studio.bat

ΡΥΘΜΙΣΕΙΣ (ΑΦΜ, myDATA, Εκτυπωτές):
-----------------------------------
Επεξεργαστείτε το αρχείο 'config\proteus.toml' με έναν απλό επεξεργαστή κειμένου
(π.χ. Σημειωματάριο / Notepad) για να ορίσετε το ΑΦΜ, το όνομα καταστήματος και
τα στοιχεία πρόσβασης myDATA της ΑΑΔΕ.

ΑΝΤΙΓΡΑΦΑ ΑΣΦΑΛΕΙΑΣ (BACKUP):
-----------------------------
Όλα τα δεδομένα, οι πωλήσεις και οι αποθήκες αποθηκεύονται τοπικά στον φάκελο:
  data\store.db
Για πλήρες αντίγραφο ασφαλείας, αρκεί να αντιγράψετε τον φάκελο 'data' σε ένα
εξωτερικό USB flash drive ή σε ασφαλή τοπικό δίσκο.

ΣΥΜΜΟΡΦΩΣΗ:
-----------
- myDATA (Α.1138/2020) & Διασύνδεση EFT/POS (Α.1155/2023)
- Ψηφιακή Κάρτα Εργασίας ΕΡΓΑΝΗ ΙΙ
- Κανονισμός GDPR & Ελληνικός Νόμος 4624/2019
==============================================================================
"@
Set-Content -Path $readme -Value $readmeContent -Encoding UTF8
Write-Host "    [OK] Generated merchant guide: README-PORTABLE.txt" -ForegroundColor Green

# 7. Generate SHA-256 integrity checksum manifest
$shaManifest = Join-Path $OutputDir "SHA256SUMS.txt"
$filesToHash = Get-ChildItem -Path $OutputDir -Recurse -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" }
$hashLines = @()

foreach ($file in $filesToHash) {
    $relative = $file.FullName.Substring($OutputDir.Length).TrimStart("\", "/")
    $hash = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash.ToLower()
    $hashLines += "$hash  $relative"
}

Set-Content -Path $shaManifest -Value ($hashLines -join "`r`n") -Encoding ASCII
Write-Host "    [OK] Generated SHA-256 integrity manifest ($($hashLines.Count) files)" -ForegroundColor Green

Write-Host "`n>>> [Proteus] Portable Bundle successfully assembled at:" -ForegroundColor Cyan
Write-Host "    $OutputDir" -ForegroundColor White

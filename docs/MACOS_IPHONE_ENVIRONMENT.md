# macOS & Apple iPhone Environment Guide (Windows Host)

This guide explains how to build, test, preview, and virtualize Proteus for macOS and Apple iPhone/iPad without owning a physical Mac.

---

## 1. Instant In-Engine Preview (Zero-Setup)
Proteus provides native Apple device presets directly in the PDS Canvas and Play mode:

- **iPhone 16 Pro Preset** (`393 × 852 pt`):
  - Renders authentic Apple Dynamic Island pill and iOS Home Bar indicator.
  - Enforces Apple HIG 44.0pt minimum touch targets.
  - Applies iOS safe area insets (54pt top, 34pt bottom).
- **macOS MacBook Preset** (`1512 × 982 pt`):
  - Renders native macOS traffic light window controls (Close `#FF5F56`, Minimize `#FFBD2E`, Zoom `#27C93F`).
  - Desktop mouse target metrics (24pt) and top menu bar spacing.

To use: Open PDS Studio (`cargo run -p proteus`), select the **iPhone 16 Pro** or **macOS MacBook** button from the top canvas device toolbar.

---

## 2. Dedicated Mobile Touch Client (`proteus-mobile`)
To test the mobile interface in a standalone handheld phone window:
```bash
cargo run -p proteus-mobile
```
- Spawns a dedicated `393 × 852` mobile window.
- Fully wired to live SQLite (zero mock data): view tickets, perform intake, optical scan, and inspect store profile.

---

## 3. macOS Virtual Machine via Docker-OSX (Full GUI & iOS Simulator)
For running full macOS with Xcode and the Apple iOS Simulator directly on Windows:

### Requirements:
- Windows 10/11 with WSL2 enabled.
- Docker Desktop with WSL2 backend.

### Launching:
Run the automated launcher script from PowerShell:
```powershell
.\scripts\launch_macos_vm.ps1
```
Or with Docker Compose:
```bash
docker compose -f project/deploy/docker-compose.macos.yml up -d
```

### Viewing macOS on Screen:
- **Direct in Web Browser**: Open `http://localhost:6080` (no installation required).
- **VNC Viewer**: Connect to `localhost:5900`.
- **SSH Terminal**: `ssh -p 50922 arch@localhost` (password: `alpine`).

The local `CRM-Builder` workspace is automatically mounted inside macOS at `/home/arch/crm-builder`.

---

## 4. Cross-Compiling Native macOS Binaries from Windows
To compile authentic macOS Mach-O executable files (`.app` / binaries) directly on Windows:
```powershell
.\scripts\build_macos_binaries.ps1 -Package proteus-client -Target all
```
Generates binaries for:
- Apple Silicon M1/M2/M3/M4: `target/aarch64-apple-darwin/release/proteus-client`
- Intel Macs: `target/x86_64-apple-darwin/release/proteus-client`

# Proteus Mobile (`proteus-mobile`)

PDS Mobile Touch Client & Android NDK export module for Proteus CRM.

## Features
- **Strict Rule 5 Compliance**: Zero mock data. Reads and writes tickets and shop branding directly to/from SQLite.
- **Apple HIG & Android Material 44pt Touch Targets**: Designed specifically for touchscreen interactions without accidental touches.
- **Safe Area Insets**: Accommodates Dynamic Island, status bar notches, and system home bars.
- **Dual Target**: Compiles both as a standalone native simulator on Windows/macOS/Linux and as an Android `.apk` / `.aab` package.

## Quick Launch (Desktop Mobile Simulator)
```bash
cargo run -p proteus-mobile
```
Opens a dedicated 393 × 852 viewport simulating mobile touch operation.

## Android Extraction & Build Instructions
When ready to extract the real Android APK:
1. Ensure Android NDK (`r25b` or newer) is installed.
2. Install `cargo-apk`:
   ```bash
   cargo install cargo-apk
   ```
3. Build debug or release APK:
   ```bash
   cargo apk build --package proteus-mobile --release
   ```
   The resulting `.apk` is generated in `target/release/apk/proteus_mobile.apk`.

## iOS Sideloading & Enterprise OTA Instructions
In compliance with the EU Digital Markets Act (DMA) and zero-cost distribution rules:
1. **AltStore PAL / SideStore Feed**:
   The companion app publishes an authentic `apps.json` catalog generated via `proteus-core::standalone::ios::AltStoreSource`.
2. **On-Device Autonomous Re-signing**:
   SideStore utilizes a local WireGuard loopback VPN (`127.0.0.1:22` pairing profile) to re-sign `proteus-mobile.ipa` on the device every 7 days without connecting to a computer.
3. **Wireless Apple Enterprise OTA Installation**:
   Enterprises with MDM or in-house provisioning install the app wirelessly via Safari using `itms-services://?action=download-manifest&url=.../manifest.plist`.
   Full blueprint available in `docs/IOS_SIDELOADING_ARCHITECTURE.md`.

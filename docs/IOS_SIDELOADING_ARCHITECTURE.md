# iOS Sideloading & Distribution Architecture: Proteus Sovereign OS

## Executive Summary

Under Apple's traditional iOS walled garden, distributing enterprise B2B applications outside the public App Store previously required either an Apple Enterprise Developer Program license ($299/year with high audit risk) or public App Store review (incompatible with custom air-gapped deployments and proprietary tenant schemas).

With the enforcement of the European Union Digital Markets Act (DMA, Regulation EU 2022/1925) and mature community sideloading infrastructures (AltStore PAL, SideStore, and OTA Enterprise Manifests), Proteus can achieve sovereign, zero-gatekeeper deployment to iOS devices for Greek and European SMBs.

---

## 1. Distribution Vectors & Evaluation

| Vector | Mechanism | Cost | User Friction | Expiration / Refresh | Suitability |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **EU DMA AltStore PAL** | Alternative App Marketplace (MarketplaceKit API) | €0 (under 1M installs) | Low (1-click via Safari in EU) | Permanent (system-managed) | **Primary for EU Customers** |
| **Enterprise OTA Manifest** | HTTPS Wireless `manifest.plist` | $299/yr Apple Org | Low (Install via company QR/URL) | 12-Month Profile Renewal | **Fleet / Managed Stores** |
| **SideStore (WireGuard)** | On-device VPN local loopback re-signing | €0 (Free Apple ID) | Medium (Initial PC pairing) | 7-day auto-refresh over LAN/Wi-Fi | **Technicians / Standalone** |
| **PWA Web Clip Fallback** | Safari Progressive Web App + LocalStorage Cache | €0 | Zero (Add to Home Screen) | Never expires, but no background sync | **Zero-Install Fallback** |

---

## 2. Technical Specifications

### 2.1 EU DMA AltStore / MarketplaceKit Source (`apps.json`)
AltStore and AltStore PAL consume an encrypted or HTTPS JSON manifest listing available applications, versions, and direct IPA binary download URLs.
- **Endpoint**: `https://cdn.proteus.gr/ios/apps.json`
- **Fields Required**:
  - `name`: "Proteus Companion & POS"
  - `bundleIdentifier`: `gr.proteus.companion`
  - `developerName`: "Proteus Sovereign Systems IKE"
  - `version`: e.g. `1.0.0`
  - `downloadURL`: Signed HTTPS URL pointing to the compiled `.ipa` binary
  - `localizedDescription`: Highlighting offline-first SQLite POS, Ergani II card, and ESL integration.

### 2.2 Wireless Over-The-Air (OTA) Manifest (`manifest.plist`)
For businesses utilizing enterprise or ad-hoc provisioning, iOS Safari supports direct installation through the `itms-services://` protocol scheme:
```xml
itms-services://?action=download-manifest&url=https://hub.local:8443/ios/manifest.plist
```
- **Requirements**:
  - Valid HTTPS TLS certificate (self-signed root CA installed or Let's Encrypt).
  - MIME type `application/octet-stream` for `.ipa` and `text/xml` for `manifest.plist`.

### 2.3 SideStore On-Device Loopback
SideStore eliminates the need for a persistent desktop server (AltServer) by hosting a local VPN server on `127.0.0.1` using WireGuard. The device signs its own applications over loopback without connecting to a computer after initial pairing.

---

## 3. Recommended Proteus Roadmap Implementation

1. **Phase 5.2.1**: Implement the iOS Manifest Engine (`proteus-core::standalone::ios`):
   - Generate standards-compliant AltStore `apps.json` sources.
   - Generate wireless OTA `manifest.plist` packages for local LAN and cloud deployment.
2. **Phase 5.2.2**: Integrate iOS source publication into `proteus-web` distribution endpoints.
3. **Phase 5.2.3**: Deliver zero-cable QR installer modal in `proteus-client` for immediate iPhone/iPad deployment on local LAN.

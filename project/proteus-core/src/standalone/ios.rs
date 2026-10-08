//! iOS Sideloading & Over-The-Air (OTA) Distribution Manifest Engine.
//!
//! Generates EU DMA AltStore PAL / SideStore `apps.json` repository feeds and
//! Apple enterprise / ad-hoc wireless `manifest.plist` containers.

use serde::{Deserialize, Serialize};

/// Application entry inside an AltStore or SideStore repository source feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltStoreApp {
    pub name: String,
    #[serde(rename = "bundleIdentifier")]
    pub bundle_identifier: String,
    #[serde(rename = "developerName")]
    pub developer_name: String,
    pub subtitle: String,
    pub version: String,
    #[serde(rename = "versionDate")]
    pub version_date: String,
    #[serde(rename = "versionDescription")]
    pub version_description: String,
    #[serde(rename = "downloadURL")]
    pub download_url: String,
    #[serde(rename = "localizedDescription")]
    pub localized_description: String,
    #[serde(rename = "iconURL")]
    pub icon_url: String,
    #[serde(rename = "tintColor")]
    pub tint_color: String,
    pub size: u64,
}

/// AltStore / SideStore Community Source Repository Manifest (`apps.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltStoreSource {
    pub name: String,
    pub identifier: String,
    pub subtitle: String,
    pub description: String,
    #[serde(rename = "iconURL")]
    pub icon_url: String,
    #[serde(rename = "headerURL")]
    pub header_url: String,
    pub website: String,
    pub apps: Vec<AltStoreApp>,
}

impl AltStoreSource {
    /// Creates a default Proteus Sovereign iOS community source.
    pub fn new_proteus_source(base_url: &str) -> Self {
        Self {
            name: "Proteus Sovereign Apps".to_string(),
            identifier: "gr.proteus.source".to_string(),
            subtitle: "Sovereign Retail, POS & ERP Tools for iOS".to_string(),
            description: "Zero-cloud, sovereign, offline-first applications compliant with Greek tax and EU regulations.".to_string(),
            icon_url: format!("{}/icons/proteus_icon.png", base_url.trim_end_matches('/')),
            header_url: format!("{}/banners/proteus_header.png", base_url.trim_end_matches('/')),
            website: "https://proteus.gr".to_string(),
            apps: Vec::new(),
        }
    }

    /// Serializes the source repository to formatted JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("AltStore JSON serialization error: {}", e))
    }
}

/// Wireless Over-The-Air (OTA) iOS Enterprise / Ad-Hoc Deployment Manifest Generator.
pub struct IosOtaManifest {
    pub title: String,
    pub bundle_identifier: String,
    pub bundle_version: String,
    pub ipa_download_url: String,
    pub display_image_url: String,
    pub full_size_image_url: String,
}

impl IosOtaManifest {
    pub fn new(
        title: impl Into<String>,
        bundle_identifier: impl Into<String>,
        bundle_version: impl Into<String>,
        ipa_download_url: impl Into<String>,
    ) -> Self {
        let ipa_url = ipa_download_url.into();
        let base_dir = ipa_url
            .rsplit_once('/')
            .map(|(prefix, _)| prefix.to_string())
            .unwrap_or_else(|| "https://proteus.gr/ios".to_string());
        Self {
            title: title.into(),
            bundle_identifier: bundle_identifier.into(),
            bundle_version: bundle_version.into(),
            ipa_download_url: ipa_url,
            display_image_url: format!("{}/icon57.png", base_dir),
            full_size_image_url: format!("{}/icon512.png", base_dir),
        }
    }

    /// Generates valid Apple XML Property List (`manifest.plist`) for `itms-services://` OTA installation.
    pub fn to_plist_xml(&self) -> String {
        format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>items</key>
    <array>
        <dict>
            <key>assets</key>
            <array>
                <dict>
                    <key>kind</key>
                    <string>software-package</string>
                    <key>url</key>
                    <string>{}</string>
                </dict>
                <dict>
                    <key>kind</key>
                    <string>display-image</string>
                    <key>needs-shine</key>
                    <false/>
                    <key>url</key>
                    <string>{}</string>
                </dict>
                <dict>
                    <key>kind</key>
                    <string>full-size-image</string>
                    <key>needs-shine</key>
                    <false/>
                    <key>url</key>
                    <string>{}</string>
                </dict>
            </array>
            <key>metadata</key>
            <dict>
                <key>bundle-identifier</key>
                <string>{}</string>
                <key>bundle-version</key>
                <string>{}</string>
                <key>kind</key>
                <string>software</string>
                <key>title</key>
                <string>{}</string>
            </dict>
        </dict>
    </array>
</dict>
</plist>"#,
            self.ipa_download_url,
            self.display_image_url,
            self.full_size_image_url,
            self.bundle_identifier,
            self.bundle_version,
            self.title
        )
    }

    /// Returns the Safari clickable `itms-services://` URL scheme for 1-click installation.
    pub fn itms_service_url(&self, manifest_plist_url: &str) -> String {
        format!("itms-services://?action=download-manifest&url={}", manifest_plist_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_altstore_source_generation_and_json() {
        let mut source = AltStoreSource::new_proteus_source("https://cdn.proteus.gr");
        let app = AltStoreApp {
            name: "Proteus Mobile POS".to_string(),
            bundle_identifier: "gr.proteus.pos".to_string(),
            developer_name: "Proteus Sovereign Systems".to_string(),
            subtitle: "Offline Retail & Fiscal Touch Terminal".to_string(),
            version: "1.0.0".to_string(),
            version_date: "2026-10-06".to_string(),
            version_description: "Initial sovereign release".to_string(),
            download_url: "https://cdn.proteus.gr/ios/ProteusPos.ipa".to_string(),
            localized_description: "Zero-cloud Greek fiscal POS with thermal printing.".to_string(),
            icon_url: "https://cdn.proteus.gr/icons/pos.png".to_string(),
            tint_color: "#2563EB".to_string(),
            size: 15_420_000,
        };
        source.apps.push(app);

        let json = source.to_json().unwrap();
        assert!(json.contains("gr.proteus.source"));
        assert!(json.contains("Proteus Mobile POS"));
        assert!(json.contains("gr.proteus.pos"));
        assert!(json.contains("https://cdn.proteus.gr/ios/ProteusPos.ipa"));
    }

    #[test]
    fn test_ios_ota_manifest_plist_structure() {
        let manifest = IosOtaManifest::new(
            "Proteus Companion",
            "gr.proteus.companion",
            "1.2.0",
            "https://hub.local:8443/ios/ProteusCompanion.ipa",
        );

        let xml = manifest.to_plist_xml();
        assert!(xml.contains("<!DOCTYPE plist PUBLIC"));
        assert!(xml.contains("<string>software-package</string>"));
        assert!(xml.contains("<string>https://hub.local:8443/ios/ProteusCompanion.ipa</string>"));
        assert!(xml.contains("<string>gr.proteus.companion</string>"));
        assert!(xml.contains("<string>1.2.0</string>"));
        assert!(xml.contains("<string>Proteus Companion</string>"));

        let itms_url = manifest.itms_service_url("https://hub.local:8443/ios/manifest.plist");
        assert_eq!(
            itms_url,
            "itms-services://?action=download-manifest&url=https://hub.local:8443/ios/manifest.plist"
        );
    }
}

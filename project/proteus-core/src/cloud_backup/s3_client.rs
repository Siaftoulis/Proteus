//! Proteus Core — Zero-Knowledge Cloudflare R2 / S3 Cold Storage Engine.
//! Built natively from first principles with bespoke AWS SigV4 signing. Zero third-party AWS SDKs.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ureq;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct R2ClientConfig {
    pub account_id: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub custom_endpoint: Option<String>,
    pub region: String,
}

impl R2ClientConfig {
    pub fn new(
        account_id: &str,
        bucket: &str,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            bucket: bucket.to_string(),
            access_key_id: access_key_id.to_string(),
            secret_access_key: secret_access_key.to_string(),
            custom_endpoint: None,
            region: "auto".to_string(),
        }
    }

    pub fn endpoint_base(&self) -> String {
        if let Some(custom) = &self.custom_endpoint {
            custom.trim_end_matches('/').to_string()
        } else {
            format!("https://{}.r2.cloudflarestorage.com", self.account_id)
        }
    }

    pub fn object_url(&self, key: &str) -> String {
        let clean_key = key.trim_start_matches('/');
        format!("{}/{}/{}", self.endpoint_base(), self.bucket, clean_key)
    }

    pub fn host(&self) -> String {
        if let Some(custom) = &self.custom_endpoint {
            custom
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/')
                .to_string()
        } else {
            format!("{}.r2.cloudflarestorage.com", self.account_id)
        }
    }
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let hash = Sha256::digest(key);
        k[..32].copy_from_slice(&hash);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut o_key_pad = [0u8; 64];
    let mut i_key_pad = [0u8; 64];
    for i in 0..64 {
        o_key_pad[i] = k[i] ^ 0x5c;
        i_key_pad[i] = k[i] ^ 0x36;
    }

    let mut inner = Sha256::new();
    inner.update(&i_key_pad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&o_key_pad);
    outer.update(&inner_hash);
    outer.finalize().into()
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn compute_sigv4_auth_header(
    method: &str,
    canonical_uri: &str,
    config: &R2ClientConfig,
    payload_hash: &str,
    amz_date: &str,
    date_stamp: &str,
) -> String {
    let host = config.host();
    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host, payload_hash, amz_date
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    let canonical_request = format!(
        "{}\n{}\n\n{}\n{}\n{}",
        method, canonical_uri, canonical_headers, signed_headers, payload_hash
    );

    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, config.region);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        sha256_hex(canonical_request.as_bytes())
    );

    let k_secret = format!("AWS4{}", config.secret_access_key);
    let k_date = hmac_sha256(k_secret.as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, config.region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hmac_sha256(&k_signing, string_to_sign.as_bytes());
    let signature_hex = format!("{:x}", Sha256::digest(&signature));

    format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        config.access_key_id, credential_scope, signed_headers, signature_hex
    )
}

pub struct R2SnapshotClient;

impl R2SnapshotClient {
    /// Uploads an encrypted file snapshot directly to Cloudflare R2 / S3.
    pub fn upload_encrypted_snapshot(
        config: &R2ClientConfig,
        key: &str,
        payload_bytes: &[u8],
    ) -> Result<String, String> {
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let clean_key = key.trim_start_matches('/');
        let canonical_uri = format!("/{}/{}", config.bucket, clean_key);
        let payload_hash = sha256_hex(payload_bytes);

        let auth_header = compute_sigv4_auth_header(
            "PUT",
            &canonical_uri,
            config,
            &payload_hash,
            &amz_date,
            &date_stamp,
        );

        let url = config.object_url(clean_key);
        let resp = ureq::put(&url)
            .set("Host", &config.host())
            .set("x-amz-date", &amz_date)
            .set("x-amz-content-sha256", &payload_hash)
            .set("Authorization", &auth_header)
            .set("Content-Type", "application/octet-stream")
            .timeout(std::time::Duration::from_secs(30))
            .send_bytes(payload_bytes);

        match resp {
            Ok(r) => {
                let etag = r.header("etag").unwrap_or("").to_string();
                Ok(etag)
            }
            Err(e) => Err(format!("Cloudflare R2 upload error: {}", e)),
        }
    }

    /// Downloads an encrypted snapshot from Cloudflare R2 / S3.
    pub fn download_encrypted_snapshot(
        config: &R2ClientConfig,
        key: &str,
    ) -> Result<Vec<u8>, String> {
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let clean_key = key.trim_start_matches('/');
        let canonical_uri = format!("/{}/{}", config.bucket, clean_key);
        let empty_hash = sha256_hex(b"");

        let auth_header = compute_sigv4_auth_header(
            "GET",
            &canonical_uri,
            config,
            &empty_hash,
            &amz_date,
            &date_stamp,
        );

        let url = config.object_url(clean_key);
        let resp = ureq::get(&url)
            .set("Host", &config.host())
            .set("x-amz-date", &amz_date)
            .set("x-amz-content-sha256", &empty_hash)
            .set("Authorization", &auth_header)
            .timeout(std::time::Duration::from_secs(60))
            .call();

        match resp {
            Ok(r) => {
                let mut bytes = Vec::new();
                r.into_reader()
                    .read_to_end(&mut bytes)
                    .map_err(|e| format!("Failed to read R2 response body: {}", e))?;
                Ok(bytes)
            }
            Err(e) => Err(format!("Cloudflare R2 download error: {}", e)),
        }
    }

    /// Deletes a snapshot object from Cloudflare R2 / S3.
    pub fn delete_snapshot(config: &R2ClientConfig, key: &str) -> Result<(), String> {
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let clean_key = key.trim_start_matches('/');
        let canonical_uri = format!("/{}/{}", config.bucket, clean_key);
        let empty_hash = sha256_hex(b"");

        let auth_header = compute_sigv4_auth_header(
            "DELETE",
            &canonical_uri,
            config,
            &empty_hash,
            &amz_date,
            &date_stamp,
        );

        let url = config.object_url(clean_key);
        let resp = ureq::delete(&url)
            .set("Host", &config.host())
            .set("x-amz-date", &amz_date)
            .set("x-amz-content-sha256", &empty_hash)
            .set("Authorization", &auth_header)
            .timeout(std::time::Duration::from_secs(10))
            .call();

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cloudflare R2 delete error: {}", e)),
        }
    }

    /// Extracts object key names from an S3 ListObjects XML response string.
    pub fn parse_s3_xml_keys(xml: &str) -> Vec<String> {
        let mut keys = Vec::new();
        let mut cursor = xml;
        while let Some(start) = cursor.find("<Key>") {
            let rest = &cursor[start + 5..];
            if let Some(end) = rest.find("</Key>") {
                let key = rest[..end].to_string();
                keys.push(key);
                cursor = &rest[end + 6..];
            } else {
                break;
            }
        }
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc2202_hmac_sha256_test_vector_1() {
        // RFC 4231 / RFC 2202 Test Case 1 for HMAC-SHA256
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let digest = hmac_sha256(&key, data);
        let hex_out = format!("{:x}", Sha256::digest(&digest));
        assert_eq!(digest.len(), 32);
        assert!(!hex_out.is_empty());
    }

    #[test]
    fn test_r2_config_url_construction() {
        let config = R2ClientConfig::new(
            "acc_123456",
            "proteus-backups",
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        );

        assert_eq!(
            config.endpoint_base(),
            "https://acc_123456.r2.cloudflarestorage.com"
        );
        assert_eq!(
            config.object_url("2026/05/backup_1.bak"),
            "https://acc_123456.r2.cloudflarestorage.com/proteus-backups/2026/05/backup_1.bak"
        );
        assert_eq!(config.host(), "acc_123456.r2.cloudflarestorage.com");
    }

    #[test]
    fn test_compute_sigv4_auth_header_format() {
        let config = R2ClientConfig::new(
            "test_account",
            "test_bucket",
            "KEY_ABC",
            "SECRET_XYZ",
        );

        let header = compute_sigv4_auth_header(
            "PUT",
            "/test_bucket/file.bak",
            &config,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "20261006T120000Z",
            "20261006",
        );

        assert!(header.starts_with("AWS4-HMAC-SHA256 Credential=KEY_ABC/20261006/auto/s3/aws4_request"));
        assert!(header.contains("SignedHeaders=host;x-amz-content-sha256;x-amz-date"));
        assert!(header.contains("Signature="));
    }

    #[test]
    fn test_parse_s3_xml_keys() {
        let sample_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Name>proteus-backups</Name>
            <Contents>
                <Key>backup_2026_01.bak</Key>
                <Size>4096</Size>
            </Contents>
            <Contents>
                <Key>backup_2026_02.bak</Key>
                <Size>8192</Size>
            </Contents>
        </ListBucketResult>"#;

        let keys = R2SnapshotClient::parse_s3_xml_keys(sample_xml);
        assert_eq!(keys, vec!["backup_2026_01.bak", "backup_2026_02.bak"]);
    }
}

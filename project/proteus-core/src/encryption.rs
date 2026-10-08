//! Sovereign Authenticated Encryption & Key Derivation (XChaCha20-Poly1305 + Argon2id).
//! Implements strict strongly-typed cryptographic errors (Rule 1 & Rule 3).

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum CryptoError {
    #[error("Argon2id key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("XChaCha20-Poly1305 encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("XChaCha20-Poly1305 decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid encrypted payload size: expected at least 24 bytes, got {0}")]
    InvalidDataLength(usize),
}

impl From<CryptoError> for String {
    fn from(err: CryptoError) -> Self {
        err.to_string()
    }
}

pub fn derive_key(password: &str, salt: &str) -> Result<[u8; 32], CryptoError> {
    use argon2::Algorithm;
    use sha2::Digest;
    let params = argon2::Params::new(65536, 3, 4, Some(32)).unwrap();
    let argon2 = argon2::Argon2::new(Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt_hash: [u8; 16] = {
        let h = sha2::Sha256::digest(salt.as_bytes());
        let mut out = [0u8; 16];
        out.copy_from_slice(&h[..16]);
        out
    };
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), &salt_hash, &mut key)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    Ok(key)
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let mut nonce_bytes = [0u8; 24];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 24 {
        return Err(CryptoError::InvalidDataLength(data.len()));
    }
    let (nonce_bytes, ciphertext) = data.split_at(24);
    let nonce = XNonce::from_slice(nonce_bytes);
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let k1 = derive_key("password", "salt").unwrap();
        let k2 = derive_key("password", "salt").unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_derive_key_different_password() {
        let k1 = derive_key("password1", "salt").unwrap();
        let k2 = derive_key("password2", "salt").unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key("test-password", "test-device").unwrap();
        let data = b"hello world this is sensitive CRM data";
        let encrypted = encrypt(data, &key).unwrap();
        assert_ne!(encrypted, data);
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = derive_key("p", "s").unwrap();
        let encrypted = encrypt(b"", &key).unwrap();
        assert!(encrypted.len() >= 24);
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = derive_key("correct", "device").unwrap();
        let key2 = derive_key("wrong", "device").unwrap();
        let data = b"secret data";
        let encrypted = encrypt(data, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);
        assert!(matches!(result, Err(CryptoError::DecryptionFailed(_))));
    }

    #[test]
    fn test_decrypt_tampered_data_fails() {
        let key = derive_key("pwd", "dev").unwrap();
        let data = b"important";
        let mut encrypted = encrypt(data, &key).unwrap();
        encrypted[30] ^= 1;
        let result = decrypt(&encrypted, &key);
        assert!(matches!(result, Err(CryptoError::DecryptionFailed(_))));
    }

    #[test]
    fn test_decrypt_too_short_fails() {
        let key = derive_key("pwd", "dev").unwrap();
        let result = decrypt(&[0u8; 10], &key);
        assert_eq!(result, Err(CryptoError::InvalidDataLength(10)));
    }

    #[test]
    fn test_decrypt_nonce_only_fails() {
        let key = derive_key("pwd", "dev").unwrap();
        let result = decrypt(&[0u8; 24], &key);
        assert!(matches!(result, Err(CryptoError::DecryptionFailed(_))));
    }
}

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

pub fn derive_key(password: &str, salt: &str) -> Result<[u8; 32], String> {
    // ponytail: argon2id KDF with minimal params (fast enough for export/import)
    use argon2::Algorithm;
    use sha2::Digest;
    let params = argon2::Params::new(65536, 3, 4, Some(32)).unwrap();
    let argon2 = argon2::Argon2::new(Algorithm::Argon2id, argon2::Version::V0x13, params);
    // argon2 requires exactly 16 bytes of salt — hash the input to get a fixed-size salt
    let salt_hash: [u8; 16] = {
        let h = sha2::Sha256::digest(salt.as_bytes());
        let mut out = [0u8; 16];
        out.copy_from_slice(&h[..16]);
        out
    };
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), &salt_hash, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 24];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, data).map_err(|e| e.to_string())?;
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if data.len() < 24 {
        return Err("Invalid encrypted data".to_string());
    }
    let (nonce_bytes, ciphertext) = data.split_at(24);
    let nonce = XNonce::from_slice(nonce_bytes);
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|e| e.to_string())?;
    cipher.decrypt(nonce, ciphertext).map_err(|e| format!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::{decrypt, derive_key, encrypt};

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
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_data_fails() {
        let key = derive_key("pwd", "dev").unwrap();
        let data = b"important";
        let mut encrypted = encrypt(data, &key).unwrap();
        encrypted[30] ^= 1; // corrupt a byte
        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short_fails() {
        let key = derive_key("pwd", "dev").unwrap();
        let result = decrypt(&[0u8; 10], &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_nonce_only_fails() {
        let key = derive_key("pwd", "dev").unwrap();
        let result = decrypt(&[0u8; 24], &key);
        assert!(result.is_err());
    }
}

use crate::scene::ProjectDocument;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::RngCore;

const DEV_KEY: &[u8; 32] = b"an_example_very_secret_key_32_b!";

/// Save the arena document to a JSON file at the given path.
/// Uses `serde_json::to_string_pretty` for human-readable output.
pub fn save_project(doc: &ProjectDocument, path: &std::path::Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(doc).map_err(|e| format!("Serialize error: {}", e))?;
    std::fs::write(path, &json).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

/// Load a `ProjectDocument` from a JSON file at the given path.
/// Returns an error if the file doesn't exist or the JSON is malformed.
/// Verifies that every node's `id` matches its key in the arena HashMap.
pub fn load_project(path: &std::path::Path) -> Result<ProjectDocument, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    let doc: ProjectDocument = serde_json::from_str(&json).map_err(|e| format!("Deserialize error: {}", e))?;

    // Verify arena ID consistency
    for (key, node) in &doc.nodes {
        if key != &node.id {
            return Err(format!(
                "Arena key mismatch: key='{}' but node.id='{}'",
                key, node.id
            ));
        }
    }
    // Verify that every root_node_id exists as a key
    for rid in &doc.root_node_ids {
        if !doc.nodes.contains_key(rid) {
            return Err(format!("Root node '{}' not found in arena nodes", rid));
        }
    }

    Ok(doc)
}

const NONCE_LEN: usize = 24;

/// Save a `ProjectDocument` to a secure binary file (.proteus):
/// bincode → XChaCha20Poly1305 → nonce|ciphertext.
pub fn save_project_secure(doc: &ProjectDocument, path: &std::path::Path) -> Result<(), String> {
    let plaintext = bincode::serialize(doc).map_err(|e| format!("Bincode serialize error: {}", e))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let key = Key::from_slice(DEV_KEY);
    let cipher = XChaCha20Poly1305::new(key);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption error: {}", e))?;
    let mut payload = nonce_bytes.to_vec();
    payload.extend_from_slice(&ciphertext);
    std::fs::write(path, &payload).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

/// Load a `ProjectDocument` from a secure binary file (.proteus).
/// Validates the file is non-tampered via the AEAD authentication tag.
pub fn load_project_secure(path: &std::path::Path) -> Result<ProjectDocument, String> {
    let payload = std::fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    if payload.len() < NONCE_LEN {
        return Err("File too short — corrupted or not a valid .proteus file".into());
    }
    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let nonce = XNonce::from_slice(nonce_bytes);
    let key = Key::from_slice(DEV_KEY);
    let cipher = XChaCha20Poly1305::new(key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed — file is corrupted or was not created by this application".to_string())?;
    bincode::deserialize(&plaintext).map_err(|e| format!("Bincode deserialize error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{create_dummy_document};

    #[test]
    fn save_load_roundtrip() {
        let doc = create_dummy_document();
        let path = std::path::Path::new("_test_roundtrip.json");

        // Save
        save_project(&doc, path).unwrap();

        // Load
        let loaded = load_project(path).unwrap();

        // Clean up
        let _ = std::fs::remove_file(path);

        assert_eq!(doc.version, loaded.version);
        assert_eq!(doc.nodes.len(), loaded.nodes.len());
        assert_eq!(doc.root_node_ids, loaded.root_node_ids);

        // Spot-check a node
        for (key, node) in &doc.nodes {
            let ln = loaded.nodes.get(key).expect("key missing after roundtrip");
            assert_eq!(node.name, ln.name);
            assert_eq!(node.position, ln.position);
        }
    }

    #[test]
    fn secure_save_load_roundtrip() {
        let doc = create_dummy_document();
        let path = std::path::Path::new("_test_secure_roundtrip.proteus");

        save_project_secure(&doc, path).unwrap();
        let loaded = load_project_secure(path).unwrap();
        let _ = std::fs::remove_file(path);

        assert_eq!(doc.version, loaded.version);
        assert_eq!(doc.nodes.len(), loaded.nodes.len());
        assert_eq!(doc.root_node_ids, loaded.root_node_ids);
        for (key, node) in &doc.nodes {
            let ln = loaded.nodes.get(key).expect("key missing after roundtrip");
            assert_eq!(node.name, ln.name);
        }
    }

    #[test]
    fn secure_load_corrupted_file_fails() {
        let path = std::path::Path::new("_test_corrupt.proteus");
        // Write garbage
        std::fs::write(path, b"this is not a valid proteus file").unwrap();
        let result = load_project_secure(path);
        let _ = std::fs::remove_file(path);
        assert!(result.is_err(), "corrupted file should return an error");
    }
}

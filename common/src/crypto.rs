use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;

/// Encrypts a plaintext string using AES-256-GCM.
/// Returns a tuple of (ciphertext, nonce).
pub fn encrypt(value: &str, key_bytes: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|e| format!("Encryption error: {}", e))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypts AES-256-GCM encrypted bytes back to a UTF-8 string.
pub fn decrypt(
    ciphertext: &[u8],
    nonce_bytes: &[u8],
    key_bytes: &[u8; 32],
) -> Result<String, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    if nonce_bytes.len() != 12 {
        return Err("Invalid nonce length".to_string());
    }

    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption error: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_then_decrypt_roundtrips() {
        let key = [7u8; 32];
        let (ciphertext, nonce) = encrypt("super-secret-env-value", &key).unwrap();
        let plaintext = decrypt(&ciphertext, &nonce, &key).unwrap();
        assert_eq!(plaintext, "super-secret-env-value");
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let key = [1u8; 32];
        let wrong_key = [2u8; 32];
        let (ciphertext, nonce) = encrypt("value", &key).unwrap();
        assert!(decrypt(&ciphertext, &nonce, &wrong_key).is_err());
    }

    #[test]
    fn decrypt_rejects_invalid_nonce_length() {
        let key = [3u8; 32];
        let result = decrypt(&[0u8; 16], &[0u8; 8], &key);
        assert_eq!(result, Err("Invalid nonce length".to_string()));
    }
}

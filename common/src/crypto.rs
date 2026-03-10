use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
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

    let ciphertext = cipher.encrypt(nonce, value.as_bytes()).map_err(|e| format!("Encryption error: {}", e))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypts AES-256-GCM encrypted bytes back to a UTF-8 string.
pub fn decrypt(ciphertext: &[u8], nonce_bytes: &[u8], key_bytes: &[u8; 32]) -> Result<String, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    if nonce_bytes.len() != 12 {
        return Err("Invalid nonce length".to_string());
    }
    
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| format!("Decryption error: {}", e))?;
    
    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode error: {}", e))
}

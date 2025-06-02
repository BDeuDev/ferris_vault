use aes_gcm::{Aes256Gcm, KeyInit, Nonce}; // Or `Aes128Gcm`
use aes_gcm::aead::{Aead};
use base64::{engine::general_purpose, Engine as _};
use super::utils::derive_key_from_password;

pub fn decrypt(cipher_b64: &str, password: &str) -> Option<String> {
    let key = derive_key_from_password(password);
    let cipher = Aes256Gcm::new_from_slice(&key).ok()?;

    let data = general_purpose::STANDARD.decode(cipher_b64).ok()?;
    if data.len() < 12 {
        return None;
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let decrypted = cipher.decrypt(nonce, ciphertext).ok()?;
    String::from_utf8(decrypted).ok()
}
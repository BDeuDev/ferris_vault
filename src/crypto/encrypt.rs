use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng, rand_core::RngCore};
use base64::{engine::general_purpose, Engine as _};
use super::utils::derive_key_from_password;

pub fn encrypt(plain: &str, password: &str) -> String {
    let key = derive_key_from_password(password);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plain.as_bytes()).unwrap();
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    general_purpose::STANDARD.encode(result)
}
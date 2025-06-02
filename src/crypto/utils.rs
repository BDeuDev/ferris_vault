use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

pub fn derive_key_from_password(password: &str) -> [u8; 32] {
    let salt = b"ferris-vault-salt";
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    key
}
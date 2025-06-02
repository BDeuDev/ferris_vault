use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
struct MasterKeyHash {
    hash: String,
}

pub fn hash_master_key(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

pub fn save_master_key_hash(hash: &str) {
    let data = MasterKeyHash {
        hash: hash.to_string(),
    };
    let json = serde_json::to_string_pretty(&data).unwrap();
    std::fs::write("src/master_key.json", json).ok();
}

pub fn load_master_key_hash() -> Option<String> {
    if let Ok(json) = std::fs::read_to_string("src/master_key.json") {
        if let Ok(data) = serde_json::from_str::<MasterKeyHash>(&json) {
            return Some(data.hash);
        }
    }
    None
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

const NONCE_SIZE: usize = 12;

pub struct SecretManager {
    cipher: Aes256Gcm,
}

impl SecretManager {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        SecretManager { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&result))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        let data = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| e.to_string())?;

        if data.len() < NONCE_SIZE {
            return Err("Invalid encrypted data".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;

        String::from_utf8(plaintext).map_err(|e| e.to_string())
    }
}

pub fn get_or_create_key() -> [u8; 32] {
    let key_path = dirs::config_dir()
        .unwrap()
        .join("mcp-manager")
        .join(".key");

    if key_path.exists() {
        let key_data = std::fs::read(&key_path).unwrap();
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_data[..32]);
        key
    } else {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        std::fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        std::fs::write(&key_path, &key).unwrap();
        key
    }
}

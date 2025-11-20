use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use keyring::Entry;
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

pub fn get_or_create_key() -> Result<[u8; 32], String> {
    let entry = Entry::new("mcp-toolkit", "master-encryption-key")
        .map_err(|e| format!("Failed to access OS keychain: {}", e))?;

    match entry.get_password() {
        Ok(password) => {
            // Key exists in keychain, decode it
            let bytes = general_purpose::STANDARD
                .decode(&password)
                .map_err(|e| format!("Failed to decode key from keychain: {}", e))?;

            if bytes.len() != 32 {
                return Err("Invalid key length in keychain".to_string());
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(key)
        }
        Err(keyring::Error::NoEntry) => {
            // Generate new key
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);

            // Store in keychain
            let encoded = general_purpose::STANDARD.encode(&key);
            entry
                .set_password(&encoded)
                .map_err(|e| format!("Failed to store key in OS keychain: {}", e))?;

            Ok(key)
        }
        Err(e) => Err(format!("OS keychain error: {}", e)),
    }
}

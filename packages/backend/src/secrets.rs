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
    // Check if we're in test mode to avoid macOS Keychain permission prompts
    if std::env::var("MCP_TEST_MODE").is_ok() {
        // Use a deterministic test key (DO NOT use in production!)
        let test_key: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        return Ok(test_key);
    }

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

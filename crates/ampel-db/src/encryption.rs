use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;

use ampel_core::errors::{AmpelError, AmpelResult};

const NONCE_SIZE: usize = 12;

/// Encryption service for sensitive data (OAuth tokens, etc.)
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// Create a new encryption service with a 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key size");
        Self { cipher }
    }

    /// Create from a base64-encoded key string
    pub fn from_base64_key(key_base64: &str) -> AmpelResult<Self> {
        let key_bytes = BASE64
            .decode(key_base64)
            .map_err(|e| AmpelError::EncryptionError(format!("Invalid base64 key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(AmpelError::EncryptionError(format!(
                "Key must be 32 bytes, got {}",
                key_bytes.len()
            )));
        }

        let key: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| AmpelError::EncryptionError("Invalid key length".to_string()))?;

        Ok(Self::new(&key))
    }

    /// Encrypt plaintext, returning nonce + ciphertext
    pub fn encrypt(&self, plaintext: &str) -> AmpelResult<Vec<u8>> {
        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rng.fill(&mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AmpelError::EncryptionError(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt ciphertext (expects nonce + ciphertext)
    pub fn decrypt(&self, encrypted: &[u8]) -> AmpelResult<String> {
        if encrypted.len() < NONCE_SIZE {
            return Err(AmpelError::EncryptionError(
                "Encrypted data too short".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AmpelError::EncryptionError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AmpelError::EncryptionError(format!("Invalid UTF-8: {}", e)))
    }
}

/// Generate a new random 32-byte encryption key as base64
pub fn generate_encryption_key() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    BASE64.encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = i as u8;
        }
        key
    }

    #[test]
    fn test_encrypt_decrypt() {
        let service = EncryptionService::new(&test_key());
        let plaintext = "my-secret-oauth-token";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_nonces() {
        let service = EncryptionService::new(&test_key());
        let plaintext = "same-plaintext";

        let encrypted1 = service.encrypt(plaintext).unwrap();
        let encrypted2 = service.encrypt(plaintext).unwrap();

        // Different encryptions should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same value
        assert_eq!(service.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(service.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_generate_key() {
        let key1 = generate_encryption_key();
        let key2 = generate_encryption_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should be valid base64 and 32 bytes when decoded
        let decoded = BASE64.decode(&key1).unwrap();
        assert_eq!(decoded.len(), 32);
    }
}

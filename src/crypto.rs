use aes_gcm::{ aead::{ Aead, KeyInit, OsRng }, Aes256Gcm, Nonce };
use base64::{ engine::general_purpose::STANDARD as B64, Engine };
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

use crate::error::AppError;

/// Cryptographic service for E2E at-rest encryption.
/// Uses AES-256-GCM with per-user derived keys via HKDF.
/// Even with database access, data cannot be read without the master key.
#[derive(Clone)]
pub struct CryptoService {
    master_key: Vec<u8>,
}

impl CryptoService {
    pub fn new(master_key_hex: &str) -> Result<Self, AppError> {
        let master_key = hex
            ::decode(master_key_hex)
            .map_err(|e| { AppError::EncryptionError(format!("Invalid master key hex: {}", e)) })?;
        if master_key.len() != 32 {
            return Err(
                AppError::EncryptionError("Master key must be 32 bytes (64 hex chars)".into())
            );
        }
        Ok(Self { master_key })
    }

    /// Derive a per-user encryption key using HKDF-SHA256.
    fn derive_user_key(&self, user_salt: &str) -> Result<[u8; 32], AppError> {
        let hk = Hkdf::<Sha256>::new(Some(user_salt.as_bytes()), &self.master_key);
        let mut okm = [0u8; 32];
        hk
            .expand(b"bsdy-e2e-encryption", &mut okm)
            .map_err(|e| AppError::EncryptionError(format!("HKDF expand failed: {}", e)))?;
        Ok(okm)
    }

    /// Encrypt plaintext for a specific user. Returns base64(nonce + ciphertext).
    pub fn encrypt(&self, plaintext: &str, user_salt: &str) -> Result<String, AppError> {
        let key = self.derive_user_key(user_salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e|
            AppError::EncryptionError(format!("Cipher init failed: {}", e))
        )?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::EncryptionError(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext, then base64
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(B64.encode(combined))
    }

    /// Decrypt base64(nonce + ciphertext) for a specific user.
    pub fn decrypt(&self, encrypted_b64: &str, user_salt: &str) -> Result<String, AppError> {
        let key = self.derive_user_key(user_salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e|
            AppError::EncryptionError(format!("Cipher init failed: {}", e))
        )?;

        let combined = B64.decode(encrypted_b64).map_err(|e| {
            AppError::EncryptionError(format!("Base64 decode failed: {}", e))
        })?;

        if combined.len() < 12 {
            return Err(AppError::EncryptionError("Ciphertext too short".into()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::EncryptionError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext).map_err(|e|
            AppError::EncryptionError(format!("UTF-8 decode failed: {}", e))
        )
    }

    /// Encrypt an optional field. Returns None if input is None.
    pub fn encrypt_optional(
        &self,
        plaintext: Option<&str>,
        user_salt: &str
    ) -> Result<Option<String>, AppError> {
        match plaintext {
            Some(text) => Ok(Some(self.encrypt(text, user_salt)?)),
            None => Ok(None),
        }
    }

    /// Decrypt an optional field. Returns None if input is None.
    pub fn decrypt_optional(
        &self,
        encrypted: Option<&str>,
        user_salt: &str
    ) -> Result<Option<String>, AppError> {
        match encrypted {
            Some(enc) => Ok(Some(self.decrypt(enc, user_salt)?)),
            None => Ok(None),
        }
    }

    /// Generate a random 32-char hex salt for a new user.
    pub fn generate_user_salt() -> String {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        hex::encode(salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key_hex = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let crypto = CryptoService::new(key_hex).unwrap();
        let salt = CryptoService::generate_user_salt();
        let plaintext = "Hello, sensitive mental health data!";
        let encrypted = crypto.encrypt(plaintext, &salt).unwrap();
        let decrypted = crypto.decrypt(&encrypted, &salt).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_salts_different_ciphertexts() {
        let key_hex = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let crypto = CryptoService::new(key_hex).unwrap();
        let salt1 = "user-salt-1";
        let salt2 = "user-salt-2";
        let plaintext = "Same data";
        let enc1 = crypto.encrypt(plaintext, salt1).unwrap();
        let enc2 = crypto.encrypt(plaintext, salt2).unwrap();
        assert_ne!(enc1, enc2);
    }
}

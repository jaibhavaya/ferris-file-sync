use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, RngCore};

pub fn encrypt_token(token: &str, encryption_key: &str) -> Result<String> {
    let key_bytes = get_key_bytes(encryption_key);

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| anyhow!("Failed to create cipher"))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = token.as_bytes();
    let ciphertext =
        cipher.encrypt(nonce, plaintext).map_err(|_| anyhow!("Failed to encrypt token"))?;

    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

pub fn decrypt_token(encrypted_token: &str, encryption_key: &str) -> Result<String> {
    let key_bytes = get_key_bytes(encryption_key);

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| anyhow!("Failed to create cipher"))?;

    let combined =
        BASE64.decode(encrypted_token).map_err(|e| anyhow!("Failed to decode base64: {}", e))?;

    if combined.len() < 12 {
        return Err(anyhow!("Invalid encrypted token: too short"));
    }

    let (nonce, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Failed to decrypt token"))?;
    let token = String::from_utf8(plaintext)
        .map_err(|e| anyhow!("Failed to convert decrypted bytes to string: {}", e))?;

    Ok(token)
}

fn get_key_bytes(key: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let hash = hasher.finalize();

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&hash);
    key_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        let test_token = "test-token-value";
        let encryption_key = "test-encryption-key";

        // Encrypt the token
        let encrypted = encrypt_token(test_token, encryption_key)?;

        // Decrypt the token
        let decrypted = decrypt_token(&encrypted, encryption_key)?;

        assert_eq!(test_token, decrypted);

        Ok(())
    }
}


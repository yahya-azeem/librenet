use serde::{Deserialize, Serialize};
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
use rand::RngCore;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GarlicPacket {
    pub next_hop: Option<String>, // PeerId as string
    pub payload: Vec<u8>,
}

impl GarlicPacket {
    /// Wraps a payload in multiple layers of encryption for the given hops.
    pub fn wrap(payload: Vec<u8>, hops: Vec<[u8; 32]>) -> Vec<u8> {
        let mut current_payload = payload;
        
        // Wrap from the last hop to the first
        for key_bytes in hops.iter().rev() {
            let key = Key::<Aes256Gcm>::from_slice(key_bytes);
            let cipher = Aes256Gcm::new(key);
            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let encrypted = cipher.encrypt(nonce, current_payload.as_ref()).expect("Encryption fails");
            
            // Prepend nonce to the encrypted data
            let mut final_layer = nonce_bytes.to_vec();
            final_layer.extend(encrypted);
            current_payload = final_layer;
        }
        
        current_payload
    }

    pub fn unwrap(payload: &[u8], key_bytes: &[u8; 32]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        if payload.len() < 12 {
            return Err("Payload too short for nonce".into());
        }
        
        let nonce = Nonce::from_slice(&payload[..12]);
        let ciphertext = &payload[12..];
        
        let decrypted = cipher.decrypt(nonce, ciphertext).map_err(|e| format!("Decryption error: {:?}", e))?;
        Ok(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_garlic_wrap_unwrap_roundtrip() {
        let payload = b"hello secrete message through the garlic nodes".to_vec();
        let hop1 = [1u8; 32];
        let hop2 = [2u8; 32];
        let hop3 = [3u8; 32];
        let hops = vec![hop1, hop2, hop3];

        let wrapped = GarlicPacket::wrap(payload.clone(), hops);

        let unwrapped1 = GarlicPacket::unwrap(&wrapped, &hop1).unwrap();
        let unwrapped2 = GarlicPacket::unwrap(&unwrapped1, &hop2).unwrap();
        let final_payload = GarlicPacket::unwrap(&unwrapped2, &hop3).unwrap();

        assert_eq!(final_payload, payload);
    }
}

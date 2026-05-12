use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::{error::AuthError, ports::TokenGenerator};

#[derive(Clone, Debug)]
pub struct RandomTokenGenerator {
    bytes: usize,
}

impl RandomTokenGenerator {
    pub fn new(bytes: usize) -> Self {
        Self { bytes }
    }
}

impl Default for RandomTokenGenerator {
    fn default() -> Self {
        Self { bytes: 32 }
    }
}

impl TokenGenerator for RandomTokenGenerator {
    fn generate_token(&self) -> Result<String, AuthError> {
        if self.bytes < 16 {
            return Err(AuthError::TokenGeneration(
                "token entropy must be at least 128 bits".to_owned(),
            ));
        }

        let mut bytes = vec![0_u8; self.bytes];
        OsRng.fill_bytes(&mut bytes);
        let token = hex::encode(&bytes);
        bytes.zeroize();
        Ok(token)
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

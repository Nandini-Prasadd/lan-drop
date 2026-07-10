use ed25519_dalek::SigningKey;
use rand_core::{OsRng, RngCore};

#[derive(Clone)]
pub struct DeviceIdentity {
    secret_key: [u8; 32],
    public_key: [u8; 32],
}

impl DeviceIdentity {
    pub fn generate() -> Self {
        let mut secret_key = [0_u8; 32];
        let mut rng = OsRng;
        rng.fill_bytes(&mut secret_key);
        Self::from_secret_key(secret_key)
    }

    pub fn from_secret_key(secret_key: [u8; 32]) -> Self {
        let public_key = SigningKey::from_bytes(&secret_key)
            .verifying_key()
            .to_bytes();
        Self {
            secret_key,
            public_key,
        }
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.public_key
    }

    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.secret_key
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceIdentity;

    #[test]
    fn reconstructing_an_identity_preserves_its_public_key() {
        let identity = DeviceIdentity::generate();
        let restored = DeviceIdentity::from_secret_key(identity.secret_key_bytes());

        assert_eq!(restored.public_key(), identity.public_key());
    }
}

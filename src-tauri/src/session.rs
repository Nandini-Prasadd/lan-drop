use std::fmt;

use rand_core::{OsRng, RngCore};
use snow::{params::NoiseParams, Builder, HandshakeState, TransportState};

const NOISE_PATTERN: &str = "Noise_XX_25519_ChaChaPoly_SHA256";

#[derive(Debug)]
pub enum SessionError {
    Noise(snow::Error),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noise(error) => write!(formatter, "Secure peer session failed: {error}"),
        }
    }
}

impl From<snow::Error> for SessionError {
    fn from(error: snow::Error) -> Self {
        Self::Noise(error)
    }
}

pub struct NoiseIdentity([u8; 32]);

impl NoiseIdentity {
    pub fn generate() -> Self {
        let mut key = [0_u8; 32];
        let mut rng = OsRng;
        rng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn private_key(&self) -> &[u8; 32] {
        &self.0
    }
}

pub fn initiator(identity: &NoiseIdentity) -> Result<HandshakeState, SessionError> {
    Builder::new(NOISE_PATTERN.parse::<NoiseParams>()?)?
        .local_private_key(identity.private_key())
        .build_initiator()
        .map_err(Into::into)
}

pub fn responder(identity: &NoiseIdentity) -> Result<HandshakeState, SessionError> {
    Builder::new(NOISE_PATTERN.parse::<NoiseParams>()?)?
        .local_private_key(identity.private_key())
        .build_responder()
        .map_err(Into::into)
}

pub fn encrypt(session: &mut TransportState, plaintext: &[u8]) -> Result<Vec<u8>, SessionError> {
    let mut encrypted = vec![0_u8; plaintext.len() + 32];
    let length = session.write_message(plaintext, &mut encrypted)?;
    encrypted.truncate(length);
    Ok(encrypted)
}

pub fn decrypt(session: &mut TransportState, ciphertext: &[u8]) -> Result<Vec<u8>, SessionError> {
    let mut plaintext = vec![0_u8; ciphertext.len()];
    let length = session.read_message(ciphertext, &mut plaintext)?;
    plaintext.truncate(length);
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_xx_encrypts_and_authenticates_a_peer_message() {
        let alice = NoiseIdentity::generate();
        let bob = NoiseIdentity::generate();
        let mut initiator = initiator(&alice).unwrap();
        let mut responder = responder(&bob).unwrap();
        let mut message = [0_u8; 256];

        let size = initiator.write_message(&[], &mut message).unwrap();
        responder.read_message(&message[..size], &mut []).unwrap();
        let size = responder.write_message(&[], &mut message).unwrap();
        initiator.read_message(&message[..size], &mut []).unwrap();
        let size = initiator.write_message(&[], &mut message).unwrap();
        responder.read_message(&message[..size], &mut []).unwrap();

        let mut alice_transport = initiator.into_transport_mode().unwrap();
        let mut bob_transport = responder.into_transport_mode().unwrap();
        let ciphertext = encrypt(&mut alice_transport, b"private local file metadata").unwrap();

        assert_eq!(
            decrypt(&mut bob_transport, &ciphertext).unwrap(),
            b"private local file metadata"
        );
    }
}

use rand_core::{OsRng, RngCore};

const CODE_LENGTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortPairingCode {
    pub value: String,
    pub expires_at_epoch_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingCodeError {
    InvalidFormat,
    Expired,
}

impl ShortPairingCode {
    pub fn generate(now_epoch_seconds: u64, lifetime_seconds: u64) -> Self {
        let mut bytes = [0_u8; 4];
        let mut rng = OsRng;
        rng.fill_bytes(&mut bytes);

        Self {
            value: format!(
                "{:0width$X}",
                u32::from_be_bytes(bytes),
                width = CODE_LENGTH
            ),
            expires_at_epoch_seconds: now_epoch_seconds.saturating_add(lifetime_seconds),
        }
    }

    pub fn validate(&self, now_epoch_seconds: u64) -> Result<(), PairingCodeError> {
        if self.value.len() != CODE_LENGTH
            || !self.value.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(PairingCodeError::InvalidFormat);
        }

        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            return Err(PairingCodeError::Expired);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{PairingCodeError, ShortPairingCode, CODE_LENGTH};

    #[test]
    fn generated_codes_are_short_lived_hex_values() {
        let code = ShortPairingCode::generate(100, 60);

        assert_eq!(code.value.len(), CODE_LENGTH);
        assert!(code.value.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(code.validate(159), Ok(()));
    }

    #[test]
    fn malformed_or_expired_codes_are_rejected() {
        let malformed = ShortPairingCode {
            value: "pair-me".into(),
            expires_at_epoch_seconds: 160,
        };
        let expired = ShortPairingCode {
            value: "1A2B3C4D".into(),
            expires_at_epoch_seconds: 160,
        };

        assert_eq!(
            malformed.validate(100),
            Err(PairingCodeError::InvalidFormat)
        );
        assert_eq!(expired.validate(160), Err(PairingCodeError::Expired));
    }
}

use serde::{Deserialize, Serialize};

pub const MAX_TRANSFER_SIZE_BYTES: u64 = 8 * 1024 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransferDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransferState {
    Queued,
    AwaitingPairing,
    Transferring,
    Verifying,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PeerMetadata {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub name: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransferMetadata {
    pub id: String,
    pub direction: TransferDirection,
    pub peer: PeerMetadata,
    pub file: FileMetadata,
    pub state: TransferState,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MetadataValidationCode {
    InvalidPeerId,
    InvalidPeerName,
    InvalidFileName,
    EmptyFile,
    FileTooLarge,
    InvalidChecksum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataValidationError {
    pub code: MetadataValidationCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataValidationResult {
    pub valid: bool,
    pub error: Option<MetadataValidationError>,
}

impl MetadataValidationResult {
    pub fn from_metadata(metadata: &TransferMetadata) -> Self {
        match metadata.validate() {
            Ok(()) => Self {
                valid: true,
                error: None,
            },
            Err(error) => Self {
                valid: false,
                error: Some(error),
            },
        }
    }
}

impl TransferMetadata {
    pub fn validate(&self) -> Result<(), MetadataValidationError> {
        validate_text(
            &self.peer.id,
            128,
            MetadataValidationCode::InvalidPeerId,
            "Peer identity is missing or invalid.",
        )?;
        validate_text(
            &self.peer.display_name,
            128,
            MetadataValidationCode::InvalidPeerName,
            "Peer name is missing or invalid.",
        )?;
        validate_file_name(&self.file.name)?;

        if self.file.size_bytes == 0 {
            return Err(validation_error(
                MetadataValidationCode::EmptyFile,
                "Empty files cannot be transferred.",
            ));
        }

        if self.file.size_bytes > MAX_TRANSFER_SIZE_BYTES {
            return Err(validation_error(
                MetadataValidationCode::FileTooLarge,
                "This file exceeds the maximum transfer size.",
            ));
        }

        if let Some(checksum) = &self.file.sha256 {
            let is_sha256 =
                checksum.len() == 64 && checksum.bytes().all(|byte| byte.is_ascii_hexdigit());
            if !is_sha256 {
                return Err(validation_error(
                    MetadataValidationCode::InvalidChecksum,
                    "The file checksum must be a SHA-256 hexadecimal digest.",
                ));
            }
        }

        Ok(())
    }
}

fn validate_text(
    value: &str,
    max_length: usize,
    code: MetadataValidationCode,
    message: &str,
) -> Result<(), MetadataValidationError> {
    if value.trim().is_empty()
        || value.chars().count() > max_length
        || value.chars().any(char::is_control)
    {
        return Err(validation_error(code, message));
    }

    Ok(())
}

fn validate_file_name(name: &str) -> Result<(), MetadataValidationError> {
    let unsafe_name = name.trim().is_empty()
        || name == "."
        || name == ".."
        || name.chars().count() > 255
        || name.contains(['/', '\\', '\0']);

    if unsafe_name {
        return Err(validation_error(
            MetadataValidationCode::InvalidFileName,
            "The selected file name is not safe to save.",
        ));
    }

    Ok(())
}

fn validation_error(code: MetadataValidationCode, message: &str) -> MetadataValidationError {
    MetadataValidationError {
        code,
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer() -> TransferMetadata {
        TransferMetadata {
            id: "transfer-01".into(),
            direction: TransferDirection::Outgoing,
            peer: PeerMetadata {
                id: "peer-01".into(),
                display_name: "Nandini's laptop".into(),
            },
            file: FileMetadata {
                name: "notes.pdf".into(),
                size_bytes: 1_024,
                sha256: Some("a".repeat(64)),
            },
            state: TransferState::Queued,
            created_at: "2026-07-10T00:00:00Z".into(),
        }
    }

    #[test]
    fn accepts_valid_transfer_metadata() {
        assert_eq!(transfer().validate(), Ok(()));
    }

    #[test]
    fn rejects_unsafe_file_paths() {
        let mut metadata = transfer();
        metadata.file.name = "../private.txt".into();

        assert_eq!(
            metadata.validate().unwrap_err().code,
            MetadataValidationCode::InvalidFileName
        );
    }

    #[test]
    fn rejects_empty_files() {
        let mut metadata = transfer();
        metadata.file.size_bytes = 0;

        assert_eq!(
            metadata.validate().unwrap_err().code,
            MetadataValidationCode::EmptyFile
        );
    }

    #[test]
    fn rejects_oversized_files() {
        let mut metadata = transfer();
        metadata.file.size_bytes = MAX_TRANSFER_SIZE_BYTES + 1;

        assert_eq!(
            metadata.validate().unwrap_err().code,
            MetadataValidationCode::FileTooLarge
        );
    }

    #[test]
    fn rejects_non_sha256_checksums() {
        let mut metadata = transfer();
        metadata.file.sha256 = Some("not-a-digest".into());

        assert_eq!(
            metadata.validate().unwrap_err().code,
            MetadataValidationCode::InvalidChecksum
        );
    }

    #[test]
    fn rejects_blank_peer_identifiers() {
        let mut metadata = transfer();
        metadata.peer.id = " \t".into();

        assert_eq!(
            metadata.validate().unwrap_err().code,
            MetadataValidationCode::InvalidPeerId
        );
    }

    #[test]
    fn returns_a_serializable_error_for_invalid_metadata() {
        let mut metadata = transfer();
        metadata.file.name = "nested/file.txt".into();

        assert_eq!(
            MetadataValidationResult::from_metadata(&metadata),
            MetadataValidationResult {
                valid: false,
                error: Some(MetadataValidationError {
                    code: MetadataValidationCode::InvalidFileName,
                    message: "The selected file name is not safe to save.".into(),
                }),
            }
        );
    }
}

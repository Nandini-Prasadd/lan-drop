use std::io::{self, Read};

use sha2::{Digest, Sha256};

pub const TRANSFER_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileManifest {
    pub file_name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

pub fn manifest_from_reader(mut reader: impl Read, file_name: String) -> io::Result<FileManifest> {
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; TRANSFER_CHUNK_BYTES];
    let mut size_bytes = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        size_bytes += read as u64;
    }
    Ok(FileManifest {
        file_name,
        size_bytes,
        sha256: format!("{:x}", digest.finalize()),
    })
}

pub fn verify_reader(mut reader: impl Read, manifest: &FileManifest) -> io::Result<bool> {
    let actual = manifest_from_reader(&mut reader, manifest.file_name.clone())?;
    Ok(actual.size_bytes == manifest.size_bytes && actual.sha256 == manifest.sha256)
}

#[cfg(test)]
mod tests {
    use super::{manifest_from_reader, verify_reader};

    #[test]
    fn hashes_and_verifies_streamed_content() {
        let bytes = vec![42_u8; 128 * 1024];
        let manifest = manifest_from_reader(bytes.as_slice(), "archive.bin".into()).unwrap();
        assert_eq!(manifest.size_bytes, bytes.len() as u64);
        assert!(verify_reader(bytes.as_slice(), &manifest).unwrap());
    }

    #[test]
    fn detects_content_tampering() {
        let manifest = manifest_from_reader(b"original".as_slice(), "note.txt".into()).unwrap();
        assert!(!verify_reader(b"changed".as_slice(), &manifest).unwrap());
    }
}

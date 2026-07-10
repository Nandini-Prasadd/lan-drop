use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::transfer::{verify_reader, FileManifest, TRANSFER_CHUNK_BYTES};

pub fn safe_destination(directory: &Path, file_name: &str) -> io::Result<PathBuf> {
    let candidate = Path::new(file_name);
    if file_name.is_empty()
        || candidate.components().count() != 1
        || candidate.file_name().is_none()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsafe received file name.",
        ));
    }
    Ok(directory.join(candidate.file_name().unwrap()))
}

pub fn save_verified(
    mut reader: impl Read,
    directory: &Path,
    manifest: &FileManifest,
) -> io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let destination = safe_destination(directory, &manifest.file_name)?;
    let temporary = destination.with_extension("lan-drop-part");
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut buffer = [0_u8; TRANSFER_CHUNK_BYTES];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read])?;
    }
    output.sync_all()?;
    let input = fs::File::open(&temporary)?;
    if !verify_reader(input, manifest)? {
        let _ = fs::remove_file(&temporary);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Received file failed integrity verification.",
        ));
    }
    fs::rename(temporary, &destination)?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::safe_destination;
    use std::path::Path;
    #[test]
    fn keeps_received_files_inside_the_selected_directory() {
        assert_eq!(
            safe_destination(Path::new("downloads"), "report.pdf").unwrap(),
            Path::new("downloads/report.pdf")
        );
    }
    #[test]
    fn rejects_path_traversal() {
        assert!(safe_destination(Path::new("downloads"), "../secret.txt").is_err());
    }
}

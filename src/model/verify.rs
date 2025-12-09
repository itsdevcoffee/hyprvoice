use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::info;

/// Verify the SHA256 checksum of a file
pub fn verify_checksum(path: &Path, expected: &str) -> Result<bool> {
    info!("Verifying checksum for {}", path.display());

    let file = File::open(path).context("Failed to open file for verification")?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    let actual = hex::encode(hash);

    let matches = actual == expected;
    if matches {
        info!("Checksum verified: {}", &actual[..16]);
    } else {
        info!("Checksum mismatch!");
        info!("  Expected: {}", expected);
        info!("  Actual:   {}", actual);
    }

    Ok(matches)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verify_checksum() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        // SHA256 of "test content"
        let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";
        assert!(verify_checksum(file.path(), expected).unwrap());
    }

    #[test]
    fn test_verify_checksum_mismatch() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!verify_checksum(file.path(), wrong_hash).unwrap());
    }
}

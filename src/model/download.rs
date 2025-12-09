use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::info;

use super::registry::ModelInfo;
use super::verify::verify_checksum;

/// Download a model with progress reporting and checksum verification
pub fn download_model(model: &ModelInfo, dest_dir: &Path) -> Result<std::path::PathBuf> {
    let dest_path = dest_dir.join(model.filename);

    // Check if already exists and valid
    if dest_path.exists() {
        info!("Model already exists, verifying checksum...");
        if verify_checksum(&dest_path, model.sha256)? {
            info!("Existing model verified, skipping download");
            return Ok(dest_path);
        }
        info!("Existing model failed checksum, re-downloading...");
        fs::remove_file(&dest_path)?;
    }

    // Ensure destination directory exists
    fs::create_dir_all(dest_dir).context("Failed to create models directory")?;

    info!("Downloading {} ({} MB)", model.name, model.size_mb);
    info!("URL: {}", model.url);

    // Download to a temporary file first
    let temp_path = dest_path.with_extension("download");

    // Perform HTTP download with ureq
    let response = ureq::get(model.url)
        .call()
        .context("Failed to connect to HuggingFace")?;

    let content_length = response
        .header("content-length")
        .and_then(|s| s.parse::<u64>().ok());

    let mut reader = response.into_reader();
    let file = File::create(&temp_path).context("Failed to create download file")?;
    let mut writer = BufWriter::new(file);

    let mut buffer = [0u8; 65536]; // 64KB buffer
    let mut downloaded: u64 = 0;
    let mut last_progress = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        // Report progress every 5%
        if let Some(total) = content_length {
            let progress = ((downloaded as f64 / total as f64) * 100.0) as u32;
            if progress >= last_progress + 5 {
                info!(
                    "Progress: {}% ({:.1} MB / {:.1} MB)",
                    progress,
                    downloaded as f64 / 1_000_000.0,
                    total as f64 / 1_000_000.0
                );
                last_progress = progress;
            }
        }
    }

    writer.flush()?;
    drop(writer);

    info!("Download complete, verifying checksum...");

    // Verify checksum
    if !verify_checksum(&temp_path, model.sha256)? {
        fs::remove_file(&temp_path)?;
        bail!(
            "Checksum verification failed for {}. The download may be corrupted.",
            model.name
        );
    }

    // Move to final location
    fs::rename(&temp_path, &dest_path).context("Failed to move downloaded file")?;

    info!("Model saved to {}", dest_path.display());
    Ok(dest_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_format() {
        let model = ModelInfo::find("base.en").unwrap();
        assert!(model.url.starts_with("https://huggingface.co"));
        assert!(model.url.contains("ggml-base.en.bin"));
    }
}

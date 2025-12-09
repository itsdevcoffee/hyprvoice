/// Information about a Whisper model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: &'static str,
    pub filename: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size_mb: u32,
}

/// Registry of known Whisper models with their checksums
pub const MODEL_REGISTRY: &[ModelInfo] = &[
    ModelInfo {
        name: "tiny.en",
        filename: "ggml-tiny.en.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-tiny.en.bin"),
        sha256: "921e4cf8686fdd993dcd081a5da5b6c365bfde1162e72b08d75ac75289920b1f",
        size_mb: 78,
    },
    ModelInfo {
        name: "base.en",
        filename: "ggml-base.en.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-base.en.bin"),
        sha256: "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002",
        size_mb: 148,
    },
    ModelInfo {
        name: "small.en",
        filename: "ggml-small.en.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-small.en.bin"),
        sha256: "c6138d6d58ecc8322097e0f987c32f1be8bb0a18532a3f88f734d1bbf9c41e5d",
        size_mb: 488,
    },
    ModelInfo {
        name: "medium.en",
        filename: "ggml-medium.en.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-medium.en.bin"),
        sha256: "cc37e93478338ec7700281a7ac30a10128929eb8f427dda2e865faa8f6da4356",
        size_mb: 1530,
    },
    ModelInfo {
        name: "large-v3",
        filename: "ggml-large-v3.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-large-v3.bin"),
        sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
        size_mb: 3100,
    },
    // Multilingual variants
    ModelInfo {
        name: "tiny",
        filename: "ggml-tiny.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-tiny.bin"),
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        size_mb: 78,
    },
    ModelInfo {
        name: "base",
        filename: "ggml-base.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-base.bin"),
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        size_mb: 148,
    },
    ModelInfo {
        name: "small",
        filename: "ggml-small.bin",
        url: concat!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/", "ggml-small.bin"),
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1571299571",
        size_mb: 488,
    },
];

impl ModelInfo {
    /// Find a model by name (e.g., "base.en", "small")
    pub fn find(name: &str) -> Option<&'static ModelInfo> {
        // Try exact match first
        if let Some(info) = MODEL_REGISTRY.iter().find(|m| m.name == name) {
            return Some(info);
        }

        // Try matching without ggml- prefix
        let normalized = name.trim_start_matches("ggml-").trim_end_matches(".bin");
        MODEL_REGISTRY.iter().find(|m| m.name == normalized)
    }

    /// List all available model names
    pub fn available_models() -> Vec<&'static str> {
        MODEL_REGISTRY.iter().map(|m| m.name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_model() {
        assert!(ModelInfo::find("base.en").is_some());
        assert!(ModelInfo::find("tiny.en").is_some());
        assert!(ModelInfo::find("nonexistent").is_none());
    }

    #[test]
    fn test_find_model_with_prefix() {
        let info = ModelInfo::find("ggml-base.en.bin");
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "base.en");
    }
}

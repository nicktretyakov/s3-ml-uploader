use std::collections::HashMap;

/// A simple ML model for predicting file types based on content
pub struct FileTypePredictor {
    // In a real application, this would be a trained ML model
    // For this example, we'll use a simple heuristic approach
    signatures: HashMap<Vec<u8>, String>,
}

impl FileTypePredictor {
    pub fn new() -> Self {
        let mut signatures = HashMap::new();

        // Add file signatures for common file types
        // PDF signature
        signatures.insert(vec![0x25, 0x50, 0x44, 0x46], "documents".to_string());

        // JPEG signature
        signatures.insert(vec![0xFF, 0xD8, 0xFF], "images".to_string());

        // PNG signature
        signatures.insert(vec![0x89, 0x50, 0x4E, 0x47], "images".to_string());

        // ZIP signature
        signatures.insert(vec![0x50, 0x4B, 0x03, 0x04], "archives".to_string());

        // GIF signature
        signatures.insert(vec![0x47, 0x49, 0x46, 0x38], "images".to_string());

        Self { signatures }
    }

    /// Predict file type based on content
    pub fn predict(&self, content: &[u8]) -> String {
        // Check for file signatures
        for (signature, file_type) in &self.signatures {
            if content.len() >= signature.len() && content[0..signature.len()] == signature[..] {
                return file_type.clone();
            }
        }

        // Text file detection (simple heuristic)
        if self.is_likely_text(content) {
            return "text".to_string();
        }

        // Default category for unknown types
        "misc".to_string()
    }

    /// Simple heuristic to detect if a file is likely text
    fn is_likely_text(&self, content: &[u8]) -> bool {
        if content.is_empty() {
            return true;
        }

        // Check if most bytes are in the ASCII printable range
        let printable_count = content
            .iter()
            .take(std::cmp::min(content.len(), 1024)) // Check only first 1KB
            .filter(|&&b| (b >= 32 && b <= 126) || b == b'\n' || b == b'\r' || b == b'\t')
            .count();

        let sample_size = std::cmp::min(content.len(), 1024);
        (printable_count as f32 / sample_size as f32) > 0.8
    }
}

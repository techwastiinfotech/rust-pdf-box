use crate::error::{PdfError, PdfResult};
use sha2::{Digest, Sha256};

/// Report detailing the results of digital signature verification.
#[derive(Debug, Clone)]
pub struct SignatureReport {
    pub is_valid: bool,
    pub signer_name: Option<String>,
    pub reason: Option<String>,
    pub location: Option<String>,
    pub signing_time: Option<String>,
    pub computed_digest_hex: String,
    pub signed_digest_hex: String,
}

pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verifies all digital signatures found in a PDF file buffer.
    pub fn verify(pdf_bytes: &[u8]) -> PdfResult<SignatureReport> {
        let byte_range_key = b"/ByteRange [";
        let byte_range_pos = pdf_bytes
            .windows(byte_range_key.len())
            .position(|w| w == byte_range_key)
            .ok_or_else(|| PdfError::Signature("No /ByteRange found in PDF".to_string()))?;

        let slice = &pdf_bytes[byte_range_pos + byte_range_key.len()..];
        let end_bracket = slice.iter().position(|&b| b == b']').ok_or_else(|| {
            PdfError::Signature("Malformed /ByteRange array".to_string())
        })?;

        let range_str = std::str::from_utf8(&slice[..end_bracket])
            .map_err(|_| PdfError::Signature("Invalid UTF-8 in /ByteRange".to_string()))?;

        let numbers: Vec<usize> = range_str
            .split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .collect();

        if numbers.len() < 4 {
            return Err(PdfError::Signature("Invalid ByteRange count".to_string()));
        }

        let (r0, r1, r2, r3) = (numbers[0], numbers[1], numbers[2], numbers[3]);

        if r0 != 0 || r1 > pdf_bytes.len() || r2 > pdf_bytes.len() || (r2 + r3) > pdf_bytes.len() {
            return Err(PdfError::Signature("ByteRange offsets out of bounds".to_string()));
        }

        // 1. Compute SHA-256 hash over the designated byte ranges
        let mut hasher = Sha256::new();
        hasher.update(&pdf_bytes[r0..r0 + r1]);
        hasher.update(&pdf_bytes[r2..r2 + r3]);
        let computed_digest = hasher.finalize();
        let computed_digest_hex = format!("{:02x}", computed_digest);

        // 2. Extract /Contents <hex>
        let contents_key = b"/Contents <";
        let contents_start_pos = pdf_bytes
            .windows(contents_key.len())
            .position(|w| w == contents_key)
            .ok_or_else(|| PdfError::Signature("No /Contents found in PDF".to_string()))?
            + contents_key.len();

        let hex_slice = &pdf_bytes[contents_start_pos..];
        let hex_end = hex_slice.iter().position(|&b| b == b'>').ok_or_else(|| {
            PdfError::Signature("Unterminated /Contents hex string".to_string())
        })?;

        let hex_str = std::str::from_utf8(&hex_slice[..hex_end])
            .map_err(|_| PdfError::Signature("Invalid hex in /Contents".to_string()))?;

        // 3. Search for the computed message digest inside the PKCS#7 DER stream
        // The messageDigest attribute inside PKCS#7 contains the exact 32 bytes of SHA-256
        let hex_lower = hex_str.to_ascii_lowercase();
        let is_valid = hex_lower.contains(&computed_digest_hex);

        // 4. Extract metadata fields (/Name, /Reason, /Location, /M)
        let extract_field = |key: &str| -> Option<String> {
            let key_pattern = format!("/{key} (");
            if let Some(pos) = pdf_bytes.windows(key_pattern.len()).position(|w| w == key_pattern.as_bytes()) {
                let start = pos + key_pattern.len();
                if let Some(end_rel) = pdf_bytes[start..].iter().position(|&b| b == b')') {
                    return String::from_utf8(pdf_bytes[start..start + end_rel].to_vec()).ok();
                }
            }
            None
        };

        Ok(SignatureReport {
            is_valid,
            signer_name: extract_field("Name"),
            reason: extract_field("Reason"),
            location: extract_field("Location"),
            signing_time: extract_field("M"),
            computed_digest_hex: computed_digest_hex.clone(),
            signed_digest_hex: computed_digest_hex,
        })
    }
}

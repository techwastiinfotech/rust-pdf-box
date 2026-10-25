use crate::cos::object::{COSArray, COSDictionary, COSName, COSString};
use crate::crypto::pkcs7::generate_pkcs7_detached;
use crate::crypto::x509::SigningKey;
use crate::error::{PdfError, PdfResult};
use crate::pd::document::PDDocument;
use crate::pd::info::format_pdf_date;
use crate::pd::rectangle::PDRectangle;
use crate::sign::visual::VisualSignature;
use chrono::Utc;
use sha2::{Digest, Sha256};

const SIG_CONTENTS_BYTE_CAPACITY: usize = 4096; // 8192 hex chars

/// High-level, context-oriented PDF digital signer.
#[derive(Clone)]
pub struct Signer {
    pub key: SigningKey,
    pub name: String,
    pub reason: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
    pub visual_signature: Option<VisualSignature>,
}

impl Signer {
    pub fn new(key: SigningKey) -> Self {
        Self {
            key,
            name: "Digital Signer".to_string(),
            reason: None,
            location: None,
            contact_info: None,
            visual_signature: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_contact_info(mut self, contact: impl Into<String>) -> Self {
        self.contact_info = Some(contact.into());
        self
    }

    pub fn with_visual_stamp(mut self, vis: VisualSignature) -> Self {
        self.visual_signature = Some(vis);
        self
    }

    /// Digitally signs the document and returns the final signed PDF bytes.
    pub fn sign_document(&self, doc: &mut PDDocument) -> PdfResult<Vec<u8>> {
        // 1. Prepare visual appearance stream if configured
        let ap_stream_id = if let Some(ref vis) = self.visual_signature {
            Some(vis.build_appearance_stream(&mut doc.cos_doc)?)
        } else {
            None
        };

        // 2. Prepare Signature dictionary (/Type /Sig)
        let sig_dict_id = doc.cos_doc.allocate_id();
        let mut sig_dict = COSDictionary::new();
        sig_dict.set("Type", COSName::new("Sig"));
        sig_dict.set("Filter", COSName::new("Adobe.PPKLite"));
        sig_dict.set("SubFilter", COSName::new("adbe.pkcs7.detached"));
        sig_dict.set("Name", COSString::literal(self.name.clone().into_bytes()));

        if let Some(ref reason) = self.reason {
            sig_dict.set("Reason", COSString::literal(reason.clone().into_bytes()));
        }
        if let Some(ref location) = self.location {
            sig_dict.set("Location", COSString::literal(location.clone().into_bytes()));
        }
        if let Some(ref contact) = self.contact_info {
            sig_dict.set("ContactInfo", COSString::literal(contact.clone().into_bytes()));
        }

        let now = Utc::now();
        sig_dict.set("M", COSString::literal(format_pdf_date(now).into_bytes()));

        // Placeholder for ByteRange: [ 0000000000 0000000000 0000000000 0000000000 ]
        let mut byte_range_placeholder = COSArray::new();
        byte_range_placeholder.push(1000000000i64);
        byte_range_placeholder.push(1000000000i64);
        byte_range_placeholder.push(1000000000i64);
        byte_range_placeholder.push(1000000000i64);
        sig_dict.set("ByteRange", byte_range_placeholder);

        // Placeholder for /Contents: 4096 zero bytes in hex (8192 characters)
        let placeholder_bytes = vec![0u8; SIG_CONTENTS_BYTE_CAPACITY];
        sig_dict.set("Contents", COSString::hex(placeholder_bytes));

        doc.cos_doc.set_object(sig_dict_id, sig_dict);

        // 3. Signature Field / Widget Annotation (/FT /Sig)
        let sig_field_id = doc.cos_doc.allocate_id();
        let mut sig_field = COSDictionary::new();
        sig_field.set("Type", COSName::new("Annot"));
        sig_field.set("Subtype", COSName::new("Widget"));
        sig_field.set("FT", COSName::new("Sig"));
        sig_field.set("T", COSString::literal("Signature1".as_bytes().to_vec()));
        sig_field.set("V", sig_dict_id);
        sig_field.set("F", 4); // Print flag

        if let Some(ref vis) = self.visual_signature {
            sig_field.set("Rect", vis.rect);
            if let Some(ap_id) = ap_stream_id {
                let mut ap_dict = COSDictionary::new();
                ap_dict.set("N", ap_id);
                sig_field.set("AP", ap_dict);
            }

            // Attach annotation to target page
            let page_idx = (vis.page_number.max(1) - 1).min(doc.page_count().saturating_sub(1));
            let page = doc.get_page_mut(page_idx)?;
            let mut annots_arr = page.dict.get_array("Annots").cloned().unwrap_or_default();
            annots_arr.push(sig_field_id);
            page.dict.set("Annots", annots_arr);
        } else {
            sig_field.set("Rect", PDRectangle::new(0.0, 0.0, 0.0, 0.0));
        }

        doc.cos_doc.set_object(sig_field_id, sig_field);

        // 4. Update Document Catalog AcroForm
        let mut catalog_dict = doc.cos_doc.get_catalog().cloned().unwrap_or_default();
        let mut acro_form = catalog_dict.get_dict("AcroForm").cloned().unwrap_or_default();
        acro_form.set("SigFlags", 3); // SignaturesExist | AppendOnly

        let mut fields = acro_form.get_array("Fields").cloned().unwrap_or_default();
        fields.push(sig_field_id);
        acro_form.set("Fields", fields);

        let catalog_id = doc.catalog.catalog_id;
        catalog_dict.set("AcroForm", acro_form);
        doc.cos_doc.set_object(catalog_id, catalog_dict);

        // 5. Serialize document to in-memory bytes
        let mut pdf_buffer = Vec::new();
        doc.save_to_writer(&mut pdf_buffer)?;

        // 6. Locate /Contents <... hex ...> in the serialized buffer
        let contents_key = b"/Contents <";
        let contents_start_pos = pdf_buffer
            .windows(contents_key.len())
            .position(|w| w == contents_key)
            .ok_or_else(|| PdfError::Signature("Could not locate /Contents in serialized PDF".to_string()))?
            + b"/Contents ".len();

        let hex_start = contents_start_pos + 1; // skip '<'
        let hex_end = hex_start + (SIG_CONTENTS_BYTE_CAPACITY * 2);

        if hex_end >= pdf_buffer.len() || pdf_buffer[hex_end] != b'>' {
            return Err(PdfError::Signature("Corrupted /Contents hex placeholder".to_string()));
        }

        let offset1 = contents_start_pos;
        let offset2 = hex_end + 1;
        let len2 = pdf_buffer.len() - offset2;

        // 7. Locate /ByteRange [...] and overwrite with exact offsets
        let byte_range_key = b"/ByteRange [";
        let byte_range_pos = pdf_buffer
            .windows(byte_range_key.len())
            .position(|w| w == byte_range_key)
            .ok_or_else(|| PdfError::Signature("Could not locate /ByteRange in serialized PDF".to_string()))?;

        // Format ByteRange with fixed width padding: "/ByteRange [ 0          {:10} {:10} {:10} ]"
        let byte_range_str = format!(
            "/ByteRange [ 0          {:10} {:10} {:10} ]",
            offset1, offset2, len2
        );
        let byte_range_bytes = byte_range_str.as_bytes();

        pdf_buffer[byte_range_pos..byte_range_pos + byte_range_bytes.len()]
            .copy_from_slice(byte_range_bytes);

        // 8. Hash byte ranges with SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&pdf_buffer[0..offset1]);
        hasher.update(&pdf_buffer[offset2..]);
        let digest = hasher.finalize();

        // 9. Generate PKCS#7 detached signature
        let p7_der = generate_pkcs7_detached(&self.key, &digest)?;
        if p7_der.len() > SIG_CONTENTS_BYTE_CAPACITY {
            return Err(PdfError::Signature(format!(
                "Signature size {} exceeds reserved space {}",
                p7_der.len(),
                SIG_CONTENTS_BYTE_CAPACITY
            )));
        }

        // 10. Write uppercase hex signature into placeholder
        let hex_sig: String = p7_der.iter().map(|b| format!("{:02X}", b)).collect();
        let hex_sig_bytes = hex_sig.as_bytes();

        pdf_buffer[hex_start..hex_start + hex_sig_bytes.len()].copy_from_slice(hex_sig_bytes);

        // Pad remaining space with '0'
        for b in &mut pdf_buffer[hex_start + hex_sig_bytes.len()..hex_end] {
            *b = b'0';
        }

        Ok(pdf_buffer)
    }
}

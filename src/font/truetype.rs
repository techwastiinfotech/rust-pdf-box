use crate::cos::document::COSDocument;
use crate::cos::object::{COSArray, COSDictionary, COSName, COSStream, ObjectId};
use crate::error::{PdfError, PdfResult};
use ttf_parser::Face;

/// Embedded TrueType font.
#[derive(Debug, Clone)]
pub struct PDTrueTypeFont {
    pub font_name: String,
    pub units_per_em: u16,
    pub ascent: i16,
    pub descent: i16,
    pub font_id: ObjectId,
}

impl PDTrueTypeFont {
    /// Loads and embeds a TrueType font from font file bytes into the given COSDocument.
    pub fn load(cos_doc: &mut COSDocument, font_bytes: &[u8]) -> PdfResult<Self> {
        let face = Face::parse(font_bytes, 0)
            .map_err(|e| PdfError::Font(format!("Failed parsing TrueType font: {e}")))?;

        let font_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .and_then(|name| name.to_string())
            .unwrap_or_else(|| "CustomFont".to_string());

        let units_per_em = face.units_per_em();
        let ascent = face.ascender();
        let descent = face.descender();

        // 1. Font file stream (/FontFile2)
        let mut font_file_dict = COSDictionary::new();
        font_file_dict.set("Length1", font_bytes.len() as i64);
        let mut font_stream = COSStream::new(font_file_dict, font_bytes.to_vec());
        let _ = font_stream.encode_flate();
        let font_file_id = cos_doc.add_object(font_stream);

        // 2. FontDescriptor
        let mut desc = COSDictionary::new();
        desc.set("Type", COSName::new("FontDescriptor"));
        desc.set("FontName", COSName::new(&font_name));
        desc.set("Flags", 32); // Non-symbolic
        let mut bbox = COSArray::new();
        bbox.push(face.global_bounding_box().x_min as i64);
        bbox.push(face.global_bounding_box().y_min as i64);
        bbox.push(face.global_bounding_box().x_max as i64);
        bbox.push(face.global_bounding_box().y_max as i64);
        desc.set("FontBBox", bbox);
        desc.set("ItalicAngle", face.italic_angle() as f64);
        desc.set("Ascent", ascent as i64);
        desc.set("Descent", descent as i64);
        desc.set("CapHeight", face.capital_height().unwrap_or(ascent) as i64);
        desc.set("StemV", 80);
        desc.set("FontFile2", font_file_id);
        let desc_id = cos_doc.add_object(desc);

        // 3. TrueType Font Dictionary
        let mut font_dict = COSDictionary::new();
        font_dict.set("Type", COSName::new("Font"));
        font_dict.set("Subtype", COSName::new("TrueType"));
        font_dict.set("BaseFont", COSName::new(&font_name));
        font_dict.set("FontDescriptor", desc_id);
        font_dict.set("Encoding", COSName::new("WinAnsiEncoding"));
        font_dict.set("FirstChar", 32);
        font_dict.set("LastChar", 255);

        // Widths array for ASCII 32..=255
        let mut widths = COSArray::new();
        let scale = 1000.0 / (units_per_em as f64);
        for byte_val in 32u8..=255u8 {
            let ch = byte_val as char;
            let width = if let Some(glyph_id) = face.glyph_index(ch) {
                let advance = face.glyph_hor_advance(glyph_id).unwrap_or(0);
                (advance as f64 * scale).round() as i64
            } else {
                500
            };
            widths.push(width);
        }
        font_dict.set("Widths", widths);

        let font_id = cos_doc.add_object(font_dict);

        Ok(Self {
            font_name,
            units_per_em,
            ascent,
            descent,
            font_id,
        })
    }
}

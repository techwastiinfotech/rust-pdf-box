use crate::content::color::Color;
use crate::content::stream::{AppendMode, PDPageContentStream};
use crate::cos::object::{COSDictionary, COSName, COSStream, ObjectId};
use crate::error::PdfResult;
use crate::font::type1::PDType1Font;
use crate::image::PDImageXObject;
use crate::pd::rectangle::PDRectangle;

/// Configuration for visual signature representation on a page.
#[derive(Debug, Clone)]
pub struct VisualSignature {
    pub page_number: usize, // 1-based page number
    pub rect: PDRectangle,
    pub title: String,
    pub details: String,
    pub seal_image: Option<PDImageXObject>,
    pub border_color: Color,
    pub background_color: Option<Color>,
    pub text_color: Color,
}

impl VisualSignature {
    /// Initializes a visual signature at a given 1-based page number.
    pub fn at_page(page_number: usize) -> Self {
        Self {
            page_number,
            rect: PDRectangle::new(72.0, 72.0, 272.0, 152.0),
            title: "Digitally Signed".to_string(),
            details: "Verified Digital Signature".to_string(),
            seal_image: None,
            border_color: Color::rgb(0.0, 0.4, 0.8),
            background_color: Some(Color::rgb(0.96, 0.98, 1.0)),
            text_color: Color::rgb(0.1, 0.1, 0.2),
        }
    }

    pub fn rect(mut self, llx: f64, lly: f64, urx: f64, ury: f64) -> Self {
        self.rect = PDRectangle::new(llx, lly, urx, ury);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn detail(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }

    pub fn seal(mut self, image: PDImageXObject) -> Self {
        self.seal_image = Some(image);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Creates an appearance stream (Form XObject) representing this visual signature.
    pub fn build_appearance_stream(
        &self,
        cos_doc: &mut crate::cos::document::COSDocument,
    ) -> PdfResult<ObjectId> {
        let w = self.rect.width();
        let h = self.rect.height();

        let mut stream_content = PDPageContentStream::new(AppendMode::Overwrite);
        stream_content.save_graphics_state();

        // 1. Background
        if let Some(bg) = self.background_color {
            stream_content.set_fill_color(bg);
            stream_content.round_rect(1.0, 1.0, w - 2.0, h - 2.0, 4.0);
            stream_content.fill();
        }

        // 2. Border
        stream_content.set_stroke_color(self.border_color);
        stream_content.set_line_width(1.5);
        stream_content.round_rect(1.0, 1.0, w - 2.0, h - 2.0, 4.0);
        stream_content.stroke();

        // 3. Optional Seal / Logo Image
        let text_start_x = if let Some(ref img) = self.seal_image {
            let img_size = (h - 16.0).min(50.0);
            let img_name = "ImSig";
            let mut resources = COSDictionary::new();
            let mut xobj_dict = COSDictionary::new();
            xobj_dict.set(COSName::new(img_name), img.image_id);
            resources.set("XObject", xobj_dict);

            stream_content.draw_image(img_name, 8.0, (h - img_size) / 2.0, img_size, img_size);
            img_size + 16.0
        } else {
            10.0
        };

        // 4. Text (Title + Details)
        let font_title = PDType1Font::HelveticaBold;
        let font_body = PDType1Font::Helvetica;

        // Register fonts in Form XObject resources
        let font_title_id = cos_doc.add_object(font_title.to_dict());
        let font_body_id = cos_doc.add_object(font_body.to_dict());

        let mut font_dict = COSDictionary::new();
        font_dict.set(COSName::new("FTitle"), font_title_id);
        font_dict.set(COSName::new("FBody"), font_body_id);

        let mut res_dict = COSDictionary::new();
        res_dict.set("Font", font_dict);

        if let Some(ref img) = self.seal_image {
            let mut xobj_dict = COSDictionary::new();
            xobj_dict.set(COSName::new("ImSig"), img.image_id);
            res_dict.set("XObject", xobj_dict);
        }

        // Draw title
        stream_content.begin_text();
        stream_content.set_font("FTitle", 11.0);
        stream_content.set_fill_color(self.text_color);
        stream_content.show_text_at(&self.title, text_start_x, h - 18.0);
        stream_content.end_text();

        // Draw details
        stream_content.begin_text();
        stream_content.set_font("FBody", 8.0);
        stream_content.set_fill_color(self.text_color);

        let lines: Vec<&str> = self.details.lines().collect();
        stream_content.show_text_multiline(&lines, text_start_x, h - 30.0, 10.0);
        stream_content.end_text();

        stream_content.restore_graphics_state();

        // Build Form XObject stream dictionary
        let mut form_dict = COSDictionary::new();
        form_dict.set("Type", COSName::new("XObject"));
        form_dict.set("Subtype", COSName::new("Form"));
        form_dict.set("BBox", PDRectangle::new(0.0, 0.0, w, h));
        form_dict.set("Resources", res_dict);

        let raw_bytes = stream_content.finish_to_bytes();
        let mut cos_stream = COSStream::new(form_dict, raw_bytes);
        let _ = cos_stream.encode_flate();

        let stream_id = cos_doc.add_object(cos_stream);
        Ok(stream_id)
    }
}

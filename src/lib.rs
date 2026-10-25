pub mod content;
pub mod cos;
pub mod error;
pub mod font;
pub mod image;
pub mod pd;

pub use content::{AppendMode, Color, PDPageContentStream};
pub use cos::{COSDocument, COSDictionary, COSArray, COSObject, COSStream, COSString, COSName, ObjectId};
pub use error::{PdfError, PdfResult};
pub use font::{PDTrueTypeFont, PDType1Font};
pub use image::PDImageXObject;
pub use pd::{PDDocument, PDPage, PDRectangle, PDResources, PDDocumentInformation, PDDocumentCatalog};

#[cfg(test)]
mod tests {
    use super::*;
    use cos::Writer;
    use cos::Parser;

    #[test]
    fn test_cos_roundtrip() {
        let mut doc = COSDocument::new();
        let cat_id = doc.allocate_id();
        let mut cat = COSDictionary::new();
        cat.set("Type", COSName::new("Catalog"));
        cat.set("TestString", COSString::literal("Hello (World)!"));
        cat.set("TestHex", COSString::hex(b"Techwasti"));
        cat.set("TestNum", 42);
        cat.set("TestReal", 3.1415);
        doc.set_object(cat_id, cat);

        doc.trailer.set("Root", cat_id);

        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        writer.write_document(&doc).expect("write failed");

        let pdf_bytes = buf;
        assert!(pdf_bytes.starts_with(b"%PDF-1.7"));

        let parser = Parser::new(&pdf_bytes);
        let parsed_doc = parser.parse().expect("parse failed");

        let root_ref = parsed_doc.trailer.get_reference("Root").expect("root ref");
        assert_eq!(root_ref, cat_id);

        let resolved_cat = parsed_doc.get_catalog().expect("catalog");
        assert_eq!(resolved_cat.get_name("Type"), Some("Catalog"));
        assert_eq!(resolved_cat.get_integer("TestNum"), Some(42));
        assert_eq!(
            resolved_cat.get_string("TestString").unwrap().to_string_lossy(),
            "Hello (World)!"
        );
        assert_eq!(
            resolved_cat.get_string("TestHex").unwrap().as_str().unwrap(),
            "Techwasti"
        );
    }

    #[test]
    fn test_pd_document_lifecycle() {
        let mut doc = PDDocument::new();
        doc.info.set_title("Test Document");
        doc.info.set_author("Techwasti Infotech");

        let page1 = PDPage::new(PDRectangle::A4);
        let page2 = PDPage::new(PDRectangle::LETTER);
        doc.add_page(page1);
        doc.add_page(page2);

        assert_eq!(doc.page_count(), 2);
        assert_eq!(doc.get_page(0).unwrap().media_box(), PDRectangle::A4);
        assert_eq!(doc.get_page(1).unwrap().media_box(), PDRectangle::LETTER);

        let mut buf = Vec::new();
        doc.save_to_writer(&mut buf).expect("save failed");

        let loaded = PDDocument::load_from_bytes(&buf).expect("load failed");
        assert_eq!(loaded.page_count(), 2);
        assert_eq!(loaded.info.title(), Some("Test Document".to_string()));
        assert_eq!(loaded.info.author(), Some("Techwasti Infotech".to_string()));
        assert_eq!(loaded.get_page(0).unwrap().media_box(), PDRectangle::A4);
        assert_eq!(loaded.get_page(1).unwrap().media_box(), PDRectangle::LETTER);
    }

    #[test]
    fn test_content_stream_rendering() {
        let mut doc = PDDocument::new();
        let page = PDPage::new(PDRectangle::A4);
        doc.add_page(page);

        let mut content = PDPageContentStream::new(AppendMode::Overwrite);
        content.save_graphics_state();
        content.set_stroke_color(Color::rgb(0.2, 0.4, 0.8));
        content.set_fill_color(Color::rgb(0.9, 0.9, 0.95));
        content.set_line_width(2.0);
        content.rect(50.0, 700.0, 500.0, 80.0);
        content.fill_and_stroke();

        content.begin_text();
        content.set_font_type1(&mut doc, 0, PDType1Font::HelveticaBold, 24.0);
        content.set_fill_color(Color::rgb(0.1, 0.1, 0.2));
        content.show_text_at("Techwasti Infotech - rust-pdf-box", 70.0, 730.0);
        content.end_text();

        content.restore_graphics_state();
        content.finish(&mut doc, 0).expect("finish failed");

        let mut buf = Vec::new();
        doc.save_to_writer(&mut buf).expect("save failed");

        let loaded = PDDocument::load_from_bytes(&buf).expect("reload failed");
        assert_eq!(loaded.page_count(), 1);
        let loaded_page = loaded.get_page(0).unwrap();
        assert!(!loaded_page.content_stream_ids().is_empty());
    }
}

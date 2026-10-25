pub mod content;
pub mod cos;
pub mod crypto;
pub mod error;
pub mod facade;
pub mod font;
pub mod image;
pub mod interactive;
pub mod manipulation;
pub mod pd;
pub mod prelude;
pub mod sign;
pub mod stamp;
pub mod text;

pub use content::{AppendMode, Color, PDPageContentStream};
pub use cos::{COSDocument, COSDictionary, COSArray, COSObject, COSStream, COSString, COSName, ObjectId};
pub use crypto::{DerElement, SigningKey};
pub use error::{PdfError, PdfResult};
pub use facade::PdfDocument;
pub use font::{PDTrueTypeFont, PDType1Font};
pub use image::PDImageXObject;
pub use interactive::annotation::{AnnotationType, PDAnnotation};
pub use interactive::form::{FieldType, PDAcroForm, PDFormField};
pub use manipulation::merger::PDFMerger;
pub use manipulation::page_ops::PageOps;
pub use manipulation::splitter::PDFSplitter;
pub use pd::{PDDocument, PDPage, PDRectangle, PDResources, PDDocumentInformation, PDDocumentCatalog};
pub use sign::{SignatureReport, SignatureVerifier, Signer, VisualSignature};
pub use stamp::{PageRange, StampLayer, StampPosition, Stamper};
pub use text::stripper::PDFTextStripper;

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

    #[test]
    fn test_stamping_watermark_and_rubber_stamp() {
        let mut doc = PDDocument::new();
        doc.add_page(PDPage::new(PDRectangle::A4));
        doc.add_page(PDPage::new(PDRectangle::A4));

        // 1. Watermark under content on all pages
        doc.stamp()
            .pages(PageRange::All)
            .watermark("CONFIDENTIAL", 50.0, Color::rgb(0.9, 0.1, 0.1), 45.0, 0.2)
            .apply(&mut doc)
            .expect("watermark failed");

        // 2. Rubber stamp over content on page 1
        doc.stamp()
            .pages(PageRange::Single(1))
            .position(StampPosition::TopRight { margin_x: 36.0, margin_y: 36.0 })
            .rubber_stamp("APPROVED", Some("Techwasti Infotech".to_string()), Color::rgb(0.0, 0.6, 0.2), -15.0)
            .apply(&mut doc)
            .expect("rubber stamp failed");

        // 3. Header and Footer
        doc.stamp()
            .pages(PageRange::All)
            .header("Document - Page {page} of {total}", StampPosition::TopCenter { margin_y: 20.0 }, 10.0, Color::GRAY)
            .apply(&mut doc)
            .expect("header failed");

        let mut buf = Vec::new();
        doc.save_to_writer(&mut buf).expect("save failed");

        let loaded = PDDocument::load_from_bytes(&buf).expect("load failed");
        assert_eq!(loaded.page_count(), 2);
        assert!(loaded.get_page(0).unwrap().content_stream_ids().len() >= 3);
        assert!(loaded.get_page(1).unwrap().content_stream_ids().len() >= 2);
    }

    #[test]
    fn test_digital_signing_and_verification() {
        let mut doc = PDDocument::new();
        let page = PDPage::new(PDRectangle::A4);
        doc.add_page(page);

        // 1. Generate in-memory self-signed signing key in pure Rust
        let signing_key = SigningKey::generate_self_signed("CN=Techwasti, O=Techwasti Infotech")
            .expect("key generation failed");

        // 2. Configure signer with visual stamp
        let signer = Signer::new(signing_key)
            .with_name("Techwasti Infotech")
            .with_reason("Contract Agreement Verification")
            .with_location("India")
            .with_visual_stamp(
                VisualSignature::at_page(1)
                    .rect(50.0, 50.0, 260.0, 120.0)
                    .title("Digitally Signed by Techwasti")
                    .detail("Status: Validated\nDate: 2026-10-25\nAuthority: Techwasti CA"),
            );

        // 3. Sign the PDF
        let signed_bytes = doc.sign(signer).expect("signing failed");
        assert!(signed_bytes.len() > 1000);

        // 4. Verify signature in the generated PDF
        let report = SignatureVerifier::verify(&signed_bytes).expect("verification failed");
        assert!(report.is_valid);
        assert_eq!(report.signer_name, Some("Techwasti Infotech".to_string()));
        assert_eq!(report.reason, Some("Contract Agreement Verification".to_string()));
        assert_eq!(report.location, Some("India".to_string()));
    }

    #[test]
    fn test_text_extraction() {
        let mut doc = PdfDocument::create();
        doc.add_blank_page(PDRectangle::A4);

        doc.draw_on_page(0, AppendMode::Overwrite, |stream, doc_inner| {
            stream.begin_text();
            stream.set_font_type1(doc_inner, 0, PDType1Font::HelveticaBold, 16.0);
            stream.show_text_at("Hello World from Apache PDFBox in Rust!", 50.0, 700.0);
            stream.end_text();
            Ok(())
        }).expect("draw failed");

        let extracted = doc.extract_text().expect("extraction failed");
        assert!(extracted.contains("Hello World from Apache PDFBox in Rust!"));
    }

    #[test]
    fn test_pdf_merger_and_splitter() {
        let mut doc1 = PdfDocument::create();
        doc1.add_blank_page(PDRectangle::A4);
        doc1.add_blank_page(PDRectangle::A4);

        let mut doc2 = PdfDocument::create();
        doc2.add_blank_page(PDRectangle::LETTER);

        doc1.merge(&doc2).expect("merge failed");
        assert_eq!(doc1.page_count(), 3);

        let split_docs = doc1.split_single_pages().expect("split failed");
        assert_eq!(split_docs.len(), 3);
        assert_eq!(split_docs[0].page_count(), 1);
        assert_eq!(split_docs[1].page_count(), 1);
        assert_eq!(split_docs[2].page_count(), 1);
    }

    #[test]
    fn test_fluent_facade_api() {
        let mut doc = PdfDocument::create();
        doc.set_title("Invoice #1042")
           .set_author("Techwasti Infotech");

        doc.add_blank_page(PDRectangle::A4);

        // Fluent watermark
        doc.watermark("PAID").expect("watermark failed");

        // Simple text drawing
        doc.draw_on_page(0, AppendMode::Append, |stream, doc_inner| {
            stream.begin_text();
            stream.set_font_type1(doc_inner, 0, PDType1Font::Helvetica, 14.0);
            stream.show_text_at("Invoice Total: $1,250.00 USD", 50.0, 750.0);
            stream.end_text();
            Ok(())
        }).expect("draw failed");

        // Verify serialization
        let bytes = doc.to_bytes().expect("to_bytes failed");
        let reloaded = PdfDocument::open_from_bytes(&bytes).expect("open failed");
        assert_eq!(reloaded.page_count(), 1);
        assert_eq!(reloaded.inner.info.title(), Some("Invoice #1042".to_string()));
    }
}

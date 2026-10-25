use rust_pdf_box::cos::{COSArray, COSDictionary, COSName, COSString};
use rust_pdf_box::prelude::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("tests/fixtures")?;

    println!("Generating sample PDF fixtures...");

    // 1. Generate sample_article.pdf (2 pages)
    {
        let mut doc = PdfDocument::create();
        doc.set_title("Architecture of Modern PDF Systems")
            .set_author("Techwasti Infotech");

        doc.add_blank_page(PDRectangle::A4);
        doc.add_blank_page(PDRectangle::A4);

        // Page 1
        doc.draw_on_page(0, AppendMode::Overwrite, |stream, inner_doc| {
            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 24.0);
            stream.set_fill_color(Color::rgb(0.1, 0.15, 0.35));
            stream.show_text_at("Architecture of Modern PDF Systems", 50.0, 750.0);
            stream.end_text();

            stream.set_stroke_color(Color::rgb(0.2, 0.4, 0.8));
            stream.set_line_width(2.0);
            stream.move_to(50.0, 735.0);
            stream.line_to(545.0, 735.0);
            stream.stroke();

            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::Helvetica, 11.0);
            stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
            stream.show_text_multiline(
                &[
                    "PDF (Portable Document Format) is an international standard (ISO 32000)",
                    "for representing documents independent of application software, hardware,",
                    "and operating systems.",
                    "",
                    "Key concepts include:",
                    "1. Low-level COS (Certificate Object Stream) object hierarchy.",
                    "2. High-level Page Description (PD) Model.",
                    "3. Content Streams containing postscript-like rendering operators.",
                    "4. Detached digital signatures using PKCS#7 / CMS standards.",
                ],
                50.0,
                700.0,
                16.0,
            );
            stream.end_text();
            Ok(())
        })?;

        // Page 2
        doc.draw_on_page(1, AppendMode::Overwrite, |stream, inner_doc| {
            stream.begin_text();
            stream.set_font_type1(inner_doc, 1, PDType1Font::HelveticaBold, 18.0);
            stream.set_fill_color(Color::rgb(0.1, 0.15, 0.35));
            stream.show_text_at("Section 2: Digital Signatures and Security", 50.0, 750.0);
            stream.end_text();

            stream.begin_text();
            stream.set_font_type1(inner_doc, 1, PDType1Font::Helvetica, 11.0);
            stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
            stream.show_text_multiline(
                &[
                    "Digital signatures provide authenticity, data integrity, and non-repudiation.",
                    "PDF files allow incremental updates where signatures are appended without",
                    "invalidating preceding byte content.",
                ],
                50.0,
                710.0,
                16.0,
            );
            stream.end_text();
            Ok(())
        })?;

        doc.save("tests/fixtures/sample_article.pdf")?;
        println!("Generated 'tests/fixtures/sample_article.pdf'");
    }

    // 2. Generate sample_invoice.pdf
    {
        let mut doc = PdfDocument::create();
        doc.set_title("Invoice #INV-2026-0042")
            .set_author("Techwasti Infotech");

        doc.add_blank_page(PDRectangle::A4);

        doc.draw_on_page(0, AppendMode::Overwrite, |stream, inner_doc| {
            // Header bar
            stream.set_fill_color(Color::rgb(0.12, 0.25, 0.55));
            stream.rect(0.0, 760.0, 595.276, 82.0);
            stream.fill();

            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 22.0);
            stream.set_fill_color(Color::WHITE);
            stream.show_text_at("TECHWASTI INFOTECH - INVOICE", 50.0, 790.0);
            stream.end_text();

            // Invoice details box
            stream.set_fill_color(Color::rgb(0.96, 0.97, 0.99));
            stream.set_stroke_color(Color::rgb(0.85, 0.88, 0.92));
            stream.set_line_width(1.0);
            stream.round_rect(50.0, 640.0, 495.0, 90.0, 4.0);
            stream.fill_and_stroke();

            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 11.0);
            stream.set_fill_color(Color::rgb(0.1, 0.1, 0.1));
            stream.show_text_at("Billed To: Global Systems Ltd", 65.0, 705.0);
            stream.set_font_type1(inner_doc, 0, PDType1Font::Helvetica, 10.0);
            stream.show_text_at("Invoice No: INV-2026-0042", 65.0, 685.0);
            stream.show_text_at("Date: 2026-10-25", 65.0, 665.0);
            stream.show_text_at("Due Date: 2026-11-25", 65.0, 645.0);
            stream.end_text();

            // Table Header
            stream.set_fill_color(Color::rgb(0.2, 0.3, 0.45));
            stream.rect(50.0, 580.0, 495.0, 25.0);
            stream.fill();

            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 10.0);
            stream.set_fill_color(Color::WHITE);
            stream.show_text_at("Description", 60.0, 590.0);
            stream.show_text_at("Hours", 320.0, 590.0);
            stream.show_text_at("Rate", 400.0, 590.0);
            stream.show_text_at("Amount (USD)", 465.0, 590.0);
            stream.end_text();

            // Line Items
            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::Helvetica, 10.0);
            stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
            stream.show_text_at("Cloud Architecture & Microservices", 60.0, 555.0);
            stream.show_text_at("40.0", 325.0, 555.0);
            stream.show_text_at("$150.00", 395.0, 555.0);
            stream.show_text_at("$6,000.00", 475.0, 555.0);

            stream.show_text_at("PDFBox Rust Implementation & Signatures", 60.0, 525.0);
            stream.show_text_at("30.0", 325.0, 525.0);
            stream.show_text_at("$150.00", 395.0, 525.0);
            stream.show_text_at("$4,500.00", 475.0, 525.0);
            stream.end_text();

            // Total Bar
            stream.set_stroke_color(Color::rgb(0.7, 0.7, 0.7));
            stream.move_to(50.0, 480.0);
            stream.line_to(545.0, 480.0);
            stream.stroke();

            stream.begin_text();
            stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 14.0);
            stream.set_fill_color(Color::rgb(0.1, 0.2, 0.4));
            stream.show_text_at("TOTAL DUE: $10,500.00 USD", 320.0, 450.0);
            stream.end_text();

            Ok(())
        })?;

        doc.save("tests/fixtures/sample_invoice.pdf")?;
        println!("Generated 'tests/fixtures/sample_invoice.pdf'");
    }

    // 3. Generate sample_form.pdf with AcroForm interactive fields
    {
        let mut doc = PDDocument::new();
        doc.info.set_title("Customer Application Form");
        doc.info.set_author("Techwasti Infotech");

        let page = PDPage::new(PDRectangle::A4);
        doc.add_page(page);

        // Content stream for form labels
        let mut stream = PDPageContentStream::new(AppendMode::Overwrite);
        stream.begin_text();
        stream.set_font_type1(&mut doc, 0, PDType1Font::HelveticaBold, 18.0);
        stream.set_fill_color(Color::rgb(0.1, 0.2, 0.4));
        stream.show_text_at("Customer Information Form", 50.0, 750.0);
        stream.end_text();

        stream.begin_text();
        stream.set_font_type1(&mut doc, 0, PDType1Font::Helvetica, 11.0);
        stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
        stream.show_text_at("Full Name:", 50.0, 680.0);
        stream.show_text_at("Email Address:", 50.0, 620.0);
        stream.show_text_at("Organization:", 50.0, 560.0);
        stream.end_text();
        stream.finish(&mut doc, 0)?;

        // Form fields
        let mut field1 = COSDictionary::new();
        field1.set("Type", COSName::new("Annot"));
        field1.set("Subtype", COSName::new("Widget"));
        field1.set("FT", COSName::new("Tx"));
        field1.set("T", COSString::literal("full_name"));
        field1.set("V", COSString::literal("Jane Doe"));
        field1.set("Rect", PDRectangle::new(150.0, 670.0, 450.0, 695.0));
        let f1_id = doc.cos_doc.add_object(field1);

        let mut field2 = COSDictionary::new();
        field2.set("Type", COSName::new("Annot"));
        field2.set("Subtype", COSName::new("Widget"));
        field2.set("FT", COSName::new("Tx"));
        field2.set("T", COSString::literal("email"));
        field2.set("V", COSString::literal("jane.doe@example.com"));
        field2.set("Rect", PDRectangle::new(150.0, 610.0, 450.0, 635.0));
        let f2_id = doc.cos_doc.add_object(field2);

        let mut field3 = COSDictionary::new();
        field3.set("Type", COSName::new("Annot"));
        field3.set("Subtype", COSName::new("Widget"));
        field3.set("FT", COSName::new("Tx"));
        field3.set("T", COSString::literal("company"));
        field3.set("V", COSString::literal("Techwasti Infotech"));
        field3.set("Rect", PDRectangle::new(150.0, 550.0, 450.0, 575.0));
        let f3_id = doc.cos_doc.add_object(field3);

        // Add fields to Page Annots
        let mut annots = COSArray::new();
        annots.push(f1_id);
        annots.push(f2_id);
        annots.push(f3_id);

        let page_ref = doc.get_page_mut(0)?;
        page_ref.dict.set("Annots", annots.clone());

        // Add AcroForm to Document Catalog
        let mut acro = COSDictionary::new();
        acro.set("Fields", annots);
        if let Some(cat) = doc.cos_doc.get_catalog_mut() {
            cat.set("AcroForm", acro);
        }

        doc.save_to_file("tests/fixtures/sample_form.pdf")?;
        println!("Generated 'tests/fixtures/sample_form.pdf'");
    }

    println!("All sample PDF fixtures successfully generated!");
    Ok(())
}

use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rust-pdf-box: Quick Start Guide ===");

    // 1. Create a document
    let mut doc = PdfDocument::create();
    doc.set_title("My First Rust PDF")
       .set_author("Techwasti Infotech");

    // 2. Add an A4 page
    let page_idx = doc.add_blank_page(PDRectangle::A4);

    // 3. Draw text and geometric shapes
    doc.draw_on_page(page_idx, AppendMode::Overwrite, |stream, inner_doc| {
        // Decorative header banner
        stream.set_fill_color(Color::rgb(0.1, 0.4, 0.8));
        stream.round_rect(40.0, 720.0, 515.0, 80.0, 6.0);
        stream.fill();

        // Banner text
        stream.begin_text();
        stream.set_font_type1(inner_doc, page_idx, PDType1Font::HelveticaBold, 22.0);
        stream.set_fill_color(Color::WHITE);
        stream.show_text_at("Welcome to rust-pdf-box!", 60.0, 755.0);
        stream.end_text();

        // Body paragraph
        stream.begin_text();
        stream.set_font_type1(inner_doc, page_idx, PDType1Font::Helvetica, 12.0);
        stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
        stream.show_text_multiline(
            &[
                "rust-pdf-box provides an idiomatic, context-oriented, and robust",
                "Rust implementation of Apache PDFBox.",
                "",
                "Features include:",
                "  * High-speed COS/PD object model",
                "  * Simple stamping & watermarking engine",
                "  * ISO 32000 digital signatures with X.509 self-signing",
                "  * AcroForm filling and form flattening",
                "  * Clean UTF-8 text extraction and document assembly",
            ],
            60.0,
            680.0,
            18.0,
        );
        stream.end_text();

        // Circle and shapes
        stream.set_stroke_color(Color::rgb(0.8, 0.2, 0.2));
        stream.set_fill_color(Color::rgb(0.95, 0.85, 0.85));
        stream.set_line_width(2.0);
        stream.circle(100.0, 450.0, 35.0);
        stream.fill_and_stroke();

        Ok(())
    })?;

    // 4. Save to disk
    doc.save("quickstart_demo.pdf")?;
    println!("Document saved successfully to 'quickstart_demo.pdf'!");

    Ok(())
}

use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rust-pdf-box: Stamping & Watermarking Demo ===");

    let mut doc = PdfDocument::create();
    doc.set_title("Techwasti Invoice & Agreement")
       .set_author("Techwasti Infotech");

    // Add 2 pages
    doc.add_blank_page(PDRectangle::A4);
    doc.add_blank_page(PDRectangle::A4);

    // 1. Draw page content on page 1
    doc.draw_on_page(0, AppendMode::Overwrite, |stream, inner_doc| {
        stream.begin_text();
        stream.set_font_type1(inner_doc, 0, PDType1Font::HelveticaBold, 22.0);
        stream.set_fill_color(Color::rgb(0.1, 0.1, 0.3));
        stream.show_text_at("Techwasti Infotech - Statement of Work", 50.0, 750.0);
        stream.end_text();

        stream.set_stroke_color(Color::rgb(0.3, 0.3, 0.3));
        stream.set_line_width(1.0);
        stream.move_to(50.0, 735.0);
        stream.line_to(545.0, 735.0);
        stream.stroke();

        stream.begin_text();
        stream.set_font_type1(inner_doc, 0, PDType1Font::Helvetica, 12.0);
        stream.set_fill_color(Color::rgb(0.2, 0.2, 0.2));
        stream.show_text_multiline(
            &[
                "Client: Global Enterprise Corp",
                "Service: High-Performance Cloud & PDF Microservices",
                "Date: October 25, 2026",
                "Status: Pending Final Verification",
            ],
            50.0,
            700.0,
            18.0,
        );
        stream.end_text();
        Ok(())
    })?;

    // 2. Add diagonal semi-transparent background watermark on all pages
    doc.stamp()
        .pages(PageRange::All)
        .watermark("CONFIDENTIAL", 56.0, Color::rgb(0.85, 0.15, 0.15), 45.0, 0.18)
        .apply(&mut doc.inner)?;

    // 3. Add foreground rubber stamp on page 1
    doc.stamp()
        .pages(PageRange::Single(1))
        .position(StampPosition::TopRight { margin_x: 40.0, margin_y: 40.0 })
        .rubber_stamp("VERIFIED", Some("Techwasti Security".to_string()), Color::rgb(0.0, 0.55, 0.2), -12.0)
        .apply(&mut doc.inner)?;

    // 4. Add header and footer with dynamic page numbering
    doc.stamp()
        .pages(PageRange::All)
        .header("Techwasti Infotech Confidential - Page {page} of {total}", StampPosition::TopCenter { margin_y: 18.0 }, 9.0, Color::GRAY)
        .apply(&mut doc.inner)?;

    doc.stamp()
        .pages(PageRange::All)
        .footer("Protected under ISO 32000-1 / rust-pdf-box specification", StampPosition::BottomCenter { margin_y: 18.0 }, 8.0, Color::GRAY)
        .apply(&mut doc.inner)?;

    doc.save("stamped_demo.pdf")?;
    println!("Successfully generated 'stamped_demo.pdf'!");

    Ok(())
}

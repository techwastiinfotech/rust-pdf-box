use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rust-pdf-box: Text Extraction Demo ===");

    let fixture_path = "tests/fixtures/sample_article.pdf";
    println!("Loading PDF from '{}'...", fixture_path);

    let doc = PdfDocument::open(fixture_path)?;
    println!("Document loaded successfully ({} pages).", doc.page_count());

    // 1. Extract text from each individual page
    for p in 0..doc.page_count() {
        let page_text = PDFTextStripper::extract_page_text(&doc.inner, p)?;
        println!(
            "\n--- Page {} Text ({} characters) ---",
            p + 1,
            page_text.len()
        );
        println!("{}", page_text);
    }

    // 2. Extract complete document text
    let full_text = doc.extract_text()?;
    let word_count = full_text.split_whitespace().count();
    println!("\n=== Summary ===");
    println!("Total Word Count: {}", word_count);
    println!("Total Character Count: {}", full_text.len());

    Ok(())
}

use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rust-pdf-box: Merge & Split Demo ===");

    // 1. Load two separate sample PDFs
    let article = PdfDocument::open("tests/fixtures/sample_article.pdf")?;
    let invoice = PdfDocument::open("tests/fixtures/sample_invoice.pdf")?;

    println!("Article page count: {}", article.page_count());
    println!("Invoice page count: {}", invoice.page_count());

    // 2. Merge them into a unified PDF
    println!("\nMerging article and invoice into a single document...");
    let mut merged = PdfDocument::create();
    merged.merge(&article)?;
    merged.merge(&invoice)?;

    println!("Merged total page count: {}", merged.page_count());
    merged.save("merged_output_demo.pdf")?;
    println!("Saved merged document to 'merged_output_demo.pdf'!");

    // 3. Split the merged document back into individual 1-page documents
    println!("\nSplitting merged document into single pages...");
    let single_pages = merged.split_single_pages()?;
    println!(
        "Successfully split into {} individual documents.",
        single_pages.len()
    );

    for (i, mut single_doc) in single_pages.into_iter().enumerate() {
        let output_filename = format!("split_page_{}.pdf", i + 1);
        single_doc.save(&output_filename)?;
        println!("  -> Saved '{}' (1 page)", output_filename);
    }

    Ok(())
}

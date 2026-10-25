use rust_pdf_box::prelude::*;

#[test]
fn test_text_extraction_from_fixture() {
    let doc = PdfDocument::open("tests/fixtures/sample_article.pdf")
        .expect("load sample_article.pdf failed");

    let text = doc.extract_text().expect("extract_text failed");
    assert!(text.contains("Architecture of Modern PDF Systems"));
    assert!(text.contains("ISO 32000"));
    assert!(text.contains("Digital Signatures and Security"));
    assert!(text.contains("PKCS#7 / CMS"));
}

#[test]
fn test_merging_and_splitting_fixtures() {
    let article = PdfDocument::open("tests/fixtures/sample_article.pdf").unwrap();
    let invoice = PdfDocument::open("tests/fixtures/sample_invoice.pdf").unwrap();

    let mut merged = PdfDocument::create();
    merged.merge(&article).unwrap();
    merged.merge(&invoice).unwrap();

    assert_eq!(merged.page_count(), 3);

    // Save and re-read merged document
    let merged_bytes = merged.to_bytes().unwrap();
    let reloaded = PdfDocument::open_from_bytes(&merged_bytes).unwrap();
    assert_eq!(reloaded.page_count(), 3);

    // Split into 3 single-page documents
    let singles = reloaded.split_single_pages().unwrap();
    assert_eq!(singles.len(), 3);
    for single in &singles {
        assert_eq!(single.page_count(), 1);
    }
}

#[test]
fn test_page_rotation_and_reordering() {
    let mut doc = PdfDocument::open("tests/fixtures/sample_article.pdf").unwrap();
    assert_eq!(doc.page_count(), 2);

    // Initial rotation is 0
    assert_eq!(doc.inner.get_page(0).unwrap().rotation(), 0);

    // Rotate page 0 by 90 degrees
    PageOps::rotate_page(&mut doc.inner, 0, 90).unwrap();
    assert_eq!(doc.inner.get_page(0).unwrap().rotation(), 90);

    // Move page 0 to index 1
    PageOps::move_page(&mut doc.inner, 0, 1).unwrap();
    // Now page 1 has rotation 90
    assert_eq!(doc.inner.get_page(1).unwrap().rotation(), 90);
}

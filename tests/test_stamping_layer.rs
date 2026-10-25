use rust_pdf_box::prelude::*;

#[test]
fn test_page_range_matching() {
    let total = 5;

    let all = PageRange::All;
    assert!(all.matches(0, total));
    assert!(all.matches(4, total));

    let odd = PageRange::Odd;
    assert!(odd.matches(0, total)); // Page 1 -> odd
    assert!(!odd.matches(1, total)); // Page 2 -> even
    assert!(odd.matches(2, total)); // Page 3 -> odd

    let even = PageRange::Even;
    assert!(!even.matches(0, total)); // Page 1
    assert!(even.matches(1, total)); // Page 2
    assert!(even.matches(3, total)); // Page 4

    let single = PageRange::Single(3);
    assert!(!single.matches(0, total));
    assert!(single.matches(2, total)); // 0-based 2 == 1-based 3

    let range = PageRange::Range(2..=4);
    assert!(!range.matches(0, total)); // Page 1
    assert!(range.matches(1, total)); // Page 2
    assert!(range.matches(3, total)); // Page 4
    assert!(!range.matches(4, total)); // Page 5
}

#[test]
fn test_stamping_pipeline_and_layering() {
    let mut doc = PdfDocument::create();
    for _ in 0..4 {
        doc.add_blank_page(PDRectangle::A4);
    }

    // 1. Prepend background watermark on odd pages
    doc.stamp()
        .pages(PageRange::Odd)
        .layer(StampLayer::UnderContent)
        .watermark("DRAFT", 48.0, Color::rgb(0.9, 0.2, 0.2), 30.0, 0.2)
        .apply(&mut doc.inner)
        .expect("watermark failed");

    // 2. Append foreground rubber stamp on page 2
    doc.stamp()
        .pages(PageRange::Single(2))
        .position(StampPosition::BottomRight { margin_x: 30.0, margin_y: 30.0 })
        .rubber_stamp("COMPLETED", Some("Audit Pass".to_string()), Color::rgb(0.0, 0.5, 0.2), 0.0)
        .apply(&mut doc.inner)
        .expect("rubber stamp failed");

    // 3. Header on all pages
    doc.stamp()
        .pages(PageRange::All)
        .header("Techwasti - Page {page} of {total}", StampPosition::TopCenter { margin_y: 20.0 }, 10.0, Color::GRAY)
        .apply(&mut doc.inner)
        .expect("header failed");

    let bytes = doc.to_bytes().expect("to_bytes failed");
    let loaded = PdfDocument::open_from_bytes(&bytes).expect("open failed");

    assert_eq!(loaded.page_count(), 4);
    // Page 1 (odd) has watermark + header
    assert!(loaded.inner.get_page(0).unwrap().content_stream_ids().len() >= 2);
    // Page 2 (even) has rubber stamp + header
    assert!(loaded.inner.get_page(1).unwrap().content_stream_ids().len() >= 2);
    // Page 3 (odd) has watermark + header
    assert!(loaded.inner.get_page(2).unwrap().content_stream_ids().len() >= 2);
}

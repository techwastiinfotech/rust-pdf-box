# Document Assembly (Merge & Split)

`rust-pdf-box` provides dedicated utilities for merging multiple PDF documents, splitting documents into ranges or individual pages, and rotating/reordering pages without data loss.

---

## 1. Merging Multiple Documents (`PDFMerger`)

When combining multiple PDF documents, naive object concatenation causes severe identifier collisions. `PDFMerger` automatically resolves object number conflicts by building a collision-free remapping table while preserving fonts, XObjects, bookmarks, and metadata:

```rust
use rust_pdf_box::manipulation::PDFMerger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut merger = PDFMerger::new();

    // Add source documents in order
    merger.add_source("cover.pdf")?;
    merger.add_source("executive_summary.pdf")?;
    merger.add_source("financial_statements.pdf")?;
    merger.add_source("appendix.pdf")?;

    // Produce merged document
    merger.merge("complete_annual_report.pdf")?;

    println!("Successfully merged 4 documents into complete_annual_report.pdf");
    Ok(())
}
```

---

## 2. Splitting Documents (`PDFSplitter`)

`PDFSplitter` divides a large PDF into manageable parts, either page-by-page or by custom range:

```rust
use rust_pdf_box::manipulation::PDFSplitter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = PDFSplitter::new();

    // Split into individual single-page documents
    let split_docs = splitter.split_to_single_pages("large_handbook.pdf")?;

    for (index, mut single_page_doc) in split_docs.into_iter().enumerate() {
        let output_path = format!("page_{:03}.pdf", index + 1);
        single_page_doc.save(&output_path)?;
    }

    Ok(())
}
```

---

## 3. Page Reordering & Rotation

Individual pages can be rotated (in multiples of 90 degrees) and rearranged:

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("scanned_document.pdf")?;

    // Rotate page 1 by 90 degrees clockwise
    if let Ok(page) = doc.inner.page_mut(1) {
        page.set_rotation(90);
    }

    // Delete an obsolete page
    doc.inner.remove_page(3)?;

    doc.save("scanned_document_corrected.pdf")?;
    Ok(())
}
```

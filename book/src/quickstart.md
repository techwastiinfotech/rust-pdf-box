# Quick Start Guide

Get up and running with `rust-pdf-box` in your Rust project in just a few minutes.

---

## 1. Installation

Add `rust-pdf-box` to your `Cargo.toml`:

```toml
[dependencies]
rust-pdf-box = "0.1.0"
```

Or using Cargo CLI:

```bash
cargo add rust-pdf-box
```

---

## 2. Your First Document

Create a blank PDF document, add a page, write text, and save:

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize a new PDF document
    let mut doc = PdfDocument::new();

    // 2. Add an A4 page
    let page_id = doc.add_blank_page(PDRectangle::a4())?;

    // 3. Write text onto the page
    let mut cs = doc.page_content_stream(page_id, AppendMode::Overwrite)?;
    cs.begin_text()?
        .set_font(PDType1Font::HelveticaBold, 22.0)?
        .set_non_stroking_color(Color::rgb(0.05, 0.45, 0.40))? // Techwasti Teal
        .set_text_matrix(Matrix::translation(50.0, 750.0))?
        .show_text("Hello from rust-pdf-box!")?
        .end_text()?;
    cs.finish()?;

    // 4. Save to file
    doc.save("hello_world.pdf")?;
    println!("Successfully generated hello_world.pdf");

    Ok(())
}
```

---

## 3. Watermarking in 3 Lines

Apply a semi-transparent diagonal watermark across all pages:

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("invoice.pdf")?;

    doc.stamp()
        .pages(PageRange::All)
        .watermark("CONFIDENTIAL", 54.0, Color::rgb(0.85, 0.1, 0.1), 45.0, 0.25)
        .apply(&mut doc.inner)?;

    doc.save("invoice_watermarked.pdf")?;
    Ok(())
}
```

---

## 4. Digital Signature with In-Memory Key

Digitally sign a PDF document adhering to ISO 32000 CMS/PKCS#7 with a visual appearance stamp:

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("contract.pdf")?;

    // Generate an in-memory X.509 v3 self-signed RSA-2048 signing key
    let key = SigningKey::generate_self_signed(
        "CN=Techwasti Signer, O=Techwasti Infotech, C=IN"
    )?;

    // Configure visual signature appearance
    let appearance = SignatureAppearance::new(1, PDRectangle::new(50.0, 50.0, 250.0, 100.0))
        .with_signer_name("Techwasti Infotech")
        .with_reason("Contract Approval")
        .with_location("Bangalore, India");

    // Apply detached PKCS#7 signature
    doc.signer()
        .with_key(key)
        .with_appearance(appearance)
        .sign_to_file("contract.pdf", "contract_signed.pdf")?;

    println!("Signed document successfully created!");
    Ok(())
}
```

---

## 5. Merging Documents

Merge multiple PDF files into one combined output:

```rust
use rust_pdf_box::manipulation::PDFMerger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut merger = PDFMerger::new();
    merger.add_source("cover.pdf")?;
    merger.add_source("report.pdf")?;
    merger.add_source("appendix.pdf")?;

    merger.merge("complete_report.pdf")?;
    println!("Merged report generated!");
    Ok(())
}
```

---

## 6. Text Extraction

Extract plain text from an existing document:

```rust
use rust_pdf_box::prelude::*;
use rust_pdf_box::text::PDFTextStripper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = PdfDocument::open("sample.pdf")?;
    let stripper = PDFTextStripper::new();
    let text = stripper.get_text(&doc.inner)?;

    println!("Extracted Text:\n{}", text);
    Ok(())
}
```

# rust-pdf-box

A comprehensive, pure Rust implementation of Apache PDFBox featuring modern, context-oriented, and ergonomic PDF processing.

Designed by **Techwasti Infotech** for high-performance PDF manipulation, easy visual/digital signing, watermarking, stamping, AcroForm interaction, text extraction, and document assembly.

## Key Features

- **Low-Level COS Architecture**: Full COS model (`COSObject`, `COSDictionary`, `COSArray`, `COSStream`, `COSString`, `COSName`) with robust PDF lexer, parser, and serializer supporting both classic XRef tables and modern compressed XRef streams (`/ObjStm`, `/XRef`).
- **High-Level PD Model**: Page Description model (`PDDocument`, `PDPage`, `PDRectangle`, `PDResources`, `PDDocumentCatalog`, `PDDocumentInformation`).
- **Context-Oriented Stamping & Watermarking**:
  - Intuitive fluent builder API for watermarks, rubber stamps, headers/footers, and bates numbering.
  - Multi-page range targeting (`PageRange::All`, `PageRange::Odd`, `PageRange::Even`, `PageRange::Range(1..=5)`).
  - Rotated, transparent, multi-line, text & image stamps applied either over or under existing page content.
- **Easy Digital Signatures (`PDDigitalSignature`)**:
  - Full ISO 32000 compliant detached PKCS#7 / CMS digital signatures (`/adbe.pkcs7.detached`, `/ETSI.CAdES.detached`).
  - Seamless self-signed certificate generation in pure Rust (`SigningKey::generate_self_signed(...)`) for rapid testing and internal signing.
  - Custom PKCS#8 / PEM / DER private key & certificate loading.
  - Visual signature appearances (`/AP`) with custom text, signer details, timestamps, reason, location, and optional signature stamp image.
  - Incremental update signing preserving existing signatures and document integrity.
- **Content Stream Engine (`PDPageContentStream`)**:
  - Path construction & painting (lines, Bézier curves, rectangles, circles, clipping).
  - Advanced graphics state (transparency / alpha, line widths, caps, joins, dash patterns, RGB / CMYK / Gray colors).
  - Text matrix, multiline text layout, standard Type 1 fonts (Helvetica, Times, Courier, Symbol, ZapfDingbats) and TrueType embedding.
  - Image embedding (JPEG passthrough, PNG with alpha masks / soft masks).
- **Interactive Forms & Annotations (AcroForm)**:
  - Form field inspection, value editing, and form flattening.
  - Support for text fields, checkboxes, choice fields, and signature fields.
- **Text Extraction (`PDFTextStripper`)**:
  - Clean UTF-8 text extraction with position metrics from PDF content streams.
- **Document Operations**:
  - Merging multiple PDF files with object remapping.
  - Splitting PDFs into ranges or individual pages.
  - Page operations: rotation, reordering, duplicate, delete.

## Quick Start

### Simple Stamping & Watermarking

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("input.pdf")?;

    // 1. Semi-transparent background watermark
    doc.stamp()
        .pages(PageRange::All)
        .watermark("CONFIDENTIAL", 54.0, Color::rgb(0.85, 0.1, 0.1), 45.0, 0.25)
        .apply(&mut doc.inner)?;

    // 2. Rubber stamp foreground overlay
    doc.stamp()
        .pages(PageRange::Single(1))
        .position(StampPosition::TopRight { margin_x: 36.0, margin_y: 36.0 })
        .rubber_stamp("APPROVED", Some("Techwasti Security".to_string()), Color::rgb(0.0, 0.6, 0.2), -15.0)
        .apply(&mut doc.inner)?;

    // 3. Dynamic Header / Footer with page numbers
    doc.stamp()
        .pages(PageRange::All)
        .header("Document - Page {page} of {total}", StampPosition::TopCenter { margin_y: 20.0 }, 10.0, Color::GRAY)
        .apply(&mut doc.inner)?;

    doc.save("stamped.pdf")?;
    Ok(())
}
```

### Easy Digital Signing in Pure Rust

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("contract.pdf")?;

    // Generate or load an X.509 signing key (zero external CLI setup required!)
    let key = SigningKey::generate_self_signed("CN=Techwasti, O=Techwasti Infotech")?;

    let signer = Signer::new(key)
        .with_name("Techwasti Infotech")
        .with_reason("Contract Agreement Execution")
        .with_location("India")
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(50.0, 50.0, 280.0, 130.0)
                .title("Digitally Signed by Techwasti")
                .detail("Signer: Techwasti Authorized\nDate: 2026-10-25\nStatus: Cryptographically Verified")
        );

    let signed_bytes = doc.sign(signer)?;
    std::fs::write("contract_signed.pdf", &signed_bytes)?;

    // Verify digital signature
    let report = SignatureVerifier::verify(&signed_bytes)?;
    println!("Signature valid: {}", report.is_valid);
    Ok(())
}
```

### Text Extraction & PDF Merging

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc1 = PdfDocument::open("doc1.pdf")?;
    let text = doc1.extract_text()?;
    println!("Extracted Text:\n{}", text);

    let mut combined = PdfDocument::create();
    let doc2 = PdfDocument::open("doc2.pdf")?;
    combined.merge(&doc1)?;
    combined.merge(&doc2)?;
    combined.save("merged.pdf")?;
    Ok(())
}
```

## Running Examples

```bash
cargo run --example stamping_demo
cargo run --example digital_signature_demo
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

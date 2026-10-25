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

### Simple Stamping

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("input.pdf")?;

    doc.stamp()
        .pages(PageRange::All)
        .watermark("CONFIDENTIAL")
        .font_size(54.0)
        .opacity(0.25)
        .rotation(45.0)
        .color(Color::rgb(0.85, 0.1, 0.1))
        .layer(StampLayer::UnderContent)
        .apply()?;

    doc.save("stamped.pdf")?;
    Ok(())
}
```

### Easy Digital Signing

```rust
use rust_pdf_box::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::open("contract.pdf")?;

    // Generate or load a signing key
    let key = SigningKey::generate_self_signed("CN=Techwasti, O=Techwasti Infotech")?;

    let signer = Signer::new(key)
        .with_name("Techwasti Infotech")
        .with_reason("Contract Agreement")
        .with_location("India")
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(72.0, 72.0, 260.0, 140.0)
                .title("Digitally Signed")
                .detail("Signer: Techwasti Infotech\nDate: 2026-10-25\nStatus: Verified")
        );

    doc.sign(signer, "contract_signed.pdf")?;
    Ok(())
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

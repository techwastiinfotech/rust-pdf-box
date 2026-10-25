# rust-pdf-box

<div align="center">

[![CI](https://github.com/techwastiinfotech/rust-pdf-box/actions/workflows/ci.yml/badge.svg)](https://github.com/techwastiinfotech/rust-pdf-box/actions)
[![Crates.io](https://img.shields.io/crates/v/rust-pdf-box.svg)](https://crates.io/crates/rust-pdf-box)
[![Docs.rs](https://docs.rs/rust-pdf-box/badge.svg)](https://docs.rs/rust-pdf-box)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](#)
[![Maintained by Techwasti](https://img.shields.io/badge/maintained%20by-techwasti%20infotech-teal.svg)](https://techwasti.com)

**A comprehensive, pure Rust implementation of Apache PDFBox featuring modern, context-oriented, and ergonomic PDF processing.**

[Key Features](#key-features) •
[Quick Start](#quick-start) •
[Guides & Architecture](#documentation--guides) •
[Examples](#examples--sample-pdfs) •
[Contributing](#contributing)

</div>

---

## Overview

`rust-pdf-box` is an open-source, pure Rust library bringing the full feature set of **Apache PDFBox** into modern Rust with zero external C/C++ or OpenSSL dependencies.

Designed by **Techwasti Infotech**, `rust-pdf-box` eliminates the boilerplate typical of low-level PDF manipulation. It provides a simple, context-oriented facade that makes common, high-demand operations—such as **digital signing with ISO 32000 compliance**, **watermarking**, **rubber stamping**, **interactive AcroForm processing**, and **document merging/splitting**—straightforward and reliable.

---

## Key Features

- **Low-Level COS Architecture**: Full COS object model (`COSObject`, `COSDictionary`, `COSArray`, `COSStream`, `COSString`, `COSName`) with a fault-tolerant PDF lexer, parser, and serializer supporting classic XRef tables and modern compressed object streams (`/Type /XRef`, `/Type /ObjStm`).
- **High-Level PD Model**: Page Description layer (`PDDocument`, `PDPage`, `PDRectangle`, `PDResources`, `PDDocumentCatalog`, `PDDocumentInformation`).
- **Context-Oriented Stamping & Watermarking**:
  - Intuitive fluent builder API for watermarks, rubber stamps, running headers/footers, and bates numbering.
  - Multi-page range targeting (`PageRange::All`, `PageRange::Odd`, `PageRange::Even`, `PageRange::Range(1..=5)`).
  - Rotated, transparent, multi-line, text & image stamps applied either over or under existing page content.
- **Easy Digital Signatures (`PDDigitalSignature`)**:
  - Full ISO 32000 compliant detached PKCS#7 / CMS digital signatures (`/adbe.pkcs7.detached`, `/ETSI.CAdES.detached`).
  - Seamless self-signed certificate generation in pure Rust (`SigningKey::generate_self_signed(...)`) for rapid testing, staging, and internal signing without OpenSSL CLI scripts.
  - Custom PKCS#8 / PEM / DER private key & certificate loading.
  - Visual signature appearances (`/AP`) with custom text, signer details, timestamps, reason, location, and optional signature stamp image.
  - Incremental update signing preserving existing signatures and document integrity.
- **Interactive Forms (AcroForms) & Annotations**:
  - Form field inspection, value editing, and form flattening.
  - Support for text fields (`/Tx`), checkboxes & radio buttons (`/Btn`), choice fields (`/Ch`), and signature fields (`/Sig`).
  - Clickable hyperlink annotations (`/Link`) and visual markups.
- **Content Stream Engine (`PDPageContentStream`)**:
  - Path construction & painting (lines, Bézier curves, rectangles, circles, clipping).
  - Advanced graphics state (transparency / alpha, line widths, caps, joins, dash patterns, RGB / CMYK / Gray colors).
  - Text matrix, multiline text layout, standard Type 1 fonts (Helvetica, Times, Courier, Symbol, ZapfDingbats) and TrueType embedding.
  - Image embedding (native JPEG `/DCTDecode` passthrough, PNG with alpha mask / soft mask `/SMask`).
- **Text Extraction (`PDFTextStripper`)**:
  - Clean UTF-8 text extraction with position metrics from PDF content streams.
- **Document Assembly & Page Operations**:
  - Merging multiple PDF files with collision-free object ID remapping.
  - Splitting PDFs into ranges or individual pages.
  - Page operations: rotation, reordering, duplicate, delete.

---

## Installation

Add `rust-pdf-box` to your `Cargo.toml`:

```toml
[dependencies]
rust-pdf-box = "0.1.0"
```

---

## Quick Start

### 1. Simple Stamping & Watermarking

Add watermarks, rubber stamps, and running page headers/footers in just a few lines:

```rust
use rust_pdf_box::prelude::*;

fn main() -> PdfResult<()> {
    let mut doc = PdfDocument::open("document.pdf")?;

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

    // 3. Dynamic Header / Footer with automatic page numbering
    doc.stamp()
        .pages(PageRange::All)
        .header("Document - Page {page} of {total}", StampPosition::TopCenter { margin_y: 20.0 }, 10.0, Color::GRAY)
        .apply(&mut doc.inner)?;

    doc.save("stamped.pdf")?;
    Ok(())
}
```

---

### 2. Digital Signatures in Pure Rust (ISO 32000 CMS / PKCS#7)

Sign PDFs cryptographically with self-signed or enterprise PKCS#8 certificates:

```rust
use rust_pdf_box::prelude::*;

fn main() -> PdfResult<()> {
    let mut doc = PdfDocument::open("contract.pdf")?;

    // Generate or load an X.509 signing key (zero external CLI setup required!)
    let key = SigningKey::generate_self_signed("CN=Techwasti Signer, O=Techwasti Infotech")?;

    let signer = Signer::new(key)
        .with_name("Techwasti Infotech")
        .with_reason("Contract Agreement Execution")
        .with_location("Mumbai, India")
        .with_visual_stamp(
            VisualSignature::at_page(1)
                .rect(50.0, 50.0, 280.0, 100.0)
                .title("Digitally Signed by Techwasti")
                .detail("Signer: Techwasti Authorized\nDate: 2026-10-25\nStatus: Cryptographically Verified")
        );

    // Creates an ISO 32000 detached PKCS#7 signature in the PDF
    let signed_bytes = doc.sign(signer)?;
    std::fs::write("contract_signed.pdf", &signed_bytes)?;

    // Cryptographic integrity verification
    let report = SignatureVerifier::verify(&signed_bytes)?;
    println!("Signature valid: {}", report.is_valid);
    println!("Signer name: {:?}", report.signer_name);
    Ok(())
}
```

---

### 3. AcroForms & Form Flattening

Read form fields, update values, and flatten them into permanent vector graphics:

```rust
use rust_pdf_box::prelude::*;

fn main() -> PdfResult<()> {
    let mut doc = PdfDocument::open("sample_form.pdf")?;

    if let Some(mut form) = doc.inner.catalog_mut()?.acro_form()? {
        // Inspect fields
        for field in form.fields(&doc.inner)? {
            println!("Field: {} = {:?}", field.name, field.value);
        }

        // Fill values
        form.set_field_value(&mut doc.inner, "FullName", "Jane Doe")?;
        form.set_field_value(&mut doc.inner, "Organization", "Techwasti")?;

        // Flatten: converts interactive inputs to permanent graphics
        form.flatten(&mut doc.inner)?;
    }

    doc.save("flattened_form.pdf")?;
    Ok(())
}
```

---

### 4. Text Extraction & Document Merging / Splitting

```rust
use rust_pdf_box::prelude::*;

fn main() -> PdfResult<()> {
    // Text extraction
    let doc1 = PdfDocument::open("sample_article.pdf")?;
    let text = doc1.extract_text()?;
    println!("Extracted text:\n{}", text);

    // Merging
    let doc2 = PdfDocument::open("sample_invoice.pdf")?;
    let mut combined = PdfDocument::create();
    combined.merge(&doc1)?;
    combined.merge(&doc2)?;
    combined.save("merged.pdf")?;

    // Splitting
    let pages = PDFSplitter::split_into_single_pages(&doc1.inner)?;
    for (i, p) in pages.into_iter().enumerate() {
        let mut sub_doc = PdfDocument { inner: p };
        sub_doc.save(&format!("page_{}.pdf", i + 1))?;
    }

    Ok(())
}
```

---

## Standalone CLI Tool (`pdfbox`)

`rust-pdf-box` includes a cross-platform CLI binary that can be used directly from shell scripts, Python, Node.js, Docker, or other CI/CD pipelines:

```bash
# Display metadata & page count
pdfbox info document.pdf

# Extract all text
pdfbox text document.pdf output.txt

# Watermark across all pages
pdfbox watermark input.pdf watermarked.pdf "CONFIDENTIAL" 0.25 45

# Apply rubber stamp
pdfbox stamp contract.pdf approved.pdf "APPROVED" "Security Team"

# Digitally sign with ISO 32000 CMS/PKCS#7
pdfbox sign contract.pdf signed.pdf "Techwasti Authority" "Execution" "India"

# Cryptographically verify signatures & detect tampering
pdfbox verify signed.pdf

# Merge multiple PDFs
pdfbox merge combined.pdf part1.pdf part2.pdf part3.pdf

# Split multi-page document into single pages
pdfbox split report.pdf ./pages/

# Flatten interactive form fields
pdfbox flatten form.pdf flattened.pdf
```

---

## Documentation & Guides

Comprehensive guides are available in the [`docs/`](docs/) directory:

- 🚀 [**Integration & Artifacts Guide**](docs/INTEGRATION_AND_ARTIFACTS.md): How to use `rust-pdf-box` in other projects, Docker containers, Python/Node.js, and CI/CD pipelines.
- 📐 [**Architecture Guide**](docs/ARCHITECTURE.md): Deep dive into the COS object tree, Page Description layer, and serialization pipeline.
- 🔏 [**Digital Signatures Guide**](docs/DIGITAL_SIGNATURES.md): Detailed explanation of ISO 32000 CMS / PKCS#7 signing, `/ByteRange` calculation, and tamper verification.
- 🎨 [**Stamping & Watermarking Guide**](docs/STAMPING_AND_WATERMARKING.md): Advanced watermarking patterns, rotation, multi-line text, rubber stamps, and page ranges.
- 📝 [**AcroForms and Annotations**](docs/ACROFORMS_AND_ANNOTATIONS.md): Inspecting form fields, updating values, flattening forms, and adding hyperlinks.
- 🔄 [**Apache PDFBox to Rust Migration**](docs/APACHE_PDFBOX_MIGRATION.md): Java-to-Rust class comparisons, syntax mappings, and architectural benefits.

---

## Examples & Sample PDFs

The repository includes a variety of executable examples and sample PDF fixtures located in [`tests/fixtures/`](tests/fixtures/):

| Fixture File | Description |
| :--- | :--- |
| `sample_article.pdf` | Multi-page structured publication with headings, paragraphs, and metadata |
| `sample_invoice.pdf` | Commercial invoice with formatted table grids, line items, and totals |
| `sample_form.pdf` | Interactive AcroForm with text fields, default appearances, and form catalog |

### Running the Examples

```bash
# 1. Quick start demonstration
cargo run --example quickstart

# 2. Watermarking and rubber stamping
cargo run --example stamping_demo

# 3. Pure-Rust digital signing and verification
cargo run --example digital_signature_demo

# 4. Form filling and flattening
cargo run --example form_processing_demo

# 5. Document merging and splitting
cargo run --example merge_split_demo

# 6. Content stream text extraction
cargo run --example extract_text_demo

# 7. Generate or rebuild all sample test fixtures
cargo run --example generate_sample_pdfs
```

---

## Running Tests

All unit and integration tests run across Ubuntu, macOS, and Windows:

```bash
# Run the complete test suite
cargo test

# Run tests with output
cargo test -- --nocapture

# Run clippy lints
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt --all -- --check
```

---

## Contributing

We welcome contributions from the community! Please see:
- [Contributing Guidelines](CONTRIBUTING.md) for instructions on submitting pull requests and reporting bugs.
- [Code of Conduct](CODE_OF_CONDUCT.md) for community standards.
- [Security Policy](SECURITY.md) for reporting security issues.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

<div align="center">
  <b>Built with ❤️ by <a href="https://techwasti.com">Techwasti Infotech</a></b>
</div>

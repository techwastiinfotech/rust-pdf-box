# API Documentation (Rustdoc)

The complete, item-level Rust documentation for `rust-pdf-box` is generated using Rust's official `cargo doc` tool and bundled alongside this guide.

---

## Direct Links to API Docs

- 🔗 **[Open rust-pdf-box Rustdoc API Reference](api/rust_pdf_box/index.html)**
- 🔗 **[Open on Docs.rs](https://docs.rs/rust-pdf-box)**

---

## Module Overview

The `rust-pdf-box` crate is divided into the following key modules:

| Module | Description |
| :--- | :--- |
| **[`prelude`](api/rust_pdf_box/prelude/index.html)** | The most common types, builders, and enums re-exported for easy glob import (`use rust_pdf_box::prelude::*;`). |
| **[`facade`](api/rust_pdf_box/facade/index.html)** | `PdfDocument` - high-level, context-oriented document management, stamping, and signing facade. |
| **[`cos`](api/rust_pdf_box/cos/index.html)** | Low-level PDF object system: `COSObject`, `COSDocument`, `COSStream`, `COSArray`, `COSDictionary`, parser, and lexer. |
| **[`pd`](api/rust_pdf_box/pd/index.html)** | High-level Page Description layer: `PDDocument`, `PDPage`, `PDRectangle`, `PDResources`, and `PDDocumentCatalog`. |
| **[`content`](api/rust_pdf_box/content/index.html)** | Content stream engine: `PDPageContentStream`, path painting, clipping, color models, and graphics states. |
| **[`stamp`](api/rust_pdf_box/stamp/index.html)** | Fluent watermarking, rubber stamps, headers/footers, and page range selectors (`PageRange`, `StampPosition`). |
| **[`sign`](api/rust_pdf_box/sign/index.html)** | Cryptographic signature engine: `Signer`, `SignatureVerifier`, visual appearances, and detached PKCS#7 signing. |
| **[`crypto`](api/rust_pdf_box/crypto/index.html)** | Pure-Rust cryptographic building blocks: ASN.1 DER encoder, X.509 v3 certificate generator, and RSA-SHA256 signer. |
| **[`interactive`](api/rust_pdf_box/interactive/index.html)** | Interactive AcroForms (`PDAcroForm`, `PDFormField`), form flattening, and page annotations (`PDAnnotation`). |
| **[`font`](api/rust_pdf_box/font/index.html)** | Standard 14 Type 1 PDF fonts (`PDType1Font`) and embedded TrueType fonts (`PDTrueTypeFont`). |
| **[`image`](api/rust_pdf_box/image/index.html)** | Image embedding (`PDImageXObject`), JPEG direct pass-through, and PNG with transparency `/SMask`. |
| **[`manipulation`](api/rust_pdf_box/manipulation/index.html)** | Document assembly tools: `PDFMerger` and `PDFSplitter`. |
| **[`text`](api/rust_pdf_box/text/index.html)** | Text extraction engine: `PDFTextStripper`. |
